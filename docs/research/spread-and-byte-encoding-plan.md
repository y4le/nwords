# Spread IDs and byte-word encoding plan

Planning consensus from Codex/Claude Parley for two post-V1 extensions:

- spread positional ID presets, where nearby numeric IDs usually produce less
  visually similar phrases;
- bidirectional arbitrary bytes and text encoded as word phrases.

Transcript:
`/tmp/parley/682705f93766/runs/workflow-architecture-plan-3f00e7b8/transcript.md`.

## Summary

Use the existing `Permutation` layer for spread ID presets. Do not treat a
shuffled dictionary as the main solution.

Use a byte-first scheme for arbitrary data. Text support is a UTF-8 adapter on
top of byte encoding, not a direct character or Unicode-code-point codec.

No implementation is part of this planning step. The next implementation phase
is an affine spread permutation behind the existing `Permutation` trait.

## Decision Lock

1. Close-ID spreading belongs at the integer-domain permutation layer:

   ```text
   id -> permutation over [0, range) -> positional symbols -> words
   ```

   Decode reverses the same path:

   ```text
   words -> positional symbols -> permuted id -> inverse permutation -> id
   ```

2. Dictionary shuffling is only a relabeling tool. It may change which word is
   displayed for a symbol index, but it does not diffuse `id + 1` across word
   positions in a positional base-N encoding.

3. Use `spread` as the user-facing family name. Do not use `secure`,
   `private`, or `encrypted` unless a future keyed cryptographic
   format-preserving encryption scheme is implemented.

4. The first spread implementation should permute the accepted ID range
   `[0, range)`, not the full phrase capacity. This keeps slack behavior
   unchanged: slack states remain rejected by the positional codec.

5. Arbitrary text is encoded as UTF-8 bytes. Unicode is byte-exact by default.
   Do not normalize text by default.

6. The byte scheme name is `word-bytes-v1`. It frames bytes as:

   ```text
   length prefix + payload bytes + zero pad bits to an 11-bit boundary
   ```

   The exact length-prefix wire format is intentionally not locked yet.

## Spread ID Presets

### Problem

With plain positional encoding, neighboring IDs often differ only in the
low-order word:

```text
42 -> ... word_a
43 -> ... word_b
```

Shuffling the dictionary changes `word_a` and `word_b`, but it does not change
the fact that only the low-order symbol moved. This is useful only as visual
relabeling.

### Architecture

The current architecture already has:

```rust
pub trait Permutation {
    fn domain(&self) -> u128;
    fn permute(&self, id: u128) -> Result<u128>;
    fn invert(&self, id: u128) -> Result<u128>;
}
```

`Positional` already composes a `Permutation`, so spread presets should use this
hook rather than introducing a dictionary-specific mechanism.

The spread permutation must be a bijection over `[0, range)`. It must reject
out-of-domain inputs and must not change capacity, range, slack, acceptance
ratio, or word count.

### Initial Implementation

Start with an affine permutation:

```text
y = (a * x + b) mod range
```

Requirements:

- `range > 0`.
- `gcd(a, range) == 1`.
- Decode uses the modular inverse of `a`:

  ```text
  x = a^-1 * (y - b) mod range
  ```

- Parameters are deterministic and preset-owned. There is no secret key in this
  first implementation.

This is dependency-free, easy to audit, and reversible. It makes sequential IDs
look less sequential, but it is not confidentiality.

### CLI Shape

Use preset names that keep the range visible:

```sh
nwords encode 42 --preset dec6-spread
nwords decode "<phrase>" --preset dec6-spread
nwords plan --preset dec6-spread
```

Also add spread siblings for presets where the range makes sense:

- `dec6-spread`
- `dec9-spread`
- `u32-spread`
- `u64-spread`
- `dec18-spread`

The `presets` and `plan` outputs should add a `permutation` field or column
before implementation is considered complete. Plain existing presets should
report `identity`; spread presets should report a stable affine identifier such
as `spread-affine-v1`.

Do not add a free-form `--spread` flag in the first implementation. Presets are
easier to document, test, and reproduce because the affine parameters live with
the range.

### Deferred Spread Work

Feistel permutations are deferred. They are a better fit if affine stepping
does not diffuse enough for user expectations, but they introduce more
parameter and testing complexity. If added, document cycle walking for ranges
that are not powers of two and keep the same `Permutation` contract.

Keyed format-preserving encryption is deferred. If actual confidentiality is a
goal, use a maintained, reviewed cryptographic design rather than inventing one.
NIST SP 800-38G is the relevant reference family for FF1/FF3-style
format-preserving encryption.

## Byte Encoding

### Purpose

The byte scheme should encode arbitrary byte slices to words and decode them
back exactly. It is a base encoding, not compression, encryption, or entropy
generation.

The main library shape should be byte-oriented:

```rust
encode_bytes(&[u8]) -> phrase
decode_bytes(phrase) -> Vec<u8>
```

Text is layered on top:

```rust
encode_text(&str) -> encode_bytes(text.as_bytes())
decode_text(phrase) -> decode_bytes(phrase) -> UTF-8 validation
```

ASCII is just a subset of UTF-8. Unicode input is preserved byte-for-byte.
For example, precomposed and decomposed forms remain distinct unless a future
caller explicitly requests normalization.

### Frame

`word-bytes-v1` owns byte framing:

```text
length prefix + payload bytes + zero pad bits
```

The symbol codec then packs the padded bit frame into 11-bit symbols and maps
those symbols through a 2048-word dictionary.

Decode must:

- read the length prefix;
- recover exactly the declared number of payload bytes;
- reject non-zero pad bits;
- reject trailing data after the declared payload;
- reject length mismatches;
- reject unknown words without echoing the unknown word text.

The exact prefix format remains open. Options:

- fixed-width length prefix, simpler and bounded;
- varint length prefix, better for short inputs but needs canonical varint
  rejection rules.

Do not lock this until the first byte-codec implementation starts.

### Dictionary

The initial dictionary should be the existing BIP-39 English wordlist used as a
positional/base-2048 dictionary. Every CLI/help/README surface that mentions
BIP-39 must keep the existing caveat:

```text
BIP-39 wordlist used as a positional dictionary, not a BIP-39 mnemonic.
```

This byte scheme is not BIP-39. It does not add BIP-39 checksum bits, validate
BIP-39 mnemonic entropy, or derive BIP-39 wallet seeds.

### CLI Shape

Target a small explicit command group:

```sh
nwords bytes encode --text "hello"
nwords bytes decode --text "<words>"
nwords bytes encode --hex deadbeef
nwords bytes decode --hex "<words>"
```

Text aliases can be added after the bytes command is stable:

```sh
nwords text encode "hello"
nwords text decode "<words>"
```

Default output remains script-friendly: phrase only for encode and decoded data
only for decode. Diagnostics and planning details belong behind explicit flags.

## Testing

### Spread Tests

Required tests for affine spread:

- construction rejects `a` values where `gcd(a, range) != 1`;
- construction rejects zero range;
- exhaustive round trips for small ranges;
- boundary round trips for large presets: `0`, `range - 1`, and selected middle
  values;
- no duplicate outputs for small domains;
- `invert(permute(id)) == id` and `permute(invert(id)) == id`;
- out-of-domain values are rejected without changing the existing error model;
- spread preset encode/decode round trips through the CLI;
- `presets` and `plan` clearly report `identity` versus `spread-affine-v1`.

### Byte Tests

Required tests for `word-bytes-v1`:

- empty byte slice;
- one-byte, two-byte, and 11-bit-boundary cases;
- payloads that require 1 through 10 pad bits;
- non-zero pad bits rejected;
- trailing bits/data after declared length rejected;
- length mismatch rejected;
- unknown words rejected without leaking the raw word text;
- deterministic vectors for at least `""`, `"f"`, `"fo"`, `"foo"`, `"hello"`,
  and selected binary payloads such as `00`, `ff`, and `deadbeef`;
- property tests for arbitrary byte slices within a bounded test size.

### Text Tests

Required tests for the UTF-8 adapter:

- ASCII round trips;
- multibyte Unicode round trips;
- invalid UTF-8 produced by byte decode is rejected by text decode;
- byte-exact preservation for non-normalized strings, including precomposed and
  decomposed forms;
- no default Unicode normalization.

## Phase Plan

### Phase A: Planning

Land this document. No implementation.

Stop condition:

- `spread` is the settled family name.
- Security-y names are explicitly banned until real keyed FPE exists.
- `word-bytes-v1` is the settled byte scheme name.
- Text is a UTF-8 adapter over bytes.
- Length prefix details, Feistel, keyed FPE, and normalization flags are
  explicitly deferred.

### Phase B: Affine Spread

Implement `AffinePermutation` behind the existing `Permutation` trait and add
spread preset siblings.

Stop condition:

- library tests cover affine construction and round trips;
- CLI tests cover spread preset encode/decode and reporting;
- stats terminology remains unchanged;
- no new runtime dependencies are added.

### Phase C: Byte Codec

Implement `word-bytes-v1` as a byte-oriented scheme.

Stop condition:

- byte encode/decode round trips;
- canonical padding and length checks are enforced;
- deterministic vectors are committed;
- CLI exposes bytes encode/decode or the library API is complete enough for the
  CLI to be a narrow follow-up.

### Phase D: Text Adapter

Add text encode/decode over the byte scheme.

Stop condition:

- UTF-8 decode validation is explicit;
- no default normalization;
- ASCII and Unicode round-trip tests pass;
- CLI behavior is documented.

## References

- [BIP-39](https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki):
  wallet mnemonic and seed generation. This is separate from positional or byte
  word encoding.
- [RFC 4648](https://www.rfc-editor.org/rfc/rfc4648.html): base encoding
  precedent for arbitrary octets, canonical behavior, alphabet choice, and the
  warning that base encoding does not provide confidentiality or entropy.
- [Base-2048](https://npm.io/package/base-2048): prior art for mapping bytes to
  2048-symbol word alphabets.
- [NIST SP 800-38G](https://csrc.nist.gov/pubs/sp/800/38/g/upd1/final):
  reference family for real format-preserving encryption if future work needs
  keyed confidentiality.
