# Implementation lock

Locked assumptions for the first implementation pass.

## Safety

- Every Rust crate starts with `#![forbid(unsafe_code)]`.
- No unsafe code is allowed in project-owned or vendored source. There are no
  exceptions for unsafe blocks, unsafe functions, unsafe traits, unsafe impls,
  or unsafe FFI.

## Dependencies

- Keep dependencies extremely minimal.
- `nwords-core` starts with no required third-party runtime dependencies.
- Unicode normalization and RustCrypto crates are acceptable in the crates that
  own BIP-39 Japanese and BIP-39 seed behavior.
- Feature flags represent optional capabilities. They are not required merely to
  hide safe dependencies that are part of supported behavior.
- Do not add convenience dependencies for formatting, iterator helpers, error
  boilerplate, test sugar, or code generation unless the architecture docs are
  updated first.

## Implementation order

Start with:

1. Phase 0: Cargo workspace skeleton.
2. Phase 1: core traits, `BitFrame`, error model, word maps, formatters,
   parsers, and `IdentityPermutation`.
3. Phase 2 and 2A: symbol codecs, positional codec, and stats kernel.

Defer:

- Crates.io reservation and publish metadata.
- Non-identity permutations.
- `phf` or generated perfect maps.
- SLIP-39, Niceware, Proquint, and PGP word list adapters.
- Big-integer exact stats math beyond `u128`.
- Security recommendation policy.

## Public claims

- BIP-39 compatibility means vector-compatible indexes, entropy round trips,
  checksum validation, and seed derivation.
- Japanese display byte parity with `rust-bitcoin/bip39` requires the explicit
  ASCII-space formatter.
- Representational capacity is not entropy unless IDs are sampled uniformly from
  the stated range.

