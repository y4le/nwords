# V1 execution plan

Concrete implementation plan for `nwords` V1, based on the settled
architecture decisions and Claude/Codex Parley planning consensus.

## Scope

V1 ships:

- BIP-39 English and Japanese.
- Positional N-word codecs.
- Stats helpers for capacity, word-count, dictionary-size, and ID-range
  planning.
- `Linear` and `Sorted` word maps.
- `IdentityPermutation`.
- Vendored vector tests and positional property tests.

V1 explicitly defers:

- `phf` and perfect generated maps.
- SLIP-39.
- Niceware.
- Proquint.
- PGP word list.
- Non-identity permutations.
- Russian/Turkish BIP-39 wordlists.
- BIP-32/xprv derivation.
- Big-integer exact stats math beyond `u128`.
- CLI/report generation for stats planning.
- Security-bit recommendation policy.

## Phase 0: Workspace Skeleton

Create the Cargo workspace and crate boundaries:

- `crates/nwords-core`
- `crates/nwords-wordlists`
- `crates/nwords-schemes`
- `crates/nwords-bip39-seed`
- `crates/nwords`

Phase-0 decisions:

- `BigEndian11Bit` and `BaseN` live in `nwords-core` because they are pure
  bit/index math.
- `nwords-schemes` wires core traits into ready-made codecs.
- The umbrella `nwords` crate exposes both the builder API and typed aliases.
  Gate the builder behind `std` only if `Box<dyn ...>` becomes a real
  `no_std + alloc` problem.
- Every crate starts with `#![forbid(unsafe_code)]`; no project-owned or
  vendored source may use unsafe code.
- `nwords-core` starts dependency-free. Crypto and Unicode crates are acceptable
  in the crates that need those primitives.

Acceptance gate:

```sh
cargo check --workspace
cargo check -p nwords --no-default-features --features alloc
```

## Phase 1: Core Primitives

Implement in `nwords-core`:

- `BitFrame<B: AsRef<[u8]> = Vec<u8>>`
- `BitView<'a> = BitFrame<&'a [u8]>`
- `Error`
- `WordMap`
- `Formatter`
- `WordParser`
- `TextNormalizer`
- `SchemeFrame`
- `SymbolCodec`
- `Permutation`
- `IdentityPermutation`
- `Linear` and `Sorted` word maps

Rules:

- Reader methods live on all `BitFrame<B>`.
- Writer helpers live only on `BitFrame<Vec<u8>>`.
- Errors must not include input word text.
- Parsing order is normalize whole phrase, split, exact lookup.
- Do not add third-party dependencies for core primitives.

Acceptance gate:

- Bit-frame unit tests cover 8-, 11-, and 16-bit boundaries.
- `Sorted` lookup matches `Linear` on sample lists.
- `IdentityPermutation` rejects out-of-domain IDs and round-trips in-domain IDs.
- `cargo test -p nwords-core`.

## Phase 2: Symbol Codecs And Positional Codec

Implement:

- `BigEndian11Bit`
- `BaseN` positional codec
- positional facade in `nwords-schemes`

Acceptance gate:

- Property tests cover arbitrary range, base, and word count.
- IDs outside `[0, range)` are rejected.
- Boundary IDs `0` and `range - 1` round-trip.
- First rejected representation is rejected.
- No emitted symbol index is outside `WordMap::len(position)`.

## Phase 2A: Stats Helper Kernel And Planner

Implement the exact positional planning kernel in `nwords-core::stats`.

Core pieces:

- `CapacityClass::Exact(u128)`.
- `CapacityClass::BeyondU128 { log2: Log2Estimate }`.
- Uniform and mixed positional capacity calculations.
- `checked_pow_u128`.
- `ceil_log_base` for "range + dictionary size -> words".
- `ceil_root` for "range + word count -> dictionary size".
- Slack and acceptance-ratio calculations for positional rejection.

Rules:

- Keep this layer pure integer math and `no_std + alloc` compatible.
- Use `u128` as the V1 exact state-count boundary.
- Label values above `u128::MAX` as estimates or bounds.
- Do not describe representational capacity as entropy unless uniform random
  sampling is part of the caller's model.

Acceptance gate:

- Hand-computed capacity tests cover word counts 1..24 over dictionary sizes
  `2`, `256`, `1626`, and `2048`.
- Boundary tests cover exact powers and one-over-exact-power cases for
  `ceil_log_base` and `ceil_root`.
- `2048^11` is exact; `2048^12` is `BeyondU128`.
- Property tests prove each inverse calculation returns the smallest valid
  answer for exact `u128` ranges.

## Phase 2B: Stats Planner Public Surface

Expose the planner after the kernel has tests.

Implement:

- A request/solution API for the four common questions:
  - fixed range + dictionary size -> required words;
  - fixed range + word count -> required dictionary size;
  - fixed dictionary size + word count -> representable range;
  - underspecified request -> bounded candidate table.
- Exact integer ratio data for slack and acceptance.
- Rustdoc examples that use "capacity", "range", and "uniform-sample entropy"
  precisely.

Acceptance gate:

- Doctests cover the four questions above.
- No public stats API leaks input words or phrases in errors.
- Advisory decimal formatting remains deferred until release polish unless it is
  simple and clearly separated from the exact integer API.

## Phase 3: BIP-39 English

Implement:

- Vendored English BIP-39 wordlist with sync/provenance notes.
- `Bip39Frame`.
- English `Bip39Parser`.
- `AsciiSpace` formatter.
- `Bip39English` typed alias.
- Umbrella exports for English BIP-39.

Acceptance gate:

- Trezor English vectors pass:
  - entropy -> mnemonic
  - mnemonic -> entropy
  - checksum validation
- Targeted invalid-checksum test returns `InvalidChecksum`.
- `cargo test -p nwords --no-default-features --features "alloc bip39"`.

Seed derivation remains deferred to Phase 5.

## Phase 4: BIP-39 Japanese

Implement:

- Vendored Japanese BIP-39 wordlist with sync/provenance notes.
- `SpecJapanese` formatter using U+3000.
- `RustBitcoinDisplay` formatter using ASCII spaces.
- Japanese parser using normalize-first, split-second, exact lookup.
- `Bip39Japanese` typed alias.

Acceptance gate:

- Trezor Japanese entropy/mnemonic vectors round-trip.
- `bip32JP` phrases with U+3000 separators parse.
- NFKD-heavy phrases and passphrases are accepted by the normalization layer.

Seed derivation and rust-bitcoin display cross-check remain Phase 5.

## Phase 5: BIP-39 Seed Derivation

Implement in `nwords-bip39-seed`:

- PBKDF2-HMAC-SHA512.
- 2048 rounds.
- 64-byte output.
- Salt = `"mnemonic" + NFKD(passphrase)`.
- Password = NFKD(mnemonic).

Use RustCrypto dependencies:

- `pbkdf2`
- `hmac`
- `sha2`
- `unicode-normalization`

These are acceptable dependencies for BIP-39 seed behavior. They do not belong
in `nwords-core`.

Acceptance gate:

- Trezor English and Japanese `seed_hex` vectors pass.
- `bip32JP` `seed` vectors pass.
- `xprv` fields remain intentionally skipped.
- Dev-dependency cross-check against `rust-bitcoin/bip39` passes for display
  parity only with `RustBitcoinDisplay`.

## Phase 5A: BIP-39 Stats Adapter

Implement the BIP-39 planning adapter behind the `bip39` feature.

Rules:

- BIP-39 word counts are spec-fixed: 12, 15, 18, 21, 24.
- Entropy bits are 128, 160, 192, 224, 256 respectively.
- Checksum bits are entropy bits divided by 32.
- Refuse non-spec word counts rather than rounding them into positional
  capacity math.

Acceptance gate:

- Table tests cover all five legal word counts.
- Invalid word counts such as 13, 14, and 25 return a typed error.
- "Minimum legal BIP-39 words for target entropy" returns the smallest legal
  count and refuses targets above 256 bits.

## Phase 6: Public Surface And Release Polish

Finish:

- Umbrella re-exports.
- README usage examples.
- Feature matrix.
- `no_std + alloc` documentation.
- `nwords::stats` re-exports.
- `std`-gated advisory stats formatting if included.
- Repo-local agent skill for architecture decisions:
  `.agents/skills/nwords-architecture/SKILL.md`.
- `cargo doc`.
- CI matrix.
- `cargo publish --dry-run` in dependency order.

Post-V1 CLI planning is tracked separately in
[`cli-plan.md`](cli-plan.md). The CLI should start with positional ID
encode/decode over a `u32` preset and should clearly label BIP-39 wordlists
used positionally as not being BIP-39 mnemonics.

CI gates:

```sh
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo check -p nwords --no-default-features --features alloc
sha256sum -c tests/vectors/SHA256SUMS
```

Documentation cleanup:

- Annotate or split `architecture.md` so V1-current material is clearly
  separate from post-V1 compatibility notes.
- Keep `architecture-decisions.md` as the implementation contract.
- Keep `test-plan.md` as the test oracle map.

## Risks

- The builder path allocates small `Vec<&str>` and parser collections. This is
  acceptable for V1 and should be documented.
- V1 is not `no_std` without `alloc`.
- SLIP-39 will need a post-V1 index-stage extension such as `IndexFrame`.
- Japanese display parity depends on the selected formatter. The default is
  spec-friendly U+3000; rust-bitcoin display parity uses ASCII spaces.
- BIP-39 wordlist provenance must be confirmed and documented before publish.
- Stats language must not blur representational capacity with security entropy.
- Exact stats math intentionally stops at `u128`; larger spaces use estimates
  until a post-V1 BigInt decision is made.

## Ready-To-Start Checklist

- Decide MSRV.
- Confirm BIP-39 wordlist provenance and license notes.
- Decide whether to reserve `nwords` on crates.io before public work.
- Start Phase 0.

## Parley Record

- Workflow: architecture-plan for base V1.
  Collaborator: Claude.
  Result: consensus.
  Transcript: `/tmp/parley/682705f93766/runs/workflow-architecture-plan-f748d36d/transcript.md`.
- Workflow: architecture-plan for stats helper.
  Collaborator: Claude.
  Result: consensus.
  Transcript: `/tmp/parley/682705f93766/runs/workflow-architecture-plan-3e159b91/transcript.md`.
- Workflow: architecture-plan for ID-to-words CLI.
  Collaborator: Claude.
  Result: consensus.
  Transcript: `/tmp/parley/682705f93766/runs/workflow-architecture-plan-753f44d3/transcript.md`.
