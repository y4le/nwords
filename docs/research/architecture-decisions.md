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

## Agent Guidance

Repo-local agent skills live under `.agents/skills/<skill-name>/SKILL.md`.
The V1 architecture skill is `.agents/skills/nwords-architecture/SKILL.md` and
should be updated whenever stats, BIP-39, positional-codec, or public API
planning rules change.

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
