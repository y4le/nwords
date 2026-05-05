# Landscape: words ↔ ID encoders

A survey of crates and reference implementations that map between symbolic
phrases (words / pronounceable chunks / mnemonics) and structured payloads
(bytes / integers / IDs). Compiled to scope the new `nwords` crate and
identify whether the generalization we want already exists.

This document is the result of multi-agent web research (Claude + Codex +
Gemini, May 2026). Where the agents converged, the finding is stated
directly. Where they disagreed, the disagreement is preserved.

## Top-line conclusion

**The gap is real.** No published Rust crate today exposes a generic
`Wordlist + BitPacker + Frame` set of traits configurable to produce
*both* BIP-39 byte-identical output *and* an arbitrary positional
N-word codec. The closest two contenders — `wordlists` and `base-d` —
are described below; neither qualifies.

---

## Classification key

- **single-purpose** — built for one spec, not parameterizable.
- **generalization** — exposes pluggable wordlist / encoding axes; could
  be extended to other schemes.
- **non-codec** — handles wordlists for adjacent purposes (passphrase
  generation, random-word selection) but does not provide reversible
  ID↔phrase encoding.

## Rust: BIP-39 family (single-purpose)

| Crate | Latest | Date | License | Notes |
|---|---|---|---|---|
| [`bip39`](https://crates.io/crates/bip39) (rust-bitcoin) | 2.2.2 | 2025-12-04 | CC0-1.0 | Canonical. `no_std`, all 10 BIP-39 languages behind features, `zeroize` support. ~10M downloads. |
| [`tiny-bip39`](https://crates.io/crates/tiny-bip39) | 2.0.0 | 2024-10 | MIT/Apache | Older fork (maciejhirsz). ~17M downloads but largely historical; many projects migrated to `rust-bitcoin/bip39`. |
| [`bip0039`](https://crates.io/crates/bip0039) | 0.12.0 | 2024-02-19 | MIT/Apache | Smaller alternative, `no_std`, all languages. |
| [`coins-bip39`](https://crates.io/crates/coins-bip39) | 0.13.0 | 2025-07-29 | MIT/Apache | Wallet-stack BIP-39 (part of `coins` ecosystem). |
| [`pqbip39`](https://lib.rs/crates/pqbip39) | 0.1.1 | 2025-06-09 | (unverified) | Newer entrant claiming `no_std`. |
| [`walletd_bip39`](https://lib.rs/crates/walletd_bip39) | 0.2.0 | 2023-05-15 | (unverified) | WalletD project's BIP-39. |
| `dusk-bip39`, `parity-bip39`, `fedimint_bip39` | various | various | various | Ecosystem forks. Not generalizations. |

All of these are tightly coupled to the BIP-39 spec (entropy + SHA-256
prefix + 11-bit packing + PBKDF2-HMAC-SHA512 seed derivation). None
expose configurable wordlist/checksum/packing axes.

## Rust: SLIP-39

| Project | Status | License | Notes |
|---|---|---|---|
| [`slip39`](https://crates.io/crates/slip39) | 0.1.1 (2020), abandoned | GPL-3.0-or-later | docs.rs page literally says "is not a library." CLI binary only. |
| [`sssmc39`](https://github.com/yeastplume/rust-sssmc39) | crates.io 0.0.3 (2020); repo active to 2023-04 | Apache-2.0 | The only credible Rust SLIP-39 implementation. Passes Trezor `vectors.json`. NOT `no_std`. The 10-bit-index layer (between word strings and packed bytes) is **private**; an upstream PR or fork is needed to expose `Share::from_indices`/`to_indices`. |
| `rustywallet-hd::slip39` | 0.3.0, MIT | — | **Not** SLIP-39 spec-compliant. Uses HMAC-SHA256 instead of RS1024, lacks Feistel/passphrase layer. **Avoid.** |

## Rust: generic / wordlist-focused — closest neighbors to `nwords`

| Crate | Latest | Date | License | Notes |
|---|---|---|---|---|
| [`base-d`](https://docs.rs/crate/base-d/latest) | 3.0.31 | 2026-01-23 | MIT OR Apache-2.0 | **Closest near-neighbor.** Universal base encoder with custom dictionaries, streaming, SIMD claims, built-in BIP-39 / Diceware / EFF / PGP / NATO lists. **Critical:** its "BIP-39" mode is *dictionary/base encoding*, not BIP-39 wallet-mnemonic semantics — no SHA-256 checksum prefix, no PBKDF2 seed derivation. Real generalization for base-N dictionary encoding; not a wallet-compat layer. |
| [`wordlists`](https://lib.rs/crates/wordlists) | 0.2.0 | 2022-06 | (unstable per author) | Multiplexes BIP-39 / Diceware / PGPfone / S/Key tables behind one API. **Stale.** No checksum framework, no PBKDF2; will not pass BIP-39 vectors. Lookup-table multiplexer only. |
| [`niceware`](https://crates.io/crates/niceware) | 1.0.0 | — | (MIT) | 65,536-word fixed list, 16 bits/word, reversible. No custom wordlist. Single-purpose. |
| [`proquint`](https://crates.io/crates/proquint) | 0.1.0 | — | — | Pronounceable 5-char quintuplets. u16/u32/u64/IPv4 fixed shapes. |
| [`proqnt`](https://crates.io/crates/proqnt) | — | — | — | Better proquint API surface; 16-bit primitives + binary-data support. |
| [`mnemonic`](https://crates.io/crates/mnemonic) | 1.1.1 | 2024-03-18 | MIT | Port of Tirosh's C `mnemonic`. Reversible byte stream. No checksum, fixed wordlist. |
| [`base256`](https://docs.rs/base256) | 0.4.2 | — | — | EFF Short and PGP word-list byte encoders. Fixed base-256. |
| [`eff-wordlist`](https://lib.rs/crates/eff-wordlist) | 1.0.3 | 2024-03-25 | AGPL-3.0 | Wordlist tables/random-word selection only. Non-codec. AGPL is a meaningful licensing concern for downstream use. |
| [`pgen`](https://lib.rs/crates/pgen) | 3.0.0-alpha.1 | 2024-11-20 | ISC | Passphrase generator. Non-codec. |
| [`parity-wordlist`](https://lib.rs/crates/parity-wordlist) | 1.3.1 | 2020-02-07 | GPL-3.0 | Brain-wallet wordlist helper. Non-generic. |
| [`haikunator`](https://crates.io/crates/haikunator) | — | — | — | Heroku-style fancy-name generator. Not reversible. Non-codec. |

## Rust: what3words and location-coding

| Crate | Notes |
|---|---|
| [`what3words-api`](https://crates.io/crates/what3words-api) | HTTP client for the proprietary what3words service. Requires API key. **No offline encoder.** |
| [`what3words`](https://lib.rs/crates/what3words) (0.1.1, 2023-11-15, GPL-3.0) | Older API client, similarly online-only. |
| [`pluscodes`](https://crates.io/crates/pluscodes), [`open-location-code`](https://docs.rs/open-location-code) | Open Location Code (Google). Letter-based, not word-based — different problem. |

The actual what3words product is proprietary: it claims to map ~57T 3m
squares onto 3-word combinations from a curated dictionary, with a
permutation step ensuring nearby squares produce visually distinct word
triples. There is no open-source compatible encoder. `nwords` should
implement the *pattern* (positional integer in arbitrary base, optional
permutation) without claiming compatibility with the proprietary product.

## Other ecosystems (reference implementations)

- **Python `mnemonic`** (Trezor) — canonical BIP-39 reference.
  `Mnemonic(language).to_mnemonic(entropy) / to_seed(phrase, passphrase)`.
- **Python `shamir-mnemonic`** (Trezor) — SLIP-39 reference. Clean module
  boundaries (`wordlist.py`, `rs1024.py`, `cipher.py`, `shamir.py`,
  `share.py`). Test vectors at
  https://github.com/trezor/python-shamir-mnemonic/blob/master/vectors.json
- **JS `bip39`** (bitcoinjs) — `entropyToMnemonic`, `mnemonicToEntropy`,
  `mnemonicToSeed`. Same mental model.
- **JS `niceware`** — `bytesToPassphrase` / `passphraseToBytes`. Direct
  ancestor of the Rust crate.
- **Go `tyler-smith/go-bip39`** — adds `NewEntropy`, `IsMnemonicValid`.
- **C `mnemonic` (Oren Tirosh)** — origin of the byte-stream-to-words
  pattern; ported to many languages.

## Schemes intentionally **out of scope** for `nwords` v1

These are documented here because they came up in research and may
become future adapters, but they are not v1 targets.

| Scheme | Why it's interesting | Why not v1 |
|---|---|---|
| **Electrum** (V1/V2) | Brute-forces seed phrases until HMAC-SHA512 of the phrase begins with a specific hex prefix (e.g. `01` standard, `100` segwit). | "Generate-then-check" model doesn't fit the same pipeline as BIP-39's "frame-then-pack." Add as scheme adapter once core proves out. |
| **Monero (Electrum-style 25-word)** | 25th word is a checksum derived from CRC32 over the first 24 words' prefixes, mod 24. | Phrase-stage checksum (operates on words, not bits). Architecture supports it via `SchemeFrame`; defer until needed. |
| **Aezeed** | Lightning Network (LND) cipher seed with version, birthday, salt, AEAD-encrypted entropy. | More than a word codec — needs cipher state. |
| **Cardano CIP-3** | Phrase-to-entropy uses BIP-39 directly; differs only in HD key derivation (Ed25519-BIP32). | Not a new word codec. Reuses BIP-39. |
| **BIP-85** | Derives downstream entropy from BIP-32 keys; the words come from BIP-39 again. | Not a word-codec problem; integrates as an entropy source. |

## What `nwords` must do that nothing else does

1. Configurable from a builder (or type-generic typed codec) to produce
   BIP-39-compatible entropy, word indexes, checksum validation, and seed
   derivation, including Japanese U+3000 separator handling and NFKD-heavy
   `bip32JP` passphrase vectors that break half-baked implementations.
2. Configurable to produce a **deterministic positional N-word codec**
   over an arbitrary integer range (need not be a power of N), with an
   optional **permutation layer** for visual distinctness.
3. Expose a clean **words ↔ symbol-index** seam so it can serve as the
   front end of `sssmc39` for SLIP-39, decoupled from the cryptographic
   stack behind the scenes.
4. Keep the architecture open to a **`phf` perfect-hash backend** as one
   of multiple `WordMap` implementations, but defer it until benchmarks
   justify the added build and feature complexity.
5. Stay **`no_std`-friendly** in the core, with `alloc`/`std`/`unicode`
   gated behind features.

## Disagreement flagged

**Performance of `phf` vs. binary search for 1024–2048-entry wordlists.**
Gemini asserts `phf` "significantly outperforms binary search" and is
"O(1) constant lookups, making it ideal for 2048-word datasets." Codex
pushes back: for lists this small, binary search of `&'static str` is
cache-friendly, and the dominant cost in real workloads (BIP-39 seed
derivation, mnemonic validation) is PBKDF2-HMAC-SHA512 (2048 SHA-512
rounds), not the word lookup. Codex's caution is the better-grounded
position; the architecture decision is to ship multiple `WordMap`
backends (`Linear`, `Sorted`, `Phf`, `Perfect`) and let benchmarks
decide per-workload, rather than promise `phf` is universally faster.

## Key links

- BIP-39 spec: https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki
- BIP-39 wordlists: https://github.com/bitcoin/bips/blob/master/bip-0039/bip-0039-wordlists.md
- BIP-39 test vectors: https://github.com/trezor/python-mnemonic/blob/master/vectors.json
- SLIP-39 spec: https://github.com/satoshilabs/slips/blob/master/slip-0039.md
- SLIP-39 test vectors: https://github.com/trezor/python-shamir-mnemonic/blob/master/vectors.json
- rust-bitcoin/bip39: https://github.com/rust-bitcoin/rust-bip39
- yeastplume/rust-sssmc39: https://github.com/yeastplume/rust-sssmc39
- base-d: https://docs.rs/crate/base-d/latest
- PGP word list (Wikipedia): https://en.wikipedia.org/wiki/PGP_word_list
- Open Location Code (Wikipedia): https://en.wikipedia.org/wiki/Open_Location_Code
