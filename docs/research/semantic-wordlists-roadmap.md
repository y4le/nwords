# Semantic wordlists roadmap

Roadmap for additional built-in English named word lists that compose into
recognizable phrase shapes. This plan covers:

- `object`
- `descriptor`
- `mood`
- `material`
- `shape`
- `weather`
- `plant`
- `food`

The phase-by-phase implementation plan is recorded in
[`semantic-wordlists-execution-plan.md`](semantic-wordlists-execution-plan.md).

The goal is not to accumulate generic dictionary slices. The goal is to ship
stable, license-clean, phrase-friendly slots that can be combined with
`named::WordListSequence` into memorable ordered shapes.

This is planning text, not a shipped compatibility contract. Once a list ships,
its contents, order, length, and canonical name become part of the encoding
contract. Changing any of those requires a new list name or version.

## Design principles

1. Prefer vetted permissive sources when they are already curated for friendly
   phrase generation.
2. Use permissive reference sources as candidate seeds when no direct source is
   clean enough to vendor as-is.
3. Let curation determine final list sizes. Do not truncate or pad to powers of
   two unless there is a concrete product reason.
4. Preserve upstream order for direct-derived lists. Use frozen alphabetical
   order for authored curated lists.
5. Keep each list grammatically scoped. Presets should compose modifiers before
   head nouns.
6. Treat phrase capacity as tuple count. Do not describe capacity as entropy
   unless IDs are uniformly sampled from the stated range.

## Source posture

Use three source categories:

| Category | Meaning | Ordering rule |
|---|---|---|
| Direct-derived | A pinned upstream list is the authoritative source, with local filtering and blocklists. | Preserve upstream order for retained words. |
| Authored with seed references | The shipped list is authored in-repo; external sources are candidate pools or checks, not byte-for-byte sources. | Frozen alphabetical order. |
| Reference only | The source informs review but should not be transformed into the shipped list. | No ordering relationship. |

Current recommendations:

| List | Source posture | Candidate sources | Target role | Initial size guidance |
|---|---|---|---|---:|
| `object` | Direct-derived | Glitch `friendly-words` `words/objects.txt` | Head | Full filtered source, currently about 3k raw words |
| `descriptor` | Direct-derived | Glitch `friendly-words` `words/predicates.txt` | Modifier | Full filtered source, currently about 1.4k raw words |
| `mood` | Authored with seed references | WordNet, Wikidata, manual review | Modifier | 48-64 |
| `material` | Authored with seed references | WordNet, Wikidata, material vocabularies | Either | 48-64 |
| `shape` | Authored with seed references | WordNet, Wikidata, geometry terms | Either | 24-40 |
| `weather` | Authored with seed references | NOAA public-domain weather glossaries | Modifier | 24-40 |
| `plant` | Authored with seed references | OpenFarm CC0, USDA PLANTS, Wikidata, WFO as reference | Head | 256-512 |
| `food` | Authored with seed references | USDA FoodData Central CC0, FoodOn as reference, Wikidata | Head | 256-512 |

`descriptor` needs one naming decision before it is frozen: it overlaps with
the existing `adjective` list. Keep it only if docs define it clearly as the
Glitch-predicate-derived descriptor list, including attributive participles
when they work before nouns. It should not be described as a synonym for
`adjective`.

## Source notes

### Glitch friendly-words

Repository: <https://github.com/glitchdotcom/friendly-words>

Recommended source commit checked during planning:
`f94b4639c71c26875f7684fa86a214c7f30deaad`.

License: MIT.

Useful files:

- `words/objects.txt`
- `words/predicates.txt`
- `words/teams.txt`
- `words/collections.txt`

Planning-time raw counts from the recommended commit:

- `objects.txt`: 3062 words
- `predicates.txt`: 1450 words
- `teams.txt`: 130 words
- `collections.txt`: 69 words

The README describes these as curated lists intended to be friendly, positive,
memorable, easy to spell, and safe for children across cultures. It also states
that the word files are organized as predicates and direct objects so generated
pairs are more likely to make grammatical sense. That makes this the strongest
direct source for `object` and `descriptor`.

### USDA FoodData Central

URL: <https://fdc.nal.usda.gov/api-guide>

FoodData Central data is public domain and published under CC0. It is a good
seed for `food`, but not a direct list source. The database is nutrition and
product oriented, so raw descriptions include branded, multiword, prepared,
technical, and overly specific entries that do not fit nwords phrase slots.

### OpenFarm

Repository: <https://github.com/openfarmcc/OpenFarm>

Software license: MIT. Data license: CC0.

OpenFarm is a useful seed for `plant`, especially crops and garden plants. It
should not define the full scope of `plant`, because the list should be common,
friendly, and phrase-oriented rather than a gardening database.

### USDA PLANTS

Downloads: <https://plants.sc.egov.usda.gov/downloads>

USDA PLANTS provides downloadable public plant lists, including a complete
checklist with national common names. It is a useful seed/reference for
`plant`, but license/public-domain posture and exact download provenance should
be recorded before using any snapshot. Raw output is taxonomy-oriented and must
be filtered to common, friendly, single-token names.

### NOAA weather sources

Useful starting points:

- National Weather Service glossary:
  <https://forecast.weather.gov/glossary.php>
- NOAA repository entry for "A comprehensive glossary of weather terms for storm
  spotters": <https://repository.library.noaa.gov/view/noaa/7223>

The NOAA repository record marks the storm-spotter glossary public domain. NOAA
sources are good seeds for `weather`, but raw glossary content is too technical
and acronym-heavy for direct use.

### WordNet

URL: <https://wordnet.princeton.edu/license-and-commercial-use>

WordNet permits use, copying, modification, and distribution with notice
preservation. It is useful for candidate generation and sense checks, especially
for `mood`, `material`, `shape`, and `plant`. Raw WordNet slices should not be
vendored directly because they include obscure, technical, ambiguous, and poor
phrase-default words.

### Wikidata

URL: <https://www.wikidata.org/wiki/Wikidata:Licensing>

Wikidata structured data is CC0. It can seed candidates for material, shape,
plant, and food. It is not curated for phrase quality, so every candidate still
needs local filtering and review.

### FoodOn

Repository: <https://github.com/FoodOntology/foodon>

License: CC BY 4.0.

FoodOn is a neutral ontology for food materials and food products. It is useful
as a reference for generic food categories, but its ontology scope and
attribution requirements make it less attractive than USDA FoodData Central as
the primary seed for `food`.

### World Flora Online

URL: <https://wfoplantlist.org/>

WFO is an authoritative open-access plant taxonomy with CC BY 4.0 text/images.
It is a reference source for plant naming and scope, not a direct source for a
friendly `plant` wordlist. It is too broad and scientific-name focused for a
phrase slot.

## Curation policy

Apply the existing adjective/animal policy as the baseline:

- lowercase ASCII only;
- single token only;
- no hyphens, apostrophes, digits, or whitespace;
- remove identity, political, religious, sexual, medical, legal, corporate,
  and administrative terms;
- remove insults, body-shaming terms, and poor user-facing defaults;
- remove very abstract, technical, arcane, or awkwardly ambiguous words;
- preserve the documented ordering rule for the source posture.

Additional category-specific rules:

| List | Extra rules |
|---|---|
| `object` | Prefer concrete head nouns. Remove terms that fail common modifier-object phrase checks. |
| `descriptor` | Keep words that work before a head noun. Participles are allowed when phrases read naturally. |
| `mood` | Positive or neutral affect only. Exclude clinical, diagnostic, hostile, or unstable states. |
| `material` | Prefer common substances and textures that work attributively. Exclude trademarks and chemical hazards. |
| `shape` | Prefer common visual forms. Avoid specialist geometry unless familiar in everyday speech. |
| `weather` | Prefer common weather adjectives or terms that work before a head noun. Exclude alerts, acronyms, scales, and severe-event bureaucracy. |
| `plant` | Common single-token plant nouns only. Exclude Latin binomials, cultivars, drug-associated plants, notably toxic defaults, and highly obscure taxonomy. |
| `food` | Generic single-token foods only. Exclude brands, alcohol/drugs, medical/allergen framing, preparations that require multiword names, dominant non-food homographs, and very obscure regional items. |

Cross-product review is required. Clean individual words can still make bad
pairs or triples. At minimum, sample each proposed preset shape and review the
highest-risk bigrams produced by adjacent positions.

## Generation process

For each list:

1. Create a source directory under `tests/vectors/<topic>/`.
2. Record source project, source URL, commit/tag or retrieval date, license,
   and local transform notes in a README.
3. Copy license files for vendored direct sources under
   `tests/vectors/licenses/`.
4. Save raw source snapshots when a direct source is used.
5. Normalize candidates to lowercase ASCII single-token words.
6. Apply length policy and category-specific filters.
7. Apply blocklists with one word per line and comments in the README
   explaining the blocklist policy.
8. Dedupe after normalization and after the singular/plural policy is applied.
9. Run commonness and readability screening as a generation-time check.
10. Run phrase-shape sampling for adjacent word combinations.
11. Human-review the final candidate file.
12. Freeze the curated list and ordering.
13. Add Rust arrays and `NamedWordList` variants.
14. Update parser aliases, `name()`, `len()`, `word()`, and `index_of()`.
15. Add boundary tests pinning list length, first word, and last word.
16. Add `SHA256SUMS` coverage for raw snapshots, blocklists, curated snapshots,
    and license files.
17. Add CLI preset tests for any new presets.

Generation scripts are allowed, but the frozen curated files are the contract.
Re-running a script against newer source data after release must produce a new
list name/version if contents or order change.

## API plan

Add variants:

```rust
pub enum NamedWordList {
    Adjective,
    Animal,
    Color,
    Object,
    Descriptor,
    Mood,
    Material,
    Shape,
    Weather,
    Plant,
    Food,
    // ...
}
```

Use singular canonical names:

- `object`
- `descriptor`
- `mood`
- `material`
- `shape`
- `weather`
- `plant`
- `food`

Accept natural plural aliases where they read well:

- `objects`
- `descriptors`
- `moods`
- `materials`
- `shapes`
- `plants`

Do not advertise awkward aliases such as `weathers`. `foods` is acceptable as a
parser convenience if desired, but the canonical display name should remain
`food`.

Add grammatical role metadata for documentation, preset selection, and optional
shape linting:

```rust
pub enum WordListRole {
    Modifier,
    Head,
    Either,
}
```

Recommended roles:

| List | Role |
|---|---|
| `adjective` | Modifier |
| `color` | Modifier |
| `descriptor` | Modifier |
| `mood` | Modifier |
| `weather` | Modifier |
| `animal` | Head |
| `object` | Head |
| `plant` | Head |
| `food` | Head |
| `material` | Either |
| `shape` | Either |
| `bip39-en` | Either |

Role metadata should not change core encoding. Position still disambiguates
words during decode. Initially, use roles for preset design, docs, and optional
CLI warnings rather than hard rejection of custom `--shape` values.

## Preset candidates

All capacities below are illustrative. They use planning-time counts from the
current built-ins and proposed target sizes. Replace them with exact values
once curated counts freeze.

Assumptions used in this table:

- `color`: 52 current words
- `adjective`: 749 current words
- `animal`: 333 current words
- `descriptor`: 1450 raw Glitch predicate words
- `object`: 3062 raw Glitch object words
- `mood`: 64 target words
- `material`: 64 target words
- `shape`: 40 target words
- `weather`: 40 target words
- `plant`: 512 target words
- `food`: 512 target words

| Preset shape | Illustrative capacity | Notes |
|---|---:|---|
| `descriptor,object` | 4,439,900 | Strong direct-derived two-word default. |
| `color,descriptor,object` | 230,874,800 | High-capacity friendly phrase using current `color`. |
| `mood,descriptor,object` | 284,153,600 | Expressive three-word phrase if `mood` is carefully positive/neutral. |
| `material,shape,object` | 7,838,720 | Concrete visual phrase such as material + form + object. |
| `weather,descriptor,plant` | 29,696,000 | Useful after `plant` ships; avoid over-severe weather terms. |
| `descriptor,plant` | 742,400 | Readable but lower capacity. |
| `mood,descriptor,food` | 47,513,600 | Better capacity than `mood,food`; depends on food curation quality. |
| `material,shape,food` | 1,310,720 | Novel, but phrase quality should be sampled before shipping. |
| `mood,adjective,animal` | 30,902,400 | Extends an existing phrase family with a new modifier slot. |

Avoid presets such as `weather,mood,object`; they are grammatically legal as
slots but produce weak phrases. Custom shapes may still allow them if users ask
for them explicitly.

## Implementation phases

### Phase A: Curation harness

- Create shared normalization and validation tooling for wordlist candidates.
- Reuse the existing vector/provenance layout.
- Add checks for lowercase ASCII, single-token entries, duplicates, ordering,
  and length policy.
- Add a phrase sampler that can take a shape and emit representative adjacent
  pairs/triples for human review.

### Phase B: Direct-derived Glitch lists

- Vendor Glitch source snapshots at a pinned commit.
- Add `object` from `words/objects.txt`.
- Add `descriptor` from `words/predicates.txt`, after resolving the naming
  distinction from existing `adjective`.
- Preserve upstream order for retained words.
- Add Rust arrays, `NamedWordList` variants, parse aliases, boundary tests, and
  `SHA256SUMS`.
- Add at least `descriptor,object` and `color,descriptor,object` to CLI preset
  planning if phrase samples pass review.

### Phase C: Small authored semantic modifiers

- Add `mood`, `material`, `shape`, and `weather`.
- Use authored curated lists with documented seed references.
- Freeze alphabetical order.
- Add role metadata and tests.
- Add phrase-shape presets after cross-product sampling.

### Phase D: Plant and food

- Build `plant` candidates from OpenFarm CC0, USDA PLANTS, Wikidata, and WFO
  as a naming reference.
- Build `food` candidates from USDA FoodData Central CC0, with FoodOn as a
  reference source if attribution is acceptable.
- Curate to common, friendly, single-token head nouns.
- Add source provenance that clearly distinguishes direct source snapshots from
  seed references.
- Add presets only after phrase sampling against `descriptor`, `mood`,
  `material`, `shape`, and `weather`.

### Phase E: Release polish

- Update README examples and CLI help.
- Document that these are phrase-shape wordlists, not security claims.
- Add migration notes explaining that future list changes require new names or
  versions.
- Consider a `nwords presets --semantic` grouping if the preset list grows.

## Acceptance gates

Before any list ships:

- Provenance README exists.
- License status is recorded.
- Direct-source license file is copied when required.
- Raw source snapshot exists for direct-derived lists.
- Curated output file exists.
- Local blocklist exists when filtering removed source words.
- `tests/vectors/SHA256SUMS` covers all source, blocklist, curated, and license
  files.
- Rust array order matches the curated snapshot.
- Boundary test pins `len()`, first word, and last word.
- Parser test covers canonical name and aliases.
- `index_of(word(i)) == i` for every word.
- No duplicate words inside the list.
- Role metadata test covers all named lists.
- CLI preset tests cover capacity reporting and round-trip behavior for shipped
  presets.

Before any preset ships:

- The shape uses a documented role sequence.
- Exact capacity is reported as capacity, not entropy.
- Boundary encode/decode IDs round-trip for the configured range.
- Slack states are rejected if the configured range is smaller than capacity.
- Phrase samples have been reviewed for bad adjacent combinations.

## Open decisions

1. Decide whether `descriptor` is distinct enough from `adjective` to freeze as
   a public name.
2. Decide whether authored lists should accept a small number of high-quality
   non-seed manual additions during review. If yes, document that the shipped
   list is authored and the seed references are non-authoritative.
3. Decide whether role metadata should ever reject custom shapes, or only power
   docs, presets, and warnings.
4. Decide the final `food` source mix: USDA FoodData Central only, or USDA plus
   FoodOn as a CC BY reference.
5. Decide whether `plant` should include only common plant words or also a small
   number of crop/food overlap terms. Either choice is valid, but it must be
   documented before freezing the list.
