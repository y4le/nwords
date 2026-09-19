# `nwords` test-vector plan

This plan separates V1 conformance tests from post-V1 adapter tests. The
vectors live under `tests/vectors/`.

## V1 required suites

### BIP-39 official/reference vectors

Use `tests/vectors/bip39/trezor-python-mnemonic-vectors.json`.

Required checks for each enabled BIP-39 language:

1. `entropy_hex -> mnemonic` matches the vector's mnemonic.
2. `mnemonic -> entropy_hex` round-trips.
3. `mnemonic + "TREZOR" -> seed_hex` matches the vector.
4. Invalid checksum cases are rejected by targeted local tests.

The Trezor file now includes 12 languages. The architecture currently targets
the 10 languages exposed by `rust-bitcoin/bip39`; Russian and Turkish should
remain skipped unless we deliberately ship those wordlists.

### BIP-39 Japanese stress vectors

Use `tests/vectors/bip39/bip32jp-test_JP_BIP39.json`.

Required checks:

1. Parse generated Japanese phrases with U+3000 separators.
2. Accept NFKD-heavy passphrases such as compatibility symbols and voiced
   kana combinations.
3. Derive the expected seed and xprv-compatible seed bytes.
4. Decide output mode explicitly:
   - `SpecJapanese` emits U+3000 between words.
   - `AsciiDisplay` can match `rust-bitcoin/bip39` display behavior.

Important correction from the earlier architecture note: NFKD does normalize
U+3000 to ASCII space. The Japanese compatibility hazard is still real, but it
is about generation/parsing separators and Unicode normalization order, not
because U+3000 survives NFKD.

### Positional N-word codec

There is no external canonical vector suite because this is the new generic
surface. Use property tests:

1. For arbitrary `range`, `base`, and word count, every accepted ID round-trips.
2. IDs outside `[0, range)` are rejected.
3. Boundary IDs `0`, `range - 1`, and the first rejected representation are
   tested directly.
4. If a permutation is configured, `invert(permute(id)) == id` over the domain.
5. The encoder never emits a symbol index outside the wordlist length for the
   current position.
6. Mixed-radix codecs round-trip boundary IDs over non-uniform per-position
   dictionary sizes and reject the first slack representation.

### Named word-list shapes

Named word-list shapes are curated built-in positional dictionaries, not random
name generators.

Required checks:

1. `WordListSequence` dispatches position `i` to the named list at `shape[i]`.
2. Position 0 is adjectives and position 1 is animals for the compatibility
   adjective-animal shape.
3. Counts and capacity are stable: 749 adjectives, 333 animals, 52 colors,
   1,437 descriptors, 3,051 objects, 64 moods, 64 materials, 40 shapes,
   40 weather terms, 128 plants, 128 foods, 249,417 adjective-animal phrase
   states, 12,969,684 color-adjective-animal phrase states, 4,384,287
   descriptor-object phrase states, 227,982,924 color-descriptor-object phrase
   states, 280,594,368 mood-descriptor-object phrase states, 7,810,560
   material-shape-object phrase states, 15,962,688 mood-adjective-animal phrase
   states, 183,936 descriptor-plant phrase states, 7,357,440
   weather-descriptor-plant phrase states, 11,771,904 mood-descriptor-food
   phrase states, and 327,680 material-shape-food phrase states.
4. Boundary words are stable (`able`/`zippy`, `aardvark`/`zebra`,
   `amaranth`/`yellow`, `abalone`/`zircon`, `aardvark`/`zydeco`,
   `alert`/`zestful`, `acrylic`/`zinc`, `angular`/`zigzag`, and
   `balmy`/`wintry`, `abelia`/`zinnia`, and `almond`/`zucchini`) because
   word order is an encoding contract.
5. CLI `aa` preset maps `0` to the first adjective-animal pair and
   `capacity - 1` to the last pair.
6. CLI `descriptor-object` preset maps `0` to the first descriptor-object pair
   and `capacity - 1` to the last pair.
7. CLI authored semantic presets map `0` to first words and `capacity - 1` to
   last words, including plant and food presets.
8. CLI `plan --shape color,adjective,animal`,
   `plan --shape descriptor,object`, and
   `plan --shape material,shape,object`, and
   `plan --shape weather,descriptor,plant` report per-position list sizes and
   total capacity without requiring a range.
9. CLI `--shape color,adjective,animal` and `--shape descriptor,object`
   round-trip custom ranges.
10. CLI range parsing accepts exact shorthand such as `1e6` and rejects
   non-integer shorthand expansions.
11. CLI rejects the generic `word` list name and suggests `bip39-en`.
12. SHA256SUMS covers upstream snapshots, blocklists, curated snapshots,
   authored snapshots, source READMEs, and upstream licenses.
13. `plant` and `food` authored word lists are disjoint.

### User-defined word-list files

User-defined wordlists are owned runtime lists used only through explicit
`--list name=path` CLI options and ordered `--shape` entries.

Required checks:

1. `--list` is rejected unless `--shape` is present, and is rejected with
   `--preset`, `--dict`, or `--words`.
2. Files must be valid UTF-8, lowercase ASCII, one word per accepted line.
3. Blank lines and full-line `#` comments are ignored.
4. File order is preserved as encoding order.
5. Duplicate words report the duplicate line and first-seen line.
6. Invalid words report the source line.
7. Invalid names, built-in name collisions, duplicate list names, unreferenced
   user lists, too-few words, excessive file size, excessive line length, and
   excessive accepted word count are rejected.
8. Mixed built-in/user-defined shapes round-trip encode and decode.
9. `plan` and `--explain` report user-list word counts and non-security
   `fnv1a64` drift fingerprints.
10. Any shape containing a user-defined list reports `preset: custom`.

### Affine spread permutations

Spread presets use the positional permutation layer. They are not security or
entropy features.

Required checks:

1. Construction rejects zero ranges.
2. Construction rejects affine multipliers where `gcd(multiplier, range) != 1`.
3. Exhaustive small-domain tests prove each output appears exactly once.
4. Boundary tests cover large preset ranges, including `0`, `range - 1`, and
   middle values.
5. CLI spread presets round-trip and report `spread-affine-v1` separately from
   `identity`.

### word-bytes-v1

The byte-word codec uses a fixed 32-bit big-endian byte length, payload bytes,
and zero pad bits to the next 11-bit boundary.

Required checks:

1. Deterministic vectors cover `""`, `"f"`, `"fo"`, `"foo"`, `"hello"`, `00`,
   `ff`, and `deadbeef`.
2. Empty and short payload word counts match the expected 11-bit framing.
3. Non-zero pad bits are rejected.
4. Appended all-zero padding words are rejected as non-canonical.
5. Declared length shorter or longer than available payload bits is rejected.
6. Unknown words are rejected without leaking the raw word text.
7. Text adapters validate UTF-8 and perform no default Unicode normalization.

### Stats helper

The stats helper is a V1 planning contract. It must distinguish
representational capacity from uniform-sample entropy.

Required deterministic tests:

1. Uniform positional capacity:
   - `2^8 == 256`.
   - `10^6 == 1_000_000`.
   - `256^4 == 4_294_967_296`.
   - `2048^11 == 2^121` and fits in `u128`.
   - `2048^12 == 2^132` and returns `BeyondU128`.
2. Mixed positional capacity:
   - `[2, 3, 5]` gives 30 states.
   - A mixed shape that overflows `u128` returns a log-domain estimate.
3. Inverse planning:
   - Fixed range + dictionary size returns the smallest valid word count.
   - Fixed range + word count returns the smallest valid dictionary size.
   - Exact-power and one-over-exact-power cases are both covered.
4. Positional rejection metrics:
   - Capacity equal to range has zero slack.
   - Range below capacity reports `capacity - range` slack.
   - Range above capacity is rejected.
5. BIP-39 stats table:
   - 12, 15, 18, 21, and 24 words map to 128, 160, 192, 224, and 256 entropy
     bits.
   - Checksum bits are 4, 5, 6, 7, and 8 respectively.
   - Non-spec counts such as 13, 14, and 25 are rejected.

Property tests:

1. For exact `u128` ranges, `ceil_log_base(range, base)` is the smallest `k`
   such that `base^k >= range`.
2. For exact `u128` ranges, `ceil_root(range, words)` is the smallest base such
   that `base^words >= range`.
3. If `range <= capacity`, then `slack + range == capacity`.
4. Tradeoff candidates are sorted by word count first and dictionary size
   second, or by another documented stable ordering.

## Post-V1 suites

### SLIP-39

Use `tests/vectors/slip39/trezor-python-shamir-mnemonic-vectors.json`.

Keep this post-V1 unless we expose or own the full SLIP-39 index/share layer.
The vectors cover both successful recovery and expected failures. They should
be run only under a `slip39` feature that depends on an adapter or audited
implementation.

### Niceware

Use `tests/vectors/niceware/diracdeltas-niceware-fixtures.json`.

These fixtures come from the canonical JS package tests. They verify 16-bit
big-endian word indexes, case-insensitive decoding, odd-byte rejection, and
wordlist invariants.

### Proquint

Use `tests/vectors/proquint/proquint-draft-rayner-02.json`.

These fixtures verify the normative CVCVC bit layout, big-endian 16-bit word
order, hyphen-insensitive decoding, case-insensitive decoding, and padding for
odd byte strings where the application chooses to pad.

## Cross-checks

When an implementation crate exists, optional dev-dependency cross-checks can be
added behind an explicit feature or ignored by default:

- `bip39`: compare English and Japanese entropy/mnemonic/seed behavior.
- `niceware`: compare fixture outputs if Niceware is in scope.
- `proqnt` or `proquint`: compare Proquint fixture outputs if in scope.

Do not use cross-check crates as the only test oracle. Vendored vectors remain
the stable oracle; cross-checks are a drift detector.
