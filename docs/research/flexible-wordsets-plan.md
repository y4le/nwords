# Flexible wordsets and byte encoding

Implementation scope, 2026-09-20. Based on the variable-length ID proposal and
EFF landscape research. Existing fixed IDs and `word-bytes-v1` remain stable.

## Commit sequence

1. Freeze EFF long in original dice-roll order with attribution, feature flags,
   and named-list CLI support.
2. Add exact Unicode custom-list construction and indexed lookup, preserving
   the existing strict CLI constructor.
3. Implement `variable-v1` in Rust with independent tier-enumeration tests.
4. Add `radix-bytes-v1` for reversible bytes with arbitrary mixed dictionaries.
5. Expose built-in/custom fixed and variable formats, framed bytes/text, and
   platform-random generation through the Node/browser package. Include owned
   prepared codecs, checked transport, declarations and consumer tests.
6. Record examples, limits, compatibility and measurements; qualify the package.

## Variable integer mapping

Rust `VariablePositional` takes a `WordMap` whose position 0 is the repeated
list and positions 1 onward form a nonempty suffix. Minimum repetitions is 0
or 1. Every shorter tier precedes the next tier; within a tier the existing
leftmost-most-significant mixed-radix ordering applies. Tier capacity is
`C * A^k`, and the tier offset sums these capacities for smaller accepted
repeat counts. Repeated words are valid independent digits.

The implementation extracts the ordinary suffix digits and mandatory repeat
right-to-left, then encodes remaining leading repeats bijectively. Decoding
uses checked multiply/add. It does not need representable tier products.

At least one of exclusive positive `range` and `maxWords` is mandatory. Both
are acceptance bounds, never mapping inputs. When both exist, all IDs below
range must fit the word bound. If maxWords alone has beyond-u128 capacity, an
explicit range is required. Range-only metadata reports unbounded capacity;
requiredWords reports the largest output in the accepted range. The supported
exclusive range is at most `u128::MAX`; JS bigint does not extend that domain.

Rust caps total words at 1024; JS caps them at 32 and input phrase bytes at
4096. The initial grammar has no fixed prefix, additional repeat group, spread
permutation, implicit format detection, or duplicate suppression. CLI `--pattern`
is not added in this scope; the proposal's CLI section remains a future adapter.

## Arbitrary mixed byte encoding

`radix-bytes-v1` uses a cycle of 1–32 lists, each with 2 through u32::MAX
entries. Word position p always uses list p modulo cycle length, including
across block boundaries. Define W(p) as the least positive word count whose
mixed capacity reaches 2^64, and T(r) = (256^r - 1) / 255.

For L = 8q + r payload bytes, encode each full eight-byte block as an ordinary
big-endian integer in W(p) mixed-radix words, then advance p by W(p). A tail
is always present: rank its 0–7 bytes as u = T(r) + big_endian(tail). Enumerate
tail phrase lengths shortest first, with capacity product(radices) per length
and ordinary mixed-radix ranking within each length, to encode u.

On decode, while remaining words exceed W(p), decode a full block and reject
values at least 2^64. The remaining 1..W(p) words form the tail. Add the
shorter-phrase tier offset to its fixed rank; require u < T(8). Find r such
that T(r) <= u < T(r+1), and emit u - T(r) as exactly r bytes. Full blocks
always emit eight bytes. This preserves empty, odd-length and leading-zero
payloads, with one canonical representation per payload.

The tail never exceeds W(p) because T(8) < 2^64. That invariant makes phrase
length determine block/tail boundaries without an extra header. Empty input
is one index-zero word. Exact multiples of eight bytes end in an index-zero
tail word. For EFF long, sixteen payload bytes use eleven words, the last of
which carries no randomness. A 65,536-entry list uses nine words for sixteen
bytes. Repeating an identical list in the cycle does not change the mapping.

No checksum is implied: some substitutions or deletions produce other valid
byte strings. Text is a UTF-8 adapter without normalization. This is reversible
functionality analogous to Niceware, not Niceware wire compatibility, and it
does not change the existing `word-bytes-v1` codec. Persist the version, exact
ordered dictionaries and cycle. Rust owns all byte conversion and integer
mapping. The implementation uses u64 division and bounded u128 accumulators;
there is no arbitrary-precision dependency or quadratic whole-message conversion.

The public block size is fixed at eight bytes. A private layout parameter
supports exhaustive tests with two-byte blocks, including position-dependent
block widths. Invalid block errors carry positions, never decoded payload values.

## Wordlists and JS descriptors

All thirteen named Rust lists are exposed: adjective, animal, color, object,
descriptor, mood, material, shape, weather, plant, food, eff-long, bip39-en.
BIP-39 English is a positional dictionary here, not a wallet mnemonic.

A list source is a canonical built-in name or `{name, words, role?}`. Custom
names are lowercase ASCII identifiers, at most 64 bytes, and cannot collide
with built-in names or aliases. Order is preserved; duplicate tokens and
conflicting definitions of a name are errors. Reusing an identical definition
in several positions is valid. Prepared formats own immutable snapshots.

`OwnedWordList::from_tokens` accepts exact Unicode tokens without whitespace,
controls or BOM. It does not normalize, trim or case-fold. A sorted index
accelerates lookup while preserving encoding order. `OwnedWordList::new`
retains its lowercase-ASCII token policy for existing CLI callers.

JS limits custom tokens to 64 UTF-8 bytes, custom vocabulary to 65,536 unique
list entries across definitions and 1 MiB of token bytes, and formats to 32
list positions. Text and phrase inputs reject lone UTF-16 surrogates to prevent
silent replacement on the JS-to-WASM boundary. The private TSV dictionary
transport independently validates these bounds in Rust before building lists.
User tokens never enter hand-built JSON error messages.

Integer methods accept either fixed `{lists, range?}` or explicit
`{scheme:'variable-v1', pattern:[{list,repeat:{min}}, ...suffix], range?, maxWords?}`.
Byte methods require `{scheme:'radix-bytes-v1', lists}`. Ambiguous formats are
rejected. `prepare` and `prepareBytes` snapshot configuration and have
idempotent `dispose` methods. Existing fixed input descriptors remain valid.

Byte payloads are capped at 4096 bytes in JS, independently of integer word
limits. Byte phrase input is capped at 8 MiB and Rust also bounds tokens using
the maximum permitted frame size. `describe()` exposes block bytes and words.
Rust byte callers supply their own `max_bytes` acceptance limit.

`generatePhrase` uniformly rejection-samples an accepted ID from platform
cryptographic randomness. `generatePassphrase(byteLength, byteFormat)` encodes
that many platform-random bytes. There is no Math.random fallback. Entropy
claims apply to the sampling distribution, never assigned/sequential IDs;
the mandatory tail marker adds no randomness. No collision registry is provided.

## Validation

- Frozen EFF source/index parity and hyphenated CLI round trips.
- Variable toy vectors, independent exhaustive mixed-tier ranking, numerical
  overflow, range/word-bound stability, and beyond-u128 capacity with small range.
- Byte independent binary vectors, mixed radices including 65,536-entry maps,
  empty/odd/zero-prefixed payloads, block/tail slack rejection, UTF-8 fidelity.
- Rust boundary tests independent of JS validation.
- Packed Node, CJS import, Chromium, TypeScript NodeNext/Bundler, existing demo,
  custom snapshot ownership, resource bounds, sanitized errors and disposal.
- Workspace fmt/clippy/tests, minimal features, docs and frozen checksums.

## Consultation decisions

Fable planning request `req_consult_2c4990bb27e6239c` independently prototyped
and enumerated the block/tail mapping above. Adopted its byte layout, tighter
variable-capacity bounds, immutable prepared custom-list snapshots, indexed
lookup and shared vectors. Retained inline custom list descriptors: they are
plain serializable data and naturally compose without a second name registry.
The private bounded TSV transport requires no new parser dependency or builder
lifecycle. Stateless byte helpers are small prepare/call/dispose conveniences.

Did not adopt an NFC-only lint or reject literal U+FFFD: exact Unicode tokens
are intentional, both normalized and decomposed forms have regression tests,
and JS rejects unpaired surrogates before conversion. Random ID rejection
sampling stays in the JS platform adapter; all actual integer/byte codecs stay
in Rust. Neither randomness nor allocation requires a core codec trait.
