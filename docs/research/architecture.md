# `nwords` — architecture

A generic Rust crate for bidirectional encoding between symbolic phrases
(words / pronounceable chunks) and structured payloads (entropy / IDs /
integers), with first-class BIP-39 byte-compatibility and pluggable
support for what3words-style positional codecs, niceware, proquint, the
PGP word list, and (experimentally) SLIP-39 via `sssmc39`.

This document is the synthesized output of multi-agent research
(Claude + Codex + Gemini, May 2026). It supersedes the initial
`Wordlist + BitPacker + Checksum` sketch in the conversation.

The trait signatures below include the follow-up Claude/Codex Parley
consensus from May 2026: bit-aware frames, object-safe builder traits,
normalize-then-split parsing, domain-bound positional permutations, and
a deliberately narrow V1.

## Design principles

1. **Separation of concerns.** Wordlist lookup, symbol-stream packing,
   scheme framing (incl. checksum placement), phrase formatting, and
   optional permutation are independent axes. Each scheme picks one of
   each.
2. **BIP-39 compatibility is non-negotiable.** Entropy, word indexes,
   checksum validation, and seed derivation must match official vectors.
   Japanese display-byte parity with `rust-bitcoin/bip39` is available
   through the explicit ASCII formatter; the default Japanese formatter is
   spec-friendly and emits U+3000.
3. **Dependency-minimal `no_std` core.** `nwords-core` has no required
   third-party runtime dependencies. Unicode and cryptographic primitives use
   maintained crates where they are part of supported behavior.
4. **Don't overpromise performance.** V1 ships `Linear` and `Sorted`
   `WordMap` backends. `phf`/perfect generated maps remain post-V1 until
   real benchmarks justify the extra build and feature complexity.
5. **Don't claim what3words compatibility.** The product is proprietary;
   we implement the *pattern*.
6. **Don't blur capacity and entropy.** Stats helpers must distinguish
   representational capacity, configured ID range, slack/rejection, and
   uniform-sample entropy.

## Trait decomposition

The traits below are the spine. Every supported scheme is a choice of
implementations across the same axes: frame, symbols, words, formatting,
parsing/normalization, and optional permutation.

```rust
/// Owned or borrowed bitstream with an explicit live bit length.
///
/// Reader methods live on all `B: AsRef<[u8]>` forms. Writer helpers such
/// as `push_bits` live only on the owned `BitFrame<Vec<u8>>` form.
pub struct BitFrame<B: AsRef<[u8]> = Vec<u8>> {
    bytes: B,
    bit_len: usize,
}

pub type BitView<'a> = BitFrame<&'a [u8]>;

/// Bidirectional, position-aware mapping between a symbol index and a word.
///
/// `position` is the index of the word within the phrase. Most wordlists
/// ignore it; PGP and similar schemes use it to alternate between
/// odd/even sub-lists. Built-in implementations include `Linear`,
/// `Sorted` (binary search), `Phf` (perfect-hash, feature-gated), and
/// `Interleaved<A, B>` (PGP-style).
pub trait WordMap {
    fn len(&self, position: usize) -> usize;
    fn word(&self, index: usize, position: usize) -> Option<&str>;
    fn index_of(&self, word: &str, position: usize) -> Option<usize>;
}

/// Pack a bit frame into a stream of symbol indices, and the inverse.
///
/// Each concrete codec owns its radix/shape configuration. The facade
/// validates produced indices against `WordMap::len(position)`.
///
/// BIP-39: 11-bit big-endian over 2048-symbol base.
/// SLIP-39: 10-bit big-endian over 1024-symbol base.
/// what3words-pattern / positional-N-word: integer-in-base-N
/// (range need not be a power of N; we use rejection at the boundary).
/// niceware: 16-bit base-65536.
/// proquint: 16-bit packed into pronounceable quintuplets.
pub trait SymbolCodec {
    fn pack(&self, frame: BitView<'_>) -> Result<Vec<u32>>;
    fn unpack(&self, symbols: &[u32]) -> Result<BitFrame>;
}

/// Translate raw payload (entropy, ID, integer) into a bit frame ready
/// for symbol-packing, and the inverse.
///
/// - BIP-39: append SHA-256(entropy)[..n/32] checksum bits → frame.
/// - SLIP-39: future extension; its RS1024 checksum operates on packed
///   indices, so it should use a post-V1 index-stage hook rather than
///   pretending `SchemeFrame` is the universal checksum site.
/// - Niceware / proquint: identity.
/// - Monero (future): identity at frame stage; checksum applies post-
///   phrase via a separate `PhraseFinalizer` adapter (out of v1 scope).
pub trait SchemeFrame {
    fn frame(&self, payload: &[u8]) -> Result<BitFrame>;
    fn unframe(&self, frame: BitView<'_>) -> Result<Vec<u8>>;
}

/// Render words into a phrase string. Output-only and object-safe.
pub trait Formatter {
    fn join(&self, words: &[&str]) -> String;
}

/// Normalize and split input phrases for word lookup.
///
/// Parsing order is normalize whole phrase, then split, then exact lookup.
/// For BIP-39 Japanese, this means U+3000 is normalized before splitting,
/// and the word map does not need per-word normalization.
pub trait WordParser {
    fn normalize_phrase<'a>(&self, phrase: &'a str) -> Cow<'a, str>;
    fn split<'a>(&self, phrase: &'a str) -> Box<dyn Iterator<Item = &'a str> + 'a>;
}

/// Normalize text for seed derivation and other text-level transforms.
pub trait TextNormalizer {
    fn normalize<'a>(&self, text: &'a str) -> Cow<'a, str>;
}

/// Optional reversible permutation over the integer ID. Used to provide
/// what3words' "neighbouring IDs map to visually distinct phrases"
/// property without bleeding into the wordlist itself.
pub trait Permutation {
    fn domain(&self) -> u128;
    fn permute(&self, id: u128) -> Result<u128>;
    fn invert(&self, id: u128) -> Result<u128>;
}
```

### Why this beats the original `Wordlist + BitPacker + Checksum` sketch

- **`SchemeFrame` replaces `Checksum`.** A naive `Checksum` trait
  assumes one canonical insertion point. Real-world schemes don't
  agree: BIP-39 prepends/appends at the entropy layer (pre-pack);
  SLIP-39 RS1024 operates on the packed *index* layer; Monero's
  checksum is a post-pack *word* selection. Modeling this as
  "frame the payload, then pack, then format" is more accurate
  and more general. Internally the BIP-39 frame implementation
  computes SHA-256 of entropy and appends the checksum bits — the
  thing the user thinks of as the BIP-39 checksum is just one
  `SchemeFrame` impl.
- **`WordMap` takes a `position`.** This is the cleanest way to
  support PGP's alternating odd/even lists. A separate
  `Interleaved<A, B>` adapter implements the trait for the common
  case; non-interleaved wordlists ignore the parameter.
- **`Formatter`, `WordParser`, and `TextNormalizer` are split out.**
  Japanese BIP-39 generation should use U+3000 ideographic spaces, while
  broad parsing should normalize the whole phrase before splitting and
  seed derivation should normalize the mnemonic and passphrase text before
  PBKDF2. Keeping these separate avoids smuggling separator, lookup, and
  seed-normalization rules into the wordlist.
- **`Permutation` is optional and orthogonal.** what3words-style
  visual-distinctness wraps around the bit-packing without entering
  the wordlist or framing layers.

### Construction style

We support both:

```rust
// Builder (std-friendly, dynamic dispatch over Box<dyn ...>):
let codec = WordCodec::builder()
    .wordmap(wordlists::bip39::ENGLISH)
    .symbol_codec(SymbolCodec11Bit)
    .frame(Bip39Frame)
    .formatter(AsciiSpace)
    .parser(Bip39Parser)
    .text_normalizer(Nfkd)
    .build();

// Type-generic typed codec (no_std-friendly, zero-cost):
type Bip39English =
    Codec<Bip39EnglishWordMap, BigEndian11Bit, Bip39Frame, AsciiSpace, Bip39Parser>;
const CODEC: Bip39English = Codec::new();
```

The type-generic form is the recommended path for `no_std` users and hot
inner loops. The builder is more ergonomic for application-level code that
picks a wordlist at runtime, and uses object-safe trait objects for each
axis.

## Compatibility matrix

| Scheme | v1? | WordMap | SymbolCodec | SchemeFrame | Formatter | Permutation | Notes |
|---|---|---|---|---|---|---|---|
| BIP-39 English | yes | `Sorted` over English BIP-39 list | `BigEndian11Bit` | `Bip39Frame` (SHA-256 prefix bits) | `AsciiSpace` | None | passes `python-mnemonic/vectors.json`; seed derivation gated |
| BIP-39 Japanese | yes | `Linear`/`Sorted` over Japanese BIP-39 list | `BigEndian11Bit` | `Bip39Frame` | `SpecJapanese` (U+3000) plus `RustBitcoinDisplay` (ASCII) | None | passes `bip32JP`; rust-bitcoin display parity only with ASCII formatter |
| Positional N-word (what3words pattern) | yes | user-supplied | `BaseN` w/ rejection at range boundary | identity/integer frame | user-choice | `IdentityPermutation` | range need not be N^k |
| Niceware | post-V1 | fixed 65,536-list | `BigEndian16Bit` | identity | `AsciiSpace` | None | fixtures vendored |
| Proquint | post-V1 | implicit (5-char alphabet) | `Proquint16Bit` | identity | `Hyphen` | None | fixtures vendored |
| PGP word list | post-V1 | `Interleaved<EvenPgp, OddPgp>` | `BigEndian8Bit` (per byte) | identity | `AsciiSpace` | None | parity-aware via `position` arg |
| SLIP-39 share | post-V1 | `Sorted` over SLIP-39 1024-list | `BigEndian10Bit` + future index-stage hook | future `IndexFrame`/RS1024 adapter | `AsciiSpace` | None | vectors vendored; do not force into V1 frame pipeline |
| Electrum / Monero / Aezeed / Cardano | future | — | — | — | — | — | see landscape.md "out of scope" table |

## Module / crate layout

A workspace with an umbrella crate. This isolates heavy/optional deps
and keeps `nwords-core` lean and `no_std`-clean.

```
nwords/                   # workspace root
├── Cargo.toml            # workspace manifest
├── crates/
│   ├── nwords-core/      # traits + symbol codecs + frame primitives
│   │                     # exact stats kernel; no_std, no heavy deps,
│   │                     # no built-in wordlists
│   ├── nwords-wordlists/ # built-in V1 wordlists (BIP-39 English/Japanese)
│   │                     # with sync notes; more lists post-V1
│   ├── nwords-schemes/   # ready-made codecs:
│   │                     #   bip39::English, bip39::Japanese,
│   │                     #   positional::ThreeWords<Range>
│   ├── nwords-bip39-seed/# PBKDF2-HMAC-SHA512 seed derivation +
│   │                     # NFKD normalization. Optional. Behind
│   │                     # `bip39-seed` feature.
│   ├── nwords-slip39/    # post-V1 SLIP-39 integration; must not impose
│   │                     # heavy deps on V1/default users.
│   └── nwords/           # umbrella crate that re-exports under
│                         # cargo features.
├── docs/
│   └── research/         # this document, landscape.md
└── tests/
    └── vectors/          # vendored test vectors (see "test plan")
```

### Feature flags (umbrella crate)

```toml
[features]
default = ["std", "alloc", "bip39", "positional", "stats"]
std = ["alloc", "nwords-core/std"]
alloc = ["nwords-core/alloc"]
stats = ["nwords-core/stats"]
bip39 = ["nwords-schemes/bip39-english", "nwords-wordlists/bip39-english"]
bip39-japanese = [
    "bip39",
    "nwords-schemes/bip39-japanese",
    "nwords-wordlists/bip39-japanese",
]
bip39-seed = ["alloc", "nwords-bip39-seed"]
positional = ["nwords-schemes/positional"]
serde = ["nwords-core/serde"]
zeroize = ["nwords-core/zeroize"]
```

The pure stats kernel is small enough to ship by default. `std`-gated advisory
formatting can live behind the existing `std` feature; BIP-39-specific stats
live behind `bip39` because BIP-39 word counts are spec-fixed rather than a
general positional planning variable.

This feature sketch is capability-level. Safe dependencies such as
`unicode-normalization`, `pbkdf2`, `hmac`, and `sha2` can be normal dependencies
of the crates that own those capabilities; they do not need extra feature flags
just to hide them.

## `no_std` boundary

| Layer | `no_std` | Requires `alloc` | Requires `std` |
|---|---|---|---|
| `BitFrame`/`WordMap` definitions | ✓ | — | — |
| Alloc-returning traits (`SymbolCodec`, `SchemeFrame`, `Formatter`, `WordParser`) | ✓ | for `Vec`/`String`/`Box`/`Cow` outputs | — |
| Static wordlists (`Sorted`, `Linear`) | ✓ | — | — |
| `Phf` wordlist backend | post-V1 | — | — |
| Phrase formatting (`String`) | ✓ | ✓ | — |
| Unicode normalization (NFKD) | — | ✓ (`unicode-normalization` w/ `alloc`) | — |
| BIP-39 entropy ↔ mnemonic round-trip | ✓ | for `Vec`/`String` | — |
| BIP-39 seed derivation (PBKDF2) | — | ✓ (RustCrypto `pbkdf2`/`hmac`/`sha2`) | — |
| SLIP-39 adapter | post-V1 | post-V1 | post-V1 |

**Crypto choice:** prefer the RustCrypto stack (`pbkdf2`, `hmac`,
`sha2`) for BIP-39 seed derivation. These are acceptable dependencies for
BIP-39 seed behavior; keep them out of `nwords-core`.

## BIP-39 compatibility checklist

Hard requirements for `nwords` BIP-39 mode to be a drop-in for
`rust-bitcoin/bip39`:

1. **11-bit big-endian indexes.** SHA-256 checksum prefix appended to
   entropy *before* slicing into 11-bit groups.
2. **Entropy sizes:** 128 / 160 / 192 / 224 / 256 → 12 / 15 / 18 / 21 /
   24 words.
3. **NFKD normalize** mnemonic *and* passphrase before PBKDF2.
4. **Japanese U+3000 / NFKD trap.** The Japanese wordlist guidance says
   generated phrases should separate words with U+3000 (ideographic
   space, UTF-8 `0xE38080`) and implementations should accommodate
   users entering that separator. NFKD normalizes U+3000 to ASCII space
   for seed derivation, so the compatibility risk is generation/parsing
   behavior plus passphrase normalization order. The canonical stress
   vectors are the `bip32JP` Japanese vectors with compatibility symbols
   and voiced kana combinations such as `㍍` and `ガ`.
5. **PBKDF2-HMAC-SHA512, 2048 rounds, 64-byte seed.** Salt = `"mnemonic"
   + passphrase` (post-NFKD).

## SLIP-39 integration plan

A post-V1 SLIP-39 adapter should preserve the same conceptual layering as the
Python reference:

```
words ──► 10-bit indices ──► packed bits w/ RS1024 ──► Share struct ──► Shamir/Feistel
        ▲                  ▲                        ▲
   not exposed       not exposed         from_u8_vec / to_u8_vec (public)
   phrase adapter
```

The 10-bit-index layer is often not exposed by existing libraries. Options, in
priority order:

1. **Upstream API.** Prefer a maintained implementation exposing
   `Share::from_indices(&[u16])` and `Share::to_indices() -> Vec<u16>`.
2. **Adapter.** Keep `nwords`' generic layer ignorant: a post-V1
   `nwords-slip39` crate takes a `Vec<u16>` of 10-bit indices and delegates to
   an audited implementation behind an explicit feature.
3. **Fork.** Only if an upstream API is unavailable and the dependency cost is
   accepted in a separate architecture decision.
4. **Reimplement.** A full SLIP-39 reimplementation in pure Rust with
   `no_std` support is a stretch goal, not v1 scope. SLIP-39 needs
   GF(256) Lagrange interpolation, RS1024 over GF(1024), Feistel cipher,
   and group/threshold metadata. Substantial work; defer.

`nwords` V1 does not ship SLIP-39. A future `nwords-slip39` adapter must be
feature-gated and must not add heavy dependencies to default users.

## Test vector plan

Vendor test vectors and run on every commit:

| Source | What | URL |
|---|---|---|
| Trezor `python-mnemonic` | BIP-39 multi-language vectors incl. `bip32JP` | https://github.com/trezor/python-mnemonic/blob/master/vectors.json |
| Trezor `python-shamir-mnemonic` | SLIP-39 official vectors | https://github.com/trezor/python-shamir-mnemonic/blob/master/vectors.json |
| `rust-bitcoin/bip39` | optional dev-dep cross-check only | crate dependency |
| `niceware` (JS) | round-trip fixtures | port from JS reference |
| `proquint` reference | u16/u32 fixtures | port from reference |
| internal | property tests for positional N-word codecs over arbitrary base / range | round-trip + range-boundary |

## Performance plan

- Ship multiple `WordMap` backends: `Linear` (tiny lists / tests),
  `Sorted` (compact `&'static [&'static str]` + binary search),
  `Phf` (feature-gated perfect-hash), `Perfect` (codegen'd lookup, for
  built-ins where build-complexity is acceptable).
- Benchmark separately: phrase-parse alone, batch parse, full seed
  derivation. Don't conflate.
- For BIP-39 seed derivation, expect PBKDF2 to dominate. Optimizing
  the lookup is irrelevant to that workload but matters for
  high-throughput phrase-validation use cases (e.g. wallet recovery
  scanners).
- Avoid allocation in hot paths where possible: parse to fixed-size
  arrays for known-length phrases (12/15/18/21/24 for BIP-39).

### Disagreement preserved

Gemini's research claimed `phf` is unambiguously faster than binary
search for 1024–2048-entry lists. Codex pushed back: at this size,
binary search of `&'static str` is cache-friendly and the dominant
cost in real workloads is PBKDF2. **The architectural decision is
neither: ship both and let benchmarks decide.** The `Phf` backend is
behind a feature flag so users who care can opt in; `Sorted` is the
default. Benchmarks will be published with the v1 release.

## Out-of-scope for v1 (deferred)

- Electrum mnemonics (different generation model).
- Monero 25-word + CRC32 prefix checksum.
- Aezeed cipher seeds.
- Cardano CIP-3 (uses BIP-39; not a new word codec).
- BIP-85 entropy derivation (not a word codec; integrates as entropy source).
- Pure-Rust no_std SLIP-39 reimplementation.
- SIMD-accelerated batch encoding (may revisit; `base-d` claims SIMD
  for general base-encoding but for word lookup the first real win is
  avoiding allocation and normalization, not SIMD).

## Open items

- Confirm `rust-bitcoin/bip39` 2.2.2's exact behavior for an
  empty-passphrase seed against our derivation; vendor as a fixture.
- Decide naming: `nwords` is short and catchy but generic; check
  crates.io availability before publishing.
- Decide whether to vendor or re-export the BIP-39 wordlists.
  Re-exporting from `rust-bitcoin/bip39` couples the dep tree;
  vendoring directly from the BIPs repo is cleaner but requires a
  sync process.
- Permutation primitive choice for the positional codec layer:
  format-preserving encryption (FPE / FF1) is the well-trodden path;
  a small Feistel network with a domain-bounded round function is
  simpler and avoids FF1's standard-bound complexity. Punt to v1
  implementation phase.
