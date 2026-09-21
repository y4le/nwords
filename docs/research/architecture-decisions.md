# Architecture decisions

Final Claude/Codex Parley consensus for the V1 implementation.

## Core API

1. Use `BitFrame<B: AsRef<[u8]> = Vec<u8>> { bytes, bit_len }`.
   `BitFrame<&[u8]>` is the borrowed read form. Writer helpers live only
   on owned `BitFrame<Vec<u8>>`.
2. `WordMap::word` returns `Option<&str>` borrowed from `self`, not
   `Option<&'static str>`.
3. `Formatter` is output-only and object-safe:
   `join(&self, words: &[&str]) -> String`.
4. Parsing is normalize-first, split-second, lookup-third. Do not normalize
   per word inside `WordMap`.
5. Keep `WordParser` and `TextNormalizer` separate. Lookup normalization and
   seed-derivation normalization are related but not identical API concerns.
6. `Permutation` is domain-bound at construction and returns `Result` for
   out-of-domain input. V1 ships only `IdentityPermutation`.

## Safety And Dependencies

All project crates must use `#![forbid(unsafe_code)]`. V1 has no exceptions for
unsafe Rust, unsafe FFI, unsafe trait implementations, or unsafe vendored
source.

The implementation lock is recorded in `docs/research/implementation-lock.md`.

Dependency policy is intentionally severe:

- `nwords-core` has no required third-party runtime dependencies.
- Feature flags separate optional capabilities; they are not required merely to
  hide safe dependencies that are part of supported core behavior.
- Prefer implementing small exact-math and parsing helpers locally over pulling
  convenience crates.
- Cryptographic and Unicode primitives are the main acceptable dependency
  category. Use maintained crates for these rather than hand-rolling them.
- Post-V1 adapters must not force heavy dependencies onto V1 users.

## Pipeline

Encode:

```text
payload -> SchemeFrame -> SymbolCodec -> WordMap -> Formatter
```

Decode:

```text
TextNormalizer/WordParser -> WordMap::index_of -> SymbolCodec -> SchemeFrame
```

V1 supports frame-stage transformations, including BIP-39 checksum bits.
Index-stage transformations are reserved for post-V1 schemes such as SLIP-39
RS1024.

## BIP-39 Japanese

Default Japanese display should be spec-friendly and emit U+3000
ideographic spaces. Also ship an ASCII-space formatter for parity with
`rust-bitcoin/bip39` display behavior.

The byte-identical claim must be precise:

- BIP-39 indexes, entropy round-trip, and seed derivation must match vectors.
- Display-byte-identical parity with `rust-bitcoin/bip39` requires the ASCII
  formatter.

## V1 Scope

Ship:

- BIP-39 English and Japanese.
- Positional N-word codec.
- Mixed-radix positional codec for per-position dictionary sizes.
- Curated English named word lists, ordered phrase shapes, and additive CLI
  presets.
- Stats helpers for capacity, ID range, word count, and dictionary-size
  planning.
- `Linear` and `Sorted` word maps.
- Identity permutation only.
- Vendored vector tests and positional property tests.

Defer:

- `phf`/perfect maps.
- SLIP-39.
- Niceware.
- Proquint.
- PGP word list.
- Non-identity bounded permutations.
- Big-integer exact capacity math beyond `u128`.
- Security recommendation policy such as "use at least N bits".

## Features

Initial feature set:

- `std` default; disabling it gives `no_std + alloc`.
- `bip39` default.
- `adjective-animal` default.
- `named` default.
- `bip39-japanese`, using Unicode normalization in the owning crate.
- `positional` default.
- `stats` included by default as pure core math; any advisory formatting is
  gated by `std`.

V1 is not `no_std` without `alloc`.

## Stats And Capacity Planning

The stats helper must keep three distinctions explicit:

1. Representational capacity is the number of phrase states a dictionary shape
   can encode: `dictionary_size.pow(word_count)` for uniform positional codecs,
   or the product of per-position dictionary lengths for mixed shapes.
2. Configured ID range is the application domain accepted by a positional
   codec. If range is smaller than capacity, the remaining states are slack and
   must be rejected.
3. Entropy or security bits are valid labels only when IDs are sampled
   uniformly from the stated range. Assigned, sequential, or user-chosen IDs
   have representational capacity, not inherent security entropy.

Exact capacity math uses `u128` as the V1 exact boundary:

```rust
pub struct Log2Estimate {
    pub lower_bits: u32,
    pub upper_bits: u32,
}

pub enum CapacityClass {
    Exact(u128),
    BeyondU128 { log2: Log2Estimate },
}
```

Values above `u128::MAX` are represented by log-domain estimates, not by a
big-integer dependency in V1. Human-facing advisory formatting is separated
from the exact kernel and may be `std`-gated.

BIP-39 planning is not a free dictionary-size/word-count problem. BIP-39 word
counts are spec-fixed at 12, 15, 18, 21, and 24 words, corresponding to
128, 160, 192, 224, and 256 entropy bits before checksum expansion.

## Named Word Lists And Phrase Shapes

The built-in English named word lists are additive default features, not a
replacement for BIP-39 English positional presets. A phrase shape is an ordered
sequence of named single-position lists such as `adjective,animal`,
`descriptor,object`, `weather,descriptor,plant`, or
`mood,descriptor,food`. The ordered shape is part of the encoding contract.

The `adjective` and `animal` lists are curated from
`andreasonny83/unique-names-generator` commit
`10ff70b131c8a080e88c315a55e45a0f5caadd24`; the `color` list is used from the
same source without local filtering. Source snapshots, blocklists, and license
files live under `tests/vectors/adjective-animal/`. Regenerating from a newer
upstream source or reordering retained words is a new wordlist version, not an
in-place compatibility edit.

The `object` and `descriptor` lists are curated from
`glitchdotcom/friendly-words` commit
`f94b4639c71c26875f7684fa86a214c7f30deaad`; the upstream `generated/words.json`
`objects` and `predicates` arrays are committed as one-word-per-line source
snapshots under `tests/vectors/friendly-words/`. Local blocklists remove poor
user-facing defaults and entries that do not compose cleanly in
`descriptor,object`. The built-in counts are 3,051 objects and 1,437
descriptors, for 4,384,287 `descriptor,object` phrase states and 227,982,924
`color,descriptor,object` phrase states.

The `mood`, `material`, `shape`, `weather`, `plant`, and `food` lists are
authored in-repo from permissive seed references rather than transformed from a
single source list. Their seed references include Princeton WordNet, Wikidata
structured data, USDA PLANTS, OpenFarm, USDA FoodData Central, and for weather
terms the National Weather Service glossary and disclaimer. The frozen counts
are 64 moods, 64 materials, 40 shapes, 40 weather terms, 128 plants, and
128 foods, all in alphabetical order. `plant` and `food` are intentionally
disjoint exact word sets; this does not imply that their meanings never
overlap. The CLI ships `mood,descriptor,object`, `material,shape,object`,
`mood,adjective,animal`, `descriptor,plant`, `weather,descriptor,plant`,
`mood,descriptor,food`, and `material,shape,food` presets after phrase review.

The public adapter is `named::WordListSequence`, which implements the existing
position-aware `WordMap` trait by dispatching position `i` to `shape[i]`.
No additional core trait is needed. The older `AdjectiveAnimal` word map remains
as a compatibility surface for the two-position `adjective,animal` shape.

The CLI exposes custom ordered shapes with `--shape adjective,animal`. Legacy
`--dict adjective-animal` remains as a compatibility alias. Canonical BIP-39
positional shape entries are named `bip39-en`; the generic name `word` is not
accepted because it obscures the BIP-39 positional caveat.

For CLI custom encode/decode over explicit `--shape`, `--words`, or legacy
`--dict` inputs, omitted `--range` means the accepted ID range is the exact
full phrase capacity when that capacity is `CapacityClass::Exact`. If the full
shape capacity is `BeyondU128`, the CLI rejects omitted `--range` and requires
an explicit finite accepted range. Presets keep their configured range; preset
names remain range/shape/permutation bundles.

The CLI exposes `nwords lists` as the built-in shape-list catalog. It reports
canonical names, parser aliases, advisory roles, word counts, and sample words
so users can discover valid `--shape` entries without reading source files.

Preset definitions stay in Rust constants for V1. They are structured as
`name + range + shape + permutation` so a later YAML/codegen layer would be
mechanical, but a non-executable YAML copy is intentionally avoided because it
would drift and a YAML parser dependency is not justified for the current
preset table.

User-defined wordlists are runtime-owned named lists, not new core traits.
The CLI accepts repeatable `--list name=path` entries only with `--shape`;
each provided user-list name must be referenced in the ordered shape. List
files are UTF-8, one lowercase ASCII word per accepted line; outer ASCII
whitespace is ignored before blank-line and full-line `#` comment handling.
The accepted token order is the encoding order. The CLI rejects invalid names,
built-in name collisions, duplicate list names, unreferenced user lists,
duplicate words with line numbers, invalid tokens with line numbers, too-few
words, invalid UTF-8, more than 32 user lists, files over 1 MiB, raw lines over
128 bytes, and lists over 4,096 accepted words.

All CLI shapes resolve to `DynamicWordListSequence`, including pure built-in
shapes. Reports retain per-position metadata so pure built-in shapes can still
be recognized as presets, while any user-defined participation is reported as
`preset: custom`. User-defined lists report an `fnv1a64:<hex>` fingerprint over
accepted tokens with NUL separators. This is a drift and reproducibility aid,
not a security hash.

## Agent Guidance

Repo-local agent skills live under `.agents/skills/<skill-name>/SKILL.md`.
The V1 architecture skill is `.agents/skills/nwords-architecture/SKILL.md` and
should be updated whenever stats, BIP-39, positional-codec, or public API
planning rules change.

Post-V1 planning for spread positional ID presets and arbitrary byte/text word
encoding is recorded in
[`spread-and-byte-encoding-plan.md`](spread-and-byte-encoding-plan.md). That
plan keeps spread behavior in the existing `Permutation` layer and treats text
as a UTF-8 adapter over byte encoding.

## Error Model

Use one `nwords::Error` enum with no string payloads:

```rust
pub enum Error {
    InvalidEntropyLength { got: usize, expected: &'static [usize] },
    InvalidWordCount { got: usize },
    UnknownWord { position: usize },
    InvalidChecksum,
    IndexOutOfRange { value: u128, range: u128 },
    SymbolOutOfRange { index: u32, position: usize, len: usize },
    BitFrameOverflow,
    NormalizationUnavailable,
}
```

Avoid leaking input words in errors. Implement `Display`; implement
`core::error::Error` only if the chosen MSRV supports it, otherwise gate the
error trait impl under `std`.

## JavaScript named-shape package (2026-09-19)

The separate `nwords-js` binding crate enables only alloc, named lists, positional
codecs and stats. Rust owns parsing and mixed-radix math. The JS boundary uses
canonical decimal strings and converts exact results to bigint. Ordered shapes
are bounded to 32 positions and input phrases to 4096 UTF-8 bytes. Metadata and
calls expose only adjective, animal and color; repeated positions are supported.
Beyond-u128 capacity requires an explicit range even for description, consistently
with the plan's range resolution contract. Its capacity remains a tagged estimate,
never the chosen range. This deliberately declines Fable's optional suggestion to
return an unresolved/null range from description.

One web-target binary serves explicit Node and browser ESM entries. Both entries
share initialization: omitted sources reuse the pending/successful instance,
explicit different sources fail, and failed attempts can retry. Byte sources use
object identity and an immediate copy. Each attempt imports a fresh generated-glue
module because wasm-bindgen can set its module state before finalization fails.
The Node loader only reads the packaged asset; web accepts URLs or reusable bytes.
There are no import-time network requests, sync loader or install hooks.

The private tarball records committed source, lockfile hash, artifact hash and
pinned builder versions. Development builds are explicitly marked. Conservative
notices cover all compiled named lists, binding dependencies and Rust's libstd
components, including platform components that may not survive this target's link.
The root MIT/Apache texts make the workspace's existing dual-license declaration
available in the redistributed artifact. Package tests install that artifact into
a temporary consumer; they exercise Node, declarations and real Chromium.

Murmur owns randomness, collision checks and persisted aliases. Its two-word shape
uses the range returned by nwords, sampled with Node's unbiased randomInt. Loading
precedes the registry lock; allocation retries occur inside it and avoid all retained
names. Explicit aliases and canonical session IDs keep their existing semantics.
Initialization failure or retry exhaustion returns an actionable error suggesting
an explicit name. This declines the proposed silent legacy-name fallback: a broken
artifact should be observable, while the explicit-name path remains available.

Fable's diff review found no codec/loader correctness defects. Before clean
qualification we adopted path remapping for workspace/Cargo-home source paths,
source-commit and clean-tree checks around qualification, precise range-error
fields, and read/compile before importing a fresh glue module. Missing or corrupt
assets therefore do not accumulate module records; failures during finalization
still require a fresh attempt. The loader retains sanitized messages rather than
attaching native causes that can contain caller-selected asset paths or URLs.
Murmur retains bounded reason codes for diagnosis without exposing those causes.

### Performance: initialize the already-compiled module directly

The shared asynchronous loader still reads and compiles before importing a fresh
WASM glue module. It then uses the glue's synchronous initializer on that compiled
module, avoiding its generic URL/Request/Response path and Node's lazy web-global
initialization. This does not expose a synchronous public loader or change source
identity, concurrent initialization, sanitized errors, or retry behavior. Packed
Node tests make Request/Response access fail to guard this invariant.

Instantiation itself now runs synchronously inside the asynchronous promise chain.
Compilation remains asynchronous and always precedes the `initSync` call.

### Performance: direct success results

The public JavaScript API retains bigint/canonical-string input and bigint output.
The binding keeps checked decimal parsing for IDs and ranges, including validation
at the raw ABI. Successful encode calls now return the phrase directly and decode
calls return a wasm-bindgen u128/BigInt. Only errors carry the structured JSON
envelope; metadata and legacy diagnostic exports retain JSON. Primitive u128
inputs remain excluded because their generated conversion wraps invalid values.

### Performance: prepared JavaScript codecs

`prepare(shape)` snapshots validated list order and range into a private Rust-owned
word map and MixedPositional codec. Existing stateless calls retain their current
shape-mutation semantics. The frozen JS facade hides the generated WASM handle,
uses its FinalizationRegistry cleanup on supported runtimes, and offers idempotent
`dispose()` for prompt release. Use after disposal fails with DISPOSED/codec before
entering WASM. No cache can evict a live prepared codec, and the object is neither
cloneable nor transferable across instances/workers. Prepared methods retain all
numeric, phrase, range and error contracts of the stateless methods.

## EFF long dictionary

`eff-long` freezes all 7,776 entries of the original EFF long list in dice-roll
order, including its four hyphenated words. It is exposed by the `eff-long`
feature (default in the umbrella crate) and the named-list CLI. It is a
general positional dictionary; no passphrase security claim follows from
encoding assigned IDs. Attribution and the original snapshot are under
`tests/vectors/eff-long/`. Existing lists and mappings remain unchanged.

## Flexible wordsets (post-V1)

The user authorized `variable-v1`, `radix-bytes-v1`, and Node/browser custom
wordsets. The mapping, framing, limits and descriptor contracts are recorded in
[flexible-wordsets-plan.md](flexible-wordsets-plan.md). Fixed IDs and
`word-bytes-v1` are unchanged. EFF long is an immutable optional dictionary.

## Wide variable-v1 (2026-09-20)

`nwords::wide_variable` adds an arbitrary-width natural-number path without
changing the existing `u128` API or the `variable-v1` phrase mapping. The
published browser codec from commit `37fec849` supplies frozen wide phrase
vectors. Rust uses 32-bit limbs with small-radix arithmetic and owns exact
bit, byte, and UTF-8 views; the bit view maps `b` to `int('1' + b) - 1`.
This overrides the earlier post-V1 deferral of wide integer support for the
variable codec. Stats and fixed positional codecs retain their existing
`u128` boundaries.

The CLI has a `names` command for this mapping and resolves built-in lists
inside its single binary. A later dictionary-free Names WASM artifact will
receive selected first-party wordsets through prepared owned maps. English
BIP-39 remains a separate standard-specific WASM artifact.
