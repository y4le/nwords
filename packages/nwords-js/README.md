# nwords for Node and browsers

This private development package contains compiled Rust codecs, WebAssembly,
JavaScript loaders, and TypeScript declarations. Consumers install the packed
tarball with npm; they do not need Rust, wasm-pack, or an install script.
Node 24.14.1 and ordinary browser ESM in the tested Chromium are the initial
qualification targets. Registry publication and broad bundler support are deferred.

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

Shapes have 1–32 ordered canonical list names: `adjective`, `animal`, and `color`.
Repetitions are allowed; aliases and other Rust lists are not part of this API.
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
normalization. Phrase input is limited to 4096 UTF-8 bytes. Persist the package
version, ordered lists, and resolved range when storing reversible encodings.
Display aliases can instead be persisted literally. Capacity is not security
entropy, and encoding does not allocate unique names or provide encryption.

`NwordsError` has a stable `code`, optional `field`, and zero-based `position`
where available. Codes are `INVALID_INPUT`, `UNKNOWN_LIST`, `INVALID_SHAPE`,
`CAPACITY_OVERFLOW`, `OUT_OF_RANGE`, `INVALID_PHRASE`, `DISPOSED`, and `INTERNAL_ERROR`.
Messages do not echo input phrases or words. `NwordsLoadError` separately uses
`LOAD_FAILED` or `ASSET_CONFLICT`. Error classes are exported from both entries.

## Building from the source repository

Use Node 24.14.1, npm 11.11.0, Rust 1.94.0 with `wasm32-unknown-unknown`, wasm-pack 0.14.0,
and the committed Cargo/npm lockfiles. The binding version is pinned to 0.2.120.

```sh
npm ci --prefix packages/nwords-js --ignore-scripts
npx --prefix packages/nwords-js playwright install chromium
npm run build --prefix packages/nwords-js
npm test --prefix packages/nwords-js
```

Normal builds require a clean committed source revision. While implementing,
use `npm run build --prefix packages/nwords-js -- --dev`; both the dirty state
and development mode are recorded. Run `npm test --prefix packages/nwords-js -- --dev`
to test that development artifact. Build output lives under `dist/`, separate
from the static demo's `site/pkg/`. `dist/package-build.json` records packed
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
