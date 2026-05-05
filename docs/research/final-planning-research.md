# Final planning research notes

Focused checks performed after the initial landscape survey.

## Crate name

`nwords` appears unclaimed on crates.io at the time of checking:

- `https://index.crates.io/nw/or/nwords` returned 404.
- `https://crates.io/api/v1/crates/nwords` returned 404 with a user agent.

This is not a reservation. If the name matters, publish a placeholder crate
before the project becomes public.

## BIP-39 details that affect the API

- The BIP-39 checksum is bit-level: append the first `ENT / 32` bits of
  SHA-256(entropy), then split the resulting bitstream into 11-bit indexes.
  A `Vec<u8>` frame without an explicit bit length is not precise enough for
  the core API.
- Japanese phrase generation should support U+3000 ideographic spaces. Broad
  parsing should accept Unicode whitespace. Seed derivation should operate on
  NFKD-normalized mnemonic and passphrase text.
- `rust-bitcoin/bip39` displays mnemonics with ASCII spaces and parses with
  `split_whitespace`. To claim both spec-friendly Japanese output and
  rust-bitcoin display compatibility, `nwords` needs explicit formatter modes.

Recommended API consequence:

- Replace raw byte frames with a `BitFrame`/`BitSlice`-style representation:
  `{ bytes, bit_len }` for owned data, plus borrowed forms where useful.
- Split phrase rendering from phrase normalization. A `Formatter` chooses
  output separators; a `Normalizer`/parser policy prepares input for lookup
  and seed derivation.

## Wordlists and licensing

- The vendored test vectors have clear provenance and license files under
  `tests/vectors/licenses`.
- BIP-39 wordlists should be vendored deliberately from a pinned source during
  implementation. The Bitcoin BIPs repo publishes the wordlists and special
  considerations, but the repo-level licensing story is not as explicit as the
  Trezor vector repositories. Before publishing built-in wordlists, confirm
  provenance and keep a sync note for each list.
- For V1, prefer English + Japanese first. That exercises both sorted ASCII
  lookup and Unicode/separator behavior without committing to every localized
  list immediately.

## Permutation layer

The positional codec needs a bounded bijection over `[0, range)`, not just
`u128 -> u128`.

Options:

1. V1 ships the trait and `IdentityPermutation` only.
2. V1 ships a simple keyed Feistel over the next power-of-two domain with
   cycle walking back into `[0, range)`.
3. V1 depends on FF1/FPE (`fpe` crate exists, MIT/Apache-2.0, no_std-capable
   with feature control) for a cryptographic format-preserving permutation.

Recommendation: choose option 1 for the initial implementation unless visual
distinctness is a hard V1 requirement. Option 2 is reasonable for a deterministic
scrambler, but should not be marketed as cryptographic privacy. Option 3 is
heavier than the positional-codec need.

## SLIP-39

The official Trezor SLIP-39 vectors are vendored, but SLIP-39 should remain
post-V1. `sssmc39` has useful high-level share APIs, but its word/index layer
is internal and the crate is old/std-oriented. Treat SLIP-39 as an adapter or
future implementation once the core word/index abstractions are stable.

## V1 recommendation

Lock V1 around:

1. Core `WordMap`, bit-frame, formatter, normalizer, and error types.
2. BIP-39 English and Japanese entropy/mnemonic/seed conformance.
3. Positional N-word codecs with identity permutation and property tests.
4. `Linear` and `Sorted` word maps.

Defer:

- `phf`/perfect generated maps.
- Niceware, Proquint, PGP, and SLIP-39 implementation work.
- Non-identity bounded permutations.
