# nwords for Node and browsers

This private development package contains compiled Rust codecs, WebAssembly,
JavaScript loaders and wordset modules, and TypeScript declarations. Consumers install the packed
tarball with npm; they do not need Rust, wasm-pack, or an install script.
Node 24.14.1 and ordinary browser ESM in the tested Chromium are the initial
qualification targets. Vite production builds are qualified; registry
publication and other bundlers are deferred.

## Selective imports

Load the dictionary-free Rust Names codec and import only the wordsets your format needs:

```js
import { loadVariable } from '@y4le/nwords/variable/web'; // use /node in Node.js
import { adjective } from '@y4le/nwords/wordsets/adjective';
import { animal } from '@y4le/nwords/wordsets/animal';

const { defineVariable } = await loadVariable();
const codec = defineVariable({
  scheme: 'variable-v1',
  pattern: [{ list: adjective, repeat: { min: 1 } }, animal],
  maxWords: 32,
});
codec.encodeId(42n);                // 'able cardinal'
codec.decodePhrase('able cardinal'); // 42n
codec.decodeBits('able cardinal');   // '01011'
codec.decodeText(codec.encodeText('hello')); // 'hello'
```

The descriptor specifies one repeated list, a fixed suffix, and an explicit
minimum of zero or one repetitions. `maxWords` is required and can be set
through 64. The mapping is `variable-v1`, independent of the word bound. This codec
accepts nonnegative `bigint` or canonical decimal strings and supports values
above `u128`. Bit strings map bijectively through `int('1' + bits) - 1`, so
leading zeros and exact bit length survive. `decodeBytes` and `decodeText`
require a byte-aligned bit view; text decoding uses strict UTF-8. A phrase
does not identify which view was used. Save the scheme, ordered wordsets,
minimum, and word bound with any persisted phrase.
An optional positive exclusive `range` restricts accepted IDs without
changing phrases already in range. `describe()` reports that range, exact
word-bounded capacity, required words, and the largest bit length guaranteed
to fit for every bitstring.
The decoder's UTF-8 phrase limit follows the selected word bound; all phrases
the codec can emit remain decodable, including 64-word custom-list phrases.

Wordset subpaths are independent ESM modules, including `color`, `object`,
`descriptor`, `mood`, `material`, `shape`, `weather`, `plant`, `food`, and
`eff-long`. A production Vite build importing only the example above emits one
dictionary-free Names WASM and omits all unimported wordsets. The tarball carries every optional
wordset and its notices. The existing `@y4le/nwords/{node,web}` entry below
retains its full Rust-backed API.

English BIP-39 has a separate Rust WASM entry:

```js
import { loadBip39 } from '@y4le/nwords/bip39/web';

const bip39 = await loadBip39();
const mnemonic = bip39.encodeEntropy(new Uint8Array(16));
// 'abandon' repeated 11 times, then 'about'
bip39.decodeMnemonic(mnemonic); // original 16 bytes
```

Node uses `@y4le/nwords/bip39/node`. Legal entropy lengths are 16, 20, 24,
28, and 32 bytes. Decoding validates English words, count, and checksum and
returns exact entropy. This API does not derive a seed or generate wallet
entropy. The BIP-39-only Vite build emits one WASM asset and no naming lists.

```js
import { loadNwords } from '@y4le/nwords/node';

const words = await loadNwords();
const shape = { lists: ['adjective', 'animal'] };
const phrase = words.encodeId(42n, shape); // 'able cardinal'
words.decodePhrase(phrase, shape);        // 42n
words.describeShape(shape).range;         // 249417n
words.lists();                           // canonical names, bigint sizes, roles
```

Use `@y4le/nwords/web` in a browser. Serve the packed files with their directory
layout intact, or pass `{ source: new URL('/assets/nwords.wasm', location.href) }`
to `loadNwords`. The exported `@y4le/nwords/wasm` asset subpath lets build tools
locate the binary. The web entry also accepts `ArrayBuffer` or `Uint8Array` bytes.
The Node loader defaults to reading the packaged asset relative to its module,
independent of the working directory, and takes no source options.
Relative source strings resolve against the packaged WASM URL.

Loading is explicit and asynchronous. Importing does not fetch or initialize.
Concurrent calls share one initialization and return the same API. A failed
attempt clears its state and can be retried, including after an invalid module's
start failure. Node and web entries in one process share that state. A pending
or successful load rejects a different explicit source with `ASSET_CONFLICT`. URLs are
compared by normalized href; byte inputs by object identity and are copied
before loading. Omitting a source reuses an existing initialization, or selects
the packaged default when there is none. There is no
promise of an isolated instance per call. Operations are synchronous after load.
CommonJS callers can use dynamic `import('@y4le/nwords/node')`; synchronous
`require()` is not supported. See the packed `examples/` directory.

Shapes have 1–32 ordered list sources. Built-ins are `adjective`, `animal`,
`color`, `object`, `descriptor`, `mood`, `material`, `shape`, `weather`, `plant`,
`food`, `eff-long`, and `bip39-en`. Repetitions and arbitrary combinations are
allowed. `bip39-en` is a positional dictionary here, not a wallet mnemonic.
Custom sources are `{name, words, role?}` objects (see below). Aliases are rejected.
IDs and optional ranges accept `bigint` or canonical decimal strings (`0` or
digits without a leading zero). JavaScript numbers, signs, whitespace, exponent
notation, fractions, and values above `u128::MAX` are rejected. A range is a
positive exclusive count, and `0 <= id < range`. Omit it to use exact full
capacity. A capacity beyond `u128` requires an explicit supported range.

`describeShape` returns ordered `lists`, the resolved `range` as bigint, and
`capacity`: either `{ kind: 'exact', value: bigint }` or
`{ kind: 'beyond-u128', log2: { lower: number, upper: number } }`. The latter is
an estimate of capacity, not the chosen range. Exact list sizes are bigint too;
JSON users explicitly convert bigint values to decimal strings.

Output uses ASCII spaces. Decode uses Rust `split_whitespace` followed by exact,
case-sensitive lookup, with no case folding, hyphen splitting, or Unicode
normalization. Integer phrase input is limited to 4096 UTF-8 bytes. Persist the package
version, scheme, exact ordered dictionaries, pattern, and acceptance bounds when storing reversible encodings.
Display aliases can instead be persisted literally. Capacity is not security
entropy, and encoding does not allocate unique names or provide encryption.

`NwordsError` has a stable `code`, optional `field`, and zero-based `position`
where available. Codes are `INVALID_INPUT`, `UNKNOWN_LIST`, `INVALID_SHAPE`,
`CAPACITY_OVERFLOW`, `OUT_OF_RANGE`, `INVALID_PHRASE`, `DISPOSED`, `NUMERIC_OVERFLOW`,
`INVALID_UTF8`, `RANDOM_UNAVAILABLE`, and `INTERNAL_ERROR`.
Messages do not echo input phrases or words. `NwordsLoadError` separately uses
`LOAD_FAILED` or `ASSET_CONFLICT`. Error classes are exported from both entries.
The selective entries also use `NOT_BYTE_ALIGNED` for a bit view that cannot
be read as bytes, plus the BIP-39 codes `INVALID_ENTROPY_LENGTH`,
`INVALID_WORD_COUNT`, `UNKNOWN_WORD`, and `INVALID_CHECKSUM`.

## Building from the source repository

Use Node 24.14.1, npm 11.11.0, Rust 1.94.0 with `wasm32-unknown-unknown`, wasm-pack 0.14.0,
and the committed Cargo/npm lockfiles. The binding version is pinned to 0.2.120.

```sh
npm ci --prefix packages/nwords-js --ignore-scripts
npx --prefix packages/nwords-js playwright install chromium
npm run build --prefix packages/nwords-js
npm run build:site --prefix packages/nwords-js
npm test --prefix packages/nwords-js
```

Normal builds require a clean committed source revision. While implementing,
use `npm run build --prefix packages/nwords-js -- --dev`; both the dirty state
and development mode are recorded. Run `npm test --prefix packages/nwords-js -- --dev`
to test that development artifact. Build output lives under `dist/`; the
production site is `dist/site/`. `dist/package-build.json` records packed
size and integrity. The template enforces `private: true` and has no lifecycle
scripts. Rust release optimization remains the default. Optional experiments use
`--profile baseline|thin|fat|size`; these custom profiles affect this package build,
not ordinary workspace release builds. An absolute `--wasm-opt /path/to/wasm-opt`
selects Binaryen 131 explicitly. The script checks its version and records its
executable hash and flags; it never downloads an optimizer. The measured profiles
did not establish a throughput win, so no optimizer is enabled by default.

## Repeated calls with one shape

`words.prepare(shape)` validates an immutable snapshot once and reuses the Rust codec:

```js
const codec = words.prepare({ lists: ['adjective', 'animal'] });
codec.encodeId(42n);                 // 'able cardinal'
codec.decodePhrase('able cardinal'); // 42n
codec.dispose();                    // optional prompt release
```

The same bigint/canonical-string inputs, exact-case parsing, Unicode whitespace,
range checks and 4096-byte phrase bound apply. Later changes to `shape` do not
change this codec; stateless calls continue to read their supplied shape each time.
Prepared codecs are local to their loaded WASM instance and cannot be cloned or
transferred to a worker. Create a separate codec in that worker.

Node 24 and the tested Chromium runtime support best-effort reclamation through
FinalizationRegistry; garbage collection and cleanup timing are not guaranteed. `dispose()` releases the Rust allocation promptly and is
idempotent. Runtimes without FinalizationRegistry require explicit disposal.
After disposal, encode/decode throw `NwordsError` with code `DISPOSED` and field
`codec`, before inspecting the operation's input. Ordinary codec errors leave it usable.

## Custom lists and growing IDs

```js
const pets = { name: 'pets', words: ['猫', 'dog', '🦊'] };
const fixed = words.prepare({ lists: ['mood', pets, 'eff-long'] });
const phrase = fixed.encodeId(12345n);
fixed.decodePhrase(phrase); // 12345n
fixed.dispose();

const growing = words.prepare({
  scheme: 'variable-v1',
  pattern: [{ list: 'adjective', repeat: { min: 0 } }, pets],
  range: 1_000_000n,
});
growing.encodeId(0n); // '猫'
growing.encodeId(3n); // 'able 猫'
growing.describe(); // range, unbounded capacity, requiredWords, frozen pattern
growing.dispose();
```

A custom list has at least two unique exact tokens. Tokens may contain Unicode,
case and punctuation, but no whitespace, controls or BOM; each is at most 64
UTF-8 bytes. Names use `[a-z][a-z0-9_-]*`, at most 64 characters, and cannot
collide with built-in names or aliases. The default role is `either`. Order is
never sorted or normalized. A repeated custom list name must have an identical
definition. The total budget across unique custom definitions is 65,536 entries
and 1 MiB of token bytes. Prepared codecs snapshot all names and words; mutating
caller arrays cannot change their mapping. JSON-compatible descriptors can use
decimal strings instead of bigint ranges.

Variable formats require a nonempty fixed suffix and exactly one leading repeat
with explicit `min: 0` or `min: 1`. Set `range`, `maxWords`, or both. `maxWords`
counts the entire phrase, including the suffix, and cannot exceed 32. All
shorter phrases precede longer ones, with ordinary mixed-radix order inside each
length. Increasing either bound preserves existing IDs and phrases; changing
minimum repetitions or dictionaries changes the mapping. `describeShape(format)`
reports cumulative capacity for a word-bounded pattern, or `unbounded` when only
a range is supplied. Repeated words are valid. Decode never guesses the scheme.

## Bytes, UTF-8 text, and random phrases

```js
const format = {
  scheme: 'radix-bytes-v1',
  lists: ['eff-long'],
};
const bytes = words.prepareBytes(format);
const phrase = bytes.encodeBytes(new Uint8Array([0, 255, 0]));
bytes.decodeBytes(phrase); // Uint8Array [0, 255, 0]
bytes.decodeText(bytes.encodeText('hello 世界')); // 'hello 世界'
bytes.describe(); // blockBytes: 8, minBlockWords: 5, maxBlockWords: 5, maxBytes: 4096
const randomPhrase = bytes.generatePassphrase(16); // encodes 16 random bytes
bytes.decodeBytes(randomPhrase).length; // 16
bytes.dispose();

words.encodeText('hello', format); // equivalent stateless methods
words.generatePassphrase(16, format);
words.generatePhrase({ lists: ['mood', 'animal'] }); // uniformly sampled ID
```

Byte formats can use any built-in/custom combination. Word positions cycle
through that ordered template continuously. Full blocks carry eight bytes,
using the fewest words whose mixed capacity covers 2^64; block word count can
vary with the cycle position. A mandatory variable-length tail encodes the
remaining 0–7 bytes. `describe()` reports block byte size and minimum/maximum
block word counts. Repeating an identical list in the cycle preserves the mapping.

Empty payloads use one word. With EFF long, sixteen bytes use eleven words;
with a 65,536-entry custom list, sixteen bytes use nine. Exact multiples of
eight bytes end with an index-zero tail word. Leading zeros and odd lengths
round-trip exactly. Decode rejects unused full-block and tail values. This is
not Niceware wire-compatible and does not alter the older `word-bytes-v1` CLI
format. It has no checksum or encryption; valid substitutions or deletions can
change the decoded bytes. Save the scheme and exact ordered cycle to decode later.

JS byte/text payloads are limited to 4096 bytes. Byte phrase inputs are limited
to 8 MiB and the maximum word count for that template's frame; integer limits
remain 32 words/4096 phrase bytes. Text preserves UTF-8 exactly and rejects lone
UTF-16 surrogates; decoding invalid UTF-8 throws `INVALID_UTF8`.

Random generation uses platform `crypto.getRandomValues`, with rejection sampling
for integer ranges and no `Math.random` fallback. Random-byte phrases carry
8 × byteLength bits of uniform-sample entropy; the mandatory tail marker adds none.
Assigned IDs do not gain entropy by being encoded. `generatePhrase` samples IDs
uniformly, not phrase lengths; larger variable tiers receive more samples.
These functions do not reserve names or check for collisions.

EFF attribution and bundled license are in `NOTICE.md` and
`notices/wordlists/eff-long/README.md`.
