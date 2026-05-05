# Test Vectors

Vendored and normalized vector files for `nwords` conformance tests.

## BIP-39

- `bip39/trezor-python-mnemonic-vectors.json`
  - Source: `trezor/python-mnemonic` `vectors.json`
  - Commit: `b57a5ad77a981e743f4167ab2f7927a55c1e82a8`
  - License: MIT, see `licenses/python-mnemonic-MIT.LICENSE`
  - Contents: 288 vectors across English, Chinese simplified/traditional,
    Czech, French, Italian, Japanese, Korean, Portuguese, Spanish, Russian,
    and Turkish. Each vector is `[entropy_hex, mnemonic, seed_hex, xprv]`.
  - Use: entropy -> mnemonic, mnemonic -> entropy, mnemonic + passphrase
    `"TREZOR"` -> seed. Only run languages whose wordlists are enabled.

- `bip39/bip32jp-test_JP_BIP39.json`
  - Source: `bip32JP/bip32JP.github.io` `test_JP_BIP39.json`
  - Commit: `ea00e83e94ca4d16c98bda4ddda80cec7e8243ed`
  - License: public domain, see `licenses/bip32JP-public-domain.LICENSE`
  - Contents: 24 Japanese vectors with per-vector passphrase, seed, and xprv.
  - Use: Japanese U+3000 separator acceptance plus NFKD passphrase stress.

## SLIP-39

- `slip39/trezor-python-shamir-mnemonic-vectors.json`
  - Source: `trezor/python-shamir-mnemonic` `vectors.json`
  - Commit: `17fcce14736afe498871d3018e4fa9330443471a`
  - License: MIT, see `licenses/python-shamir-mnemonic-MIT.LICENSE`
  - Contents: 45 vectors: 15 valid recovery sets and 30 invalid/error sets.
  - Use: post-V1 SLIP-39 adapter and `sssmc39`/reference cross-checks.

## Niceware

- `niceware/diracdeltas-niceware-fixtures.json`
  - Source: fixtures extracted from `diracdeltas/niceware` `test/mainTest.js`
  - Commit: `84b054010a56231b0e0da0318d15346dc4487562`
  - License: MIT, see `licenses/niceware-MIT.LICENSE`
  - Contents: encode/decode fixtures, case-insensitive decode fixture, and
    wordlist invariants.
  - Use: post-V1 Niceware compatibility tests.

## Proquint

- `proquint/proquint-draft-rayner-02.json`
  - Source: `draft-rayner-proquint-02`, Section 6.6.
  - Contents: fixed 16-bit values, byte-string examples, and decode rules.
  - Use: post-V1 Proquint compatibility tests.

## Integrity

`SHA256SUMS` records the hashes of the vendored and normalized files in this
directory. Regenerate it after intentional vector updates:

```sh
sha256sum $(find tests/vectors -type f -maxdepth 3 ! -name SHA256SUMS | sort) > tests/vectors/SHA256SUMS
```
