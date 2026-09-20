# Variable-length word IDs

Status: original design proposal, 2026-09-19. The Rust and JS mapping is now
implemented as described in [flexible-wordsets-plan.md](flexible-wordsets-plan.md).
That document is the implementation contract where details differ. CLI options
below remain proposed. Existing fixed-shape encodings remain unchanged.

## Motivation and recommendation

Allow a phrase to grow with its integer ID:

```text
animal
adjective animal
adjective adjective animal
...
```

This is useful for counters, tickets, and other dense IDs that start small.
Increasing the accepted ID range should never rename an existing ID. Random
IDs sampled uniformly from a large range generally still need long phrases;
variable length is not a substitute for choosing an appropriate ID range.

The recommended first version has one leading repeatable list followed by a
nonempty fixed suffix. Examples are `adjective*,animal`, `adjective+,animal`,
and `adjective*,color,animal`. There are no fixed prefixes, multiple repeat
groups, alternatives, or nested groups in the initial proposal.

Both Codex and Opus favor a separate, versioned scheme with every shorter
phrase enumerated before the next length. Opus favors a minimum of one
adjective for a human-facing preset: two-word names are easier to recognize in
logs and chat than bare nouns. Zero repeats remains an explicit option for
the shortest possible names. The preset default is not settled.

## Mapping contract

The provisional scheme name is `variable-v1`. Define its mapping over the
nonnegative integers, independently of implementation limits.

For a repeated list of size `A`, a fixed suffix of `s` positions with mixed
capacity `C`, and a minimum repeat count `m`:

```text
expanded shape(k) = repeated list k times, followed by the fixed suffix
tier_capacity(k)  = C * A^k
tier_offset(k)    = sum(C * A^j for j in m..k)  # excludes k
ID               = tier_offset(k) + fixed_rank(expanded shape(k), phrase)
```

Here `k >= m`. `fixed_rank` is the existing identity mixed-radix ordering:
the leftmost word is most significant, and each list's stored order defines
its digit values. It is not alphabetic sorting of the rendered phrase.

Encoding selects the unique tier containing the ID, subtracts its offset,
and encodes that local value in the expanded fixed shape. Decoding determines
`k` from the word count, decodes that fixed shape, and adds the tier offset.
This definition gives every grammatical phrase exactly one integer before
acceptance bounds are applied.

For the single-animal suffix, `C` is simply the number of animals. With the
current 749 adjectives and 333 animals and `m = 0`:

| Words | Shape | ID interval, exclusive upper bound |
|---|---|---|
| 1 | animal | `[0, 333)` |
| 2 | adjective animal | `[333, 249750)` |
| 3 | adjective adjective animal | `[249750, 187063083)` |
| 4 | adjective adjective adjective animal | `[187063083, 140110249500)` |

These are new mappings. In particular, a two-word phrase in the `m = 0`
format has an ID 333 greater than its existing fixed-shape ID. Decoders must
not guess the scheme from the phrase. For `m = 1`, the first tier does match
the existing two-word identity ordering, but later tiers still need the new
scheme.

### Repetition and canonical phrases

Repeating a list repeats an independent digit position. Repeating the same
word is legal: `calm calm cat` is a distinct phrase, not padded `calm cat`.
Avoiding adjacent or global word duplicates would require a different
enumeration scheme and is deferred.

The minimum repeat count is part of the mapping identity. Changing it changes
the first tier and every offset. Initially expose only `m = 0` and `m = 1`;
the mathematical definition also covers larger minimums if later needed.

An alternative is ordinary radix encoding with leading zero-valued adjectives
forbidden. That can preserve fixed-codec numeric values after stripping
leading zero words, but arbitrarily prevents the first dictionary word from
starting many otherwise natural phrases. Prefer the tier mapping. Any future
pad/strip compatibility adapter should be explicitly separate.

### Implementation without tier powers

For `adjective*,animal`, the mapping is equivalently a bijective base-`A`
adjective prefix followed by an ordinary base-`B` animal digit:

```text
encode(id):
    animal = id % B
    q = id / B
    adjectives = []
    while q > 0:
        q -= 1
        adjectives.push(q % A)
        q /= A
    return reverse(adjectives), animal

decode(adjectives, animal):
    q = 0
    for a in adjectives, left to right:
        q = checked(q * A + (a + 1))
    return checked(q * B + animal)
```

Division is integer division; words are represented by zero-based indexes.
All indexes, arithmetic, and resource limits must be validated.

For a longer suffix, extract its ordinary mixed-radix digits right to left
before the bijective loop, and append them left to right during decode. For
minimum `m`, additionally treat the last `m` adjectives immediately before
the suffix as mandatory ordinary base-`A` digits. Only the remaining leading
adjectives use bijective digits. This avoids needing an exact suffix product
or tier power during encoding and decoding.

The tier definition is normative; this loop is an equivalent implementation
for the proposed leading-repeat grammar. Fixed prefixes are deferred because
a tempting extension of this loop can disagree with the per-tier mixed-radix
ordering. An eventual prefix extension needs explicit cross-check vectors.

## Mapping identity and acceptance bounds

Persist the scheme version, pattern including minimum repeats, and exact
ordered wordlists or immutable versions identifying them. List fingerprints
can help detect drift. Appending or reordering words can change existing
mappings and must not happen silently within a format version.

`range` and `maxWords` are acceptance bounds, not inputs to the mapping:

- `range = R` accepts exactly IDs in `[0, R)`.
- `maxWords = M` counts total words, including the fixed suffix.
- Increasing either bound preserves all previously accepted ID/phrase pairs.
- Both bounds apply to encode and decode. A grammatical phrase can still be
  out of range, especially when the range ends partway through a tier.
- If both bounds are supplied, require `R <= capacity_through(M)` so every
  accepted ID can be encoded within the word limit.

For `M >= s + m`:

```text
capacity_through(M) = sum(C * A^k for k in m..=(M - s))
```

An unconstrained pattern has unbounded mathematical capacity. Require an
explicit `range` or `maxWords`; omitting both must not mean "full capacity."
When only `maxWords` is supplied, use its cumulative capacity as the range
only if it is exactly representable by the existing range type. Otherwise
require an explicit supported range, consistently with fixed-shape behavior.
A capacity beyond `u128` is not itself a reason to reject a configuration
that has an explicit representable range.

Initially retain the existing positive exclusive `u128` range model. Thus
`R <= u128::MAX` and `id < R`; this does not expose the full `[0, 2^128)`
domain. Supporting that domain or larger integers is a separate API decision.
JS `bigint` must not silently extend the Rust domain. Defining the abstract
mapping now allows a future arbitrary-precision API to preserve old vectors.

Retain the JS boundary's 32-word and 4096-byte limits, and apply corresponding
bounded parsing to a new CLI surface. These are implementation guards, not a
promise of unlimited inputs. A range-only configuration must fit within the
implementation word cap. Explicit `maxWords` must fit that cap and allow at
least the suffix plus the minimum repeats. Reject oversized inputs before
allocating or performing unbounded work. Require lists of at least two words,
as in the current mixed-radix codec; unary repetition is out of scope.

## Proposed interfaces

These examples describe an interface direction, not callable APIs.

### CLI

```sh
nwords encode 12345 --pattern 'adjective*,animal' --range 1e6
nwords decode 'able aardvark' --pattern 'adjective*,animal' --range 1e6
nwords plan --pattern 'adjective*,animal' --max-words 3
nwords encode 12345 --pattern 'adjective+,animal' --range 1e6
```

Use a distinct `--pattern` flag, mutually exclusive with fixed-shape selection
through `--shape`, `--words`, `--dict`, or existing `--preset` bundles. Reuse
list names and user-list loading through `--list name=path`; referenced-list
validation should include both the repeated list and suffix.

Support postfix `*` for minimum zero and `+` for minimum one only, on the
first list. Always quote patterns: shells can interpret unquoted `*` or
braces before the CLI receives them. General brace quantifiers are deferred.
Reject patterns without a repeat slot; fixed shapes already have a spelling.

### JavaScript and Rust

Use plain structured data for the JS descriptor. A helper such as `repeat()`
could return this data, but should not be required for serialization:

```js
const format = {
  scheme: 'variable-v1',
  pattern: [
    { list: 'adjective', repeat: { min: 0 } },
    'animal',
  ],
  range: 1_000_000n,
};

words.encodeId(12345n, format);
words.decodePhrase('able aardvark', format);
```

Require the structured `min` explicitly. Reject combining `pattern` with
`lists`. Keep existing fixed descriptors and methods compatible. As in the
current JS API, decimal strings can carry integer values through JSON where
native bigint is unavailable. Exact public type names and whether to overload
the existing methods remain implementation-review questions.

In Rust, prefer a separate variable positional facade that reuses named-list
lookup, formatting, parsing, and mixed-radix conventions. Keep integer mapping
in Rust and expose it through WASM, rather than duplicating it in JS. This
proposal does not require a new single-position wordlist trait, a general
grammar engine, or a big-integer dependency.

### Parsing and planning

Decode the whole phrase under an explicitly supplied format. Normalize first
only where configured, then split and perform exact list lookup. For the JS
surface, preserve the existing case-sensitive whitespace splitting and ASCII
space output. Do not silently add case folding, hyphen splitting, or Unicode
normalization.

Word count identifies the repeated segment and the suffix positions even
when lists share words. Tokens within a list must remain unique and must not
contain separators. Whole-phrase decoding does not imply that concatenated
phrases or phrases embedded in prose can be unambiguously separated.

Errors should distinguish invalid configuration, word count, unknown word,
numeric overflow, and out-of-range values using typed errors without echoing
input words. Exact new public variants remain to be designed.

Planning should show tier capacities and ID intervals, cumulative capacity
through a requested word count, and words required for a configured range.
Use checked exact math or `CapacityClass::BeyondU128` as appropriate; never
replace a capacity with the selected range. Without a word bound, report
unbounded mathematical capacity explicitly rather than as a numeric estimate.
Capacity is not security entropy for sequential or assigned IDs.

## Tradeoffs and deferred features

- Removing an adjective can leave another valid, smaller ID. Fixed word
  counts detect this class of missing-word mistake; variable lengths do not.
  If the minimum repeat count would be violated, deletion is rejected, but
  longer phrases still have this weakness. Checksums are a separate feature.
- Phrase length reveals an interval containing the ID. The encoding does not
  conceal the size or ordering of an application namespace.
- Do not initially support spread permutations. A whole-range permutation
  loses the "small ID means short phrase" property and may remap names when
  its domain changes. A future per-tier permutation needs its own contract.
- Keep allocation, randomness, uniqueness checks across stored aliases, and
  collision retries outside the codec, as with existing named shapes.
- Defer prefixes, repeated blocks, multiple repeat slots, duplicate-word
  suppression, arbitrary-precision APIs, and new parsing conveniences.

## Vectors and implementation acceptance criteria

Use toy lists `adjective = [calm, wild]` and `animal = [cat, dog]`:

| ID | `adjective*,animal` | `adjective+,animal` |
|---|---|---|
| 0 | cat | calm cat |
| 1 | dog | calm dog |
| 2 | calm cat | wild cat |
| 3 | calm dog | wild dog |
| 4 | wild cat | calm calm cat |
| 5 | wild dog | calm calm dog |
| 6 | calm calm cat | calm wild cat |
| 7 | calm calm dog | calm wild dog |
| 8 | calm wild cat | wild calm cat |
| 9 | calm wild dog | wild calm dog |
| 12 | wild wild cat | calm calm calm cat |
| 13 | wild wild dog | calm calm calm dog |
| 14 | calm calm calm cat | calm calm wild cat |

For minimum zero, `maxWords = 3` gives capacity 14. ID 13 is accepted and
ID 14 is rejected. With `range = 7`, `calm calm cat` is accepted as ID 6,
while `calm calm dog` is a valid shape rejected as ID 7.

Before implementing public adapters, add durable vectors and properties for:

1. Every tier boundary, both minimums, repeated words, and a multi-list suffix
   with unequal list sizes. Check the streaming algorithm against the
   independent tier-offset definition.
2. Both round-trip directions, uniqueness, and nondecreasing encoded word
   count as IDs increase.
3. Stability when increasing range or maximum words; rejection when bounds
   are too small or inconsistent, including a range ending inside a tier.
4. Overlapping list contents, exact parsing, missing suffix words, empty
   phrases, invalid indexes, and unknown words with positional errors.
5. Numeric boundaries near the largest supported ID, oversized grammatical
   phrases, cumulative capacities beyond `u128`, checked-arithmetic failures,
   and explicit small ranges with beyond-u128 capacity.
6. Explicit scheme selection and unchanged fixed-shape vectors. Changing
   minimum repeats must be tested as a different mapping.
7. CLI and JS parity with Rust, rejection of ambiguous descriptors and
   unsupported grammar, and enforcement of word/byte limits in both directions.

This is proposed coverage, not a new current-release gate. If implementation
is scheduled, first settle the descriptor and error types and promote the
mapping and vectors into the architecture decisions, test plan, and execution
plan. Library implementation should precede CLI and JS adapters.
