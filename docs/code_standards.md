# Code standards

Minimal engineering rules for `nwords`. These are adapted from the Stratent
standards, with project-specific scope for a small Rust word-codec library.

## Rust baseline

- `cargo fmt --check` is clean before merge.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` is
  clean before merge.
- `cargo test --workspace --all-features` passes before merge.
- Workspace-wide `rustfmt.toml` and `clippy.toml`, if added, live at the
  repository root.
- Use the workspace edition consistently across crates.
- V1 public crates must support `std` by default and `no_std + alloc` where the
  feature plan requires it.

## Production code discipline

- No unsafe code, with no exceptions. No `unsafe` blocks, `unsafe fn`,
  `unsafe trait`, unsafe impls, or unsafe FFI in project-owned or vendored
  source.
- Every Rust crate must set `#![forbid(unsafe_code)]` at the crate root.
- No `unwrap()` or `expect()` in library production paths. Tests, examples, and
  one-off development binaries may use `expect()` with a useful message.
- Public APIs return typed error enums, not `anyhow::Error` or
  `Box<dyn Error>`.
- Errors must not include raw input words or phrases. Prefer positions,
  lengths, and enum variants.
- Keep public trait boundaries object-safe where the architecture requires it,
  especially `WordMap`, `Formatter`, `WordParser`, `TextNormalizer`,
  `SchemeFrame`, `SymbolCodec`, and `Permutation`.
- Add abstractions only when they support the settled pipeline or remove real
  duplication.

## Unicode and phrase handling

- Decode parsing order is normalize whole phrase, split, then exact lookup.
  Do not hide per-word normalization inside `WordMap`.
- Keep display formatting separate from parsing.
- BIP-39 Japanese defaults to U+3000 ideographic spaces. ASCII-space display is
  a separate rust-bitcoin parity formatter.
- Be precise about parity claims: index streams, entropy round trips, checksums,
  and seed derivation must match vectors; display-byte parity depends on the
  selected formatter.
- BIP-39 seed derivation uses NFKD normalization, PBKDF2-HMAC-SHA512, 2048
  rounds, and RustCrypto crates. Do not hand-roll cryptographic primitives.

## Stats and estimates

- Do not label a number "entropy" unless the uniform-sampling precondition is
  stated. Assigned, sequential, or user-chosen IDs have representational
  capacity, not inherent security entropy.
- Prefer exact integer math for capacity, range, slack, ceil-log, and ceil-root
  calculations. Values beyond the exact V1 boundary are log-domain estimates
  and must be labelled as estimates.
- Keep BIP-39 planning separate from positional-codec planning. BIP-39 word
  counts are spec-fixed; positional dictionary sizes and word counts are design
  variables.

## Dependencies

- Keep dependencies extremely minimal. The default answer to a new dependency
  is no unless it removes meaningful risk or implements a standard primitive we
  should not own.
- `nwords-core` should have no required third-party runtime dependencies.
- Prefer `core`, `alloc`, and the standard library over small convenience
  crates.
- Use feature flags for optional capabilities, not as ceremony around safe
  dependencies that are part of the supported core behavior.
- Prefer workspace dependency declarations once the Cargo workspace exists.
- New dependencies need a short justification in the commit or PR notes that
  names the risk avoided and the transitive-dependency cost.
- Security-sensitive, cryptographic, and Unicode behavior uses maintained,
  audited crates where possible.
- Do not add a dependency for formatting, iterator helpers, error boilerplate,
  test sugar, or code generation unless the architecture docs explicitly accept
  the cost.
- Do not add generated perfect maps, `phf`, or non-identity permutations in V1
  without updating the architecture decision docs first.

## Tests

- Unit tests live with the crate they test.
- Cross-crate integration tests live under `tests/`.
- Vector tests are required for BIP-39 entropy, mnemonic, checksum, and seed
  behavior.
- Positional codecs need property tests for range boundaries, round trips,
  rejected out-of-range IDs, and rejected first-invalid representations.
- Tests should be able to fail under a plausible broken implementation. Delete
  tautological tests, over-mocked tests, and tests with unconditional success.
- Coverage percentage is not a gate; risky behavior and compatibility contracts
  are the gate.

Required local gates for V1:

```sh
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo check -p nwords --no-default-features --features alloc
sha256sum -c tests/vectors/SHA256SUMS
```

## Vectors and wordlists

- Vendored vectors and wordlists must include provenance notes: source project,
  source URL, commit/tag or retrieval date, license, and any local transform.
- Keep license files for vendored fixtures under `tests/vectors/licenses/`.
- Update `tests/vectors/SHA256SUMS` whenever vector contents intentionally
  change.
- Do not silently edit upstream compatibility vectors to make tests pass.

## Documentation

- Public behavior belongs in crate docs and README examples.
- Architecture decisions belong in `docs/research/architecture-decisions.md`
  until replaced by a more formal decision-record structure.
- Update docs in the same change as public API, feature, or compatibility
  behavior changes.
- Performance claims need benchmarks or should be phrased as design intent.

## Commits

- One logical change per commit.
- Format-only changes stand alone.
- Commit subjects are imperative and no longer than 72 characters.
- Commit bodies explain why when the reason is not obvious from the diff.
