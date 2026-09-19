# Semantic wordlists execution plan

Execution plan for the semantic wordlist roadmap in
[`semantic-wordlists-roadmap.md`](semantic-wordlists-roadmap.md). This plan
turns the roadmap into implementation phases, acceptance criteria, and upfront
architecture decisions for:

- new built-in semantic word lists;
- grammar-role metadata for phrase-shape presets;
- curation tooling and provenance checks;
- user-defined wordlists in the CLI and library.

This is a planning document. Shipped wordlist contents, order, canonical names,
aliases, and preset shapes remain compatibility contracts once released.

## Goals

1. Add phrase-friendly built-in lists for `object`, `descriptor`, `mood`,
   `material`, `shape`, `weather`, `plant`, and `food`.
2. Keep all built-in list provenance license-clean and reviewable.
3. Preserve the existing `WordMap` and `MixedPositional` architecture.
4. Provide a clear path for user-defined wordlists without adding a new core
   trait.
5. Keep capacity, range, slack, and entropy terminology precise.

## Non-goals

- Do not add a new core single-position wordlist trait.
- Do not require powers-of-two list sizes.
- Do not add runtime dependencies for curation or code generation.
- Do not make phrase grammar a hard codec constraint.
- Do not treat user-defined wordlist fingerprints as security hashes.
- Do not silently normalize, sort, dedupe, stem, or repair user-provided word
  files.

## Architecture Decisions

### Built-In Lists

Built-in semantic lists extend `nwords_wordlists::named::NamedWordList` and the
existing `WordListSequence<'a>` adapter.

Rules:

- Add one enum variant per shipped built-in list.
- Store each list in a generated/static Rust array module.
- Preserve upstream order for direct-derived lists.
- Use frozen alphabetical order for authored curated lists.
- Keep `NamedWordList::index_of` linear unless a list has an explicitly
  documented sorted-order invariant. Upstream-preserved order must not be
  binary-searched just because it happens to look sorted.
- Keep `WordListSequence<'a>` as the borrowed/static adapter used by builtin
  presets.

The current `WordMap` trait already supports owned word storage because
`word(&self, index, position) -> Option<&str>` returns a borrow from `self`, not
`&'static str`. That is the key reason user-defined wordlists do not require a
new trait.

### Role Metadata

Add advisory role metadata in `nwords_wordlists::named`, not in `nwords-core`:

```rust
pub enum WordListRole {
    Modifier,
    Head,
    Either,
}
```

Add:

```rust
impl NamedWordList {
    pub const fn role(self) -> WordListRole;
}
```

Roles are used for documentation, preset design, phrase sampling, and optional
CLI warnings. They must not affect encoding or decoding. Position remains the
only decoding context.

Recommended built-in roles:

| List | Role |
|---|---|
| `adjective` | `Modifier` |
| `color` | `Modifier` |
| `descriptor` | `Modifier` |
| `mood` | `Modifier` |
| `weather` | `Modifier` |
| `animal` | `Head` |
| `object` | `Head` |
| `plant` | `Head` |
| `food` | `Head` |
| `material` | `Either` |
| `shape` | `Either` |
| `bip39-en` | `Either` |

### User-Defined Wordlists

User-defined wordlists use an owned adapter that implements `WordMap`. This
adapter should live in `nwords_wordlists::named` or a `named::dynamic` child
module behind `alloc`.

The public API should be thin and typed:

```rust
pub struct OwnedWordList {
    name: String,
    role: WordListRole,
    words: Vec<String>,
}

pub struct DynamicWordListSequence {
    // Internal representation, not necessarily public fields:
    owned: Vec<OwnedWordList>,
    positions: Vec<DynamicWordListSlot>,
}

pub enum DynamicWordListSlot {
    Builtin(NamedWordList),
    Owned(usize),
}

pub enum WordListError {
    // Exact variants decided during implementation.
}
```

`DynamicWordListSequence` should support:

- pure built-in shapes;
- pure user-defined shapes;
- mixed built-in/user-defined shapes;
- repeated user-defined lists without cloning list contents where practical.

Presets may remain static `&'static [NamedWordList]`. At runtime, the CLI should
lower presets, built-in `--shape` values, and mixed user-defined `--shape`
values into a single `DynamicWordListSequence` path before constructing
`MixedPositional`. This avoids a long-term split between builtin-only reporting
and dynamic reporting.

Implementation may start with a simpler clone-on-repeat representation only if
the public constructor surface does not expose that choice. The preferred
internal shape is an owned pool plus position slots.

### Capacity Calculation

Do not add separate capacity logic to each sequence type. Capacity should be
computed from per-position lengths:

```text
capacity = product(word_map.len(position) for position in 0..word_count)
```

Implementation can collect position lengths and call the existing mixed
capacity helper. The report path should work for both `WordListSequence` and
`DynamicWordListSequence`.

### User File Format

CLI support should use this surface:

```sh
nwords encode 42 --range 1000 --list project=./project.txt --shape color,project
nwords plan --list project=./project.txt --shape project,animal
```

Rules:

- `--list name=path` is repeatable.
- `name` must be lowercase ASCII and match `[a-z][a-z0-9_-]*`.
- `name` must not collide with a built-in canonical name or alias. The
  implementation should use `NamedWordList::parse(name).is_some()` for this
  check.
- Duplicate `--list` names in one invocation are rejected.
- Every provided user-list name must be referenced by `--shape`; unreferenced
  entries are rejected to catch typos.
- Loaded user names can be used in `--shape` alongside built-ins.
- Built-in names take precedence only by rejecting collisions; no shadowing.
- User-defined files are runtime inputs, not vendored vectors, and are not
  covered by `tests/vectors/SHA256SUMS`.

File parsing:

1. Reject files that are not valid UTF-8.
2. Ignore blank lines.
3. Ignore full-line comments where the first non-whitespace character is `#`.
4. Trim outer ASCII whitespace on content lines.
5. Validate the remaining token exactly.
6. Reject invalid tokens.
7. Reject duplicate accepted tokens.
8. Reject fewer than two accepted words.
9. Preserve accepted-token order as the wordlist order.

Token validation:

- lowercase ASCII only;
- single token only;
- no uppercase;
- no digits;
- no hyphen, apostrophe, punctuation, or internal whitespace;
- no silent lowercasing, sorting, deduping, stemming, or normalization.

Validation guards:

| Limit | Initial value |
|---|---:|
| Max `--list` flags per invocation | 32 |
| Max file size | 1 MiB |
| Max line length | 128 bytes |
| Max accepted entries per list | 4,096 |
| Min accepted entries per list | 2 |

These limits are acceptance guards, not decode contracts. Loosening them later
is compatible. Tightening them may reject previously valid user files and should
be treated carefully.

User-list errors should include line numbers. Duplicate errors should include
the current line and first-seen line. Multi-error reporting is preferred;
first-error reporting is acceptable for the first implementation if documented.

### User File Fingerprints

For user-defined lists, `plan` and `--explain` should report a dependency-free
drift fingerprint for each loaded list:

```text
user_list_0_name: project
user_list_0_words: 128
user_list_0_fingerprint: fnv1a64:0123456789abcdef
```

Fingerprint rules:

- compute over the accepted-token vector, not the raw file bytes;
- feed each accepted token's bytes followed by a NUL separator;
- comments and blank lines do not affect the fingerprint;
- invalid files have no capacity and no fingerprint;
- use a dependency-free FNV-1a 64-bit implementation;
- label it as a drift/reproducibility aid, not a security hash.

The CLI docs should tell users to pin and share their wordlist files when they
need reproducible decoding.

## Vector And Source Layout

Use source posture to decide directory structure.

### Direct-Derived Sources

Group direct-derived snapshots by upstream project:

```text
tests/vectors/friendly-words/
  README.md
  glitch-friendly-words-objects.txt
  glitch-friendly-words-predicates.txt
  object-blocklist.txt
  descriptor-blocklist.txt
  nwords-objects.txt
  nwords-descriptors.txt
```

License files belong under:

```text
tests/vectors/licenses/
```

Direct-derived READMEs must record:

- source project;
- URL;
- commit/tag or retrieval date;
- license;
- raw snapshot names;
- local transform policy;
- blocklist policy;
- ordering rule.

### Authored Curated Lists

Group authored lists under:

```text
tests/vectors/semantic-wordlists/<list>/
  README.md
  nwords-<list>.txt
  <list>-blocklist.txt        # optional
```

Authored READMEs must state that the list is authored in-repo and that seed
references are non-authoritative. Do not vendor CC BY sources as if they were
direct sources unless the attribution and derivative-work posture is explicitly
accepted for that list.

### Hash Coverage

`tests/vectors/SHA256SUMS` must cover:

- direct raw snapshots;
- blocklists;
- curated snapshots;
- README provenance files;
- copied license files.

For authored lists without raw source snapshots, hash the curated snapshot,
README, and blocklist files.

## Curation Harness

Add a dev-only curation harness before adding new lists. It may be an `xtask`
crate or a checked-in tool under `tools/`, but it must not add runtime
dependencies to library crates.

Required harness capabilities:

- extract or read raw source snapshots;
- normalize candidates according to list policy;
- apply length rules and blocklists;
- validate lowercase ASCII single-token entries;
- detect duplicates;
- verify required order rule;
- emit one-word-per-line curated snapshots;
- emit Rust array modules;
- compute SHA256 entries or verify `SHA256SUMS`;
- sample adjacent phrase pairs/triples for human review.

The harness is a guard, not a truth engine. It can check syntax, duplicates,
ordering, source hashes, and phrase samples. It cannot prove a word is friendly
or appropriate; human review remains required.

The first acceptance gate for the harness is reproducing the already-shipped
`adjective`, `animal`, and `color` outputs from existing vectors. This gives the
harness a real regression oracle.

## Execution Phases

### Phase 0: Contract Lock

Purpose: settle architecture before adding more lists.

Changes:

- Add or document `WordListRole`.
- Add `NamedWordList::role()`.
- Decide and document the public shape of `OwnedWordList`,
  `DynamicWordListSequence`, and `WordListError`.
- Document the user-defined file format and CLI surface.
- Document source directory conventions.
- Resolve whether `descriptor` is distinct enough from `adjective`.
- Decide final canonical names and aliases for all eight planned lists.

Acceptance criteria:

- Role metadata exists or the implementation-ready signature is documented.
- Every existing `NamedWordList` has a role.
- The generic/dynamic adapter design is documented without adding a new core
  trait.
- User-defined file validation rules are documented.
- Built-in name collision behavior is documented.
- `descriptor` naming is either approved or replaced with a better name.

Required tests if code changes in this phase:

- role table covers every built-in variant;
- parser alias tests cover canonical names and aliases;
- `cargo test -p nwords-wordlists --all-features`;
- `cargo check -p nwords --no-default-features --features alloc`.

### Phase 1: Curation Harness

Purpose: build repeatable list-generation and validation tooling without
shipping new wordlists.

Changes:

- Add dev-only curation harness.
- Encode direct-derived and authored-list validation policies.
- Add phrase sampler.
- Add a harness command that reproduces existing adjective/animal/color files.
- Add documentation for running the harness and reviewing samples.

Acceptance criteria:

- Harness reproduces existing `nwords-adjectives.txt`,
  `nwords-animals.txt`, and `unique-names-generator-colors.txt`.
- Harness rejects duplicate, non-ASCII, uppercase, multi-token, and invalid
  entries in fixture tests.
- Harness can validate upstream-order and alphabetical-order policies.
- Harness can emit deterministic Rust arrays.
- Harness output is stable across repeated runs.
- No runtime library dependency is added for harness-only behavior.

Required tests:

- harness unit tests for validation;
- fixture tests for invalid inputs;
- SHA256 verification;
- `cargo fmt --all -- --check`;
- `cargo test --workspace --all-features`.

### Phase 2: Direct-Derived Glitch Lists

Purpose: add the high-confidence direct-derived semantic lists.

Lists:

- `object`
- `descriptor`

Changes:

- Vendor Glitch `friendly-words` raw snapshots at a pinned commit.
- Copy the MIT license under `tests/vectors/licenses/`.
- Add blocklists and curated snapshots.
- Generate Rust arrays.
- Add `NamedWordList` variants, names, aliases, roles, lengths, word lookup, and
  index lookup.
- Add phrase-shape presets only after phrase samples pass review.

Recommended initial presets:

- `descriptor,object`
- `color,descriptor,object`

Acceptance criteria:

- Source README records project, URL, commit, license, transforms, blocklists,
  and ordering rule.
- Curated lists preserve upstream order for retained words.
- Boundary tests pin length, first word, and last word.
- `index_of(word(i)) == i` for every entry.
- No duplicates inside either list.
- Parser accepts canonical names and approved aliases.
- Role metadata is correct.
- SHA256SUMS covers all source, blocklist, curated, README, and license files.
- Presets report capacity and round-trip boundary IDs.
- Phrase samples have been human-reviewed.

Required tests:

- `cargo test -p nwords-wordlists --all-features`;
- CLI `plan --shape` tests for the new shapes;
- CLI encode/decode round-trip tests for preset ranges;
- `sha256sum -c tests/vectors/SHA256SUMS`;
- `cargo check -p nwords --no-default-features --features alloc`.

### Phase 3: Small Authored Semantic Lists

Purpose: add bounded authored lists with low source/license risk.

Lists:

- `mood`
- `material`
- `shape`
- `weather`

Changes:

- Create authored curated snapshots.
- Record seed references as non-authoritative.
- Freeze alphabetical order.
- Generate Rust arrays.
- Add variants, aliases, roles, tests, and selected presets.

Recommended preset candidates:

- `mood,descriptor,object`
- `material,shape,object`
- `mood,adjective,animal`

Acceptance criteria:

- Each list has an authored-list README.
- Seed references and exclusions are documented.
- Alphabetical order is enforced and documented.
- Boundary and full round-trip lookup tests pass.
- Category-specific hazards are reviewed.
- Phrase samples are reviewed for every shipped preset.
- No preset describes capacity as entropy.

Required tests:

- per-list boundary tests;
- duplicate and ordering tests;
- parser alias tests;
- role coverage tests;
- CLI plan/preset tests for shipped shapes;
- full workspace tests.

### Phase 4: Plant And Food

Purpose: add larger curated head-noun lists after source and hazard review.

Lists:

- `plant`
- `food`

Changes:

- Confirm source/license posture before using any source snapshot.
- Use CC0/public-domain sources as candidate seeds where possible.
- Keep CC BY sources reference-only unless attribution posture is explicitly
  accepted.
- Curate to common, friendly, single-token head nouns. Keep `plant`
  ornamental/garden-focused and disjoint from `food`.
- Generate arrays, variants, tests, and only the presets that pass phrase
  sampling.

Recommended preset candidates:

- `descriptor,plant`
- `weather,descriptor,plant`
- `mood,descriptor,food`
- `material,shape,food`, only if phrase review passes.

Acceptance criteria:

- Final source mix is documented for both lists.
- `food` excludes brands, alcohol/drugs, medical/allergen framing, dominant
  non-food homographs, and very obscure regional items.
- `plant` excludes Latin binomials, cultivars, drug-associated plants, notably
  toxic defaults, and obscure taxonomy.
- `plant` and `food` are disjoint exact word sets.
- Curated files are frozen in documented order.
- Phrase samples pass review before presets ship.
- All list and preset tests from earlier phases apply.

Required tests:

- per-list validation tests;
- boundary and full lookup tests;
- SHA256 verification;
- CLI preset and custom-shape tests;
- full workspace tests.

### Phase 5: User-Defined Wordlists

Purpose: let users combine runtime wordlist files with built-in lists while
preserving codec architecture.

Changes:

- Implement `OwnedWordList`, `DynamicWordListSequence`, and `WordListError`.
- Implement `WordMap` for `DynamicWordListSequence`.
- Add CLI `--list name=path`, repeatable.
- Resolve all CLI shapes to `DynamicWordListSequence`.
- Support mixed built-in and user-defined shapes.
- Report user-list length and drift fingerprint in `plan` and `--explain`.
- Document reproducibility requirements for user-provided files.

Acceptance criteria:

- No new core trait is added.
- `nwords-core` remains dependency-free.
- Dynamic sequence supports pure built-in, pure user-defined, and mixed shapes.
- User-defined parsing rejects invalid UTF-8, bad names, name collisions,
  duplicate list names, invalid tokens, duplicate words, too few entries, and
  excessive file/line/list sizes.
- Blank lines and full-line comments are ignored.
- File order is preserved.
- Fingerprint is computed over accepted tokens and labeled as non-security.
- CLI reports owned-list metadata in `plan` and `--explain`.
- Presets are never mislabeled when a user-defined list participates in a shape.

Required tests:

- library tests for owned list validation;
- library tests for dynamic `WordMap` dispatch;
- mixed built-in/user-defined encode/decode round trips;
- duplicate-word errors include line numbers;
- name collision tests using `NamedWordList::parse`;
- comment/blank-line parsing tests;
- invalid UTF-8 and size-limit tests;
- fingerprint stability tests;
- CLI plan/explain tests with user-defined lists;
- `cargo check -p nwords --no-default-features --features alloc`.

### Phase 6: Release Polish

Purpose: make the expanded surface understandable and maintainable.

Changes:

- Update README examples.
- Update CLI help.
- Add semantic preset grouping if the preset table becomes noisy.
- Document list-versioning rules.
- Document user-file reproducibility and drift fingerprints.
- Add migration notes explaining that shipped list changes require new names or
  versions.

Acceptance criteria:

- README includes at least one built-in semantic shape example.
- README includes at least one user-defined list example once Phase 5 ships.
- CLI help distinguishes built-in named shapes from user-provided lists.
- Docs never describe representational capacity as entropy without the
  uniform-sampling precondition.
- The full V1 local gate passes.

Required final gate:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo check -p nwords --no-default-features --features alloc
sha256sum -c tests/vectors/SHA256SUMS
```

## Cross-Phase Acceptance Rules

Every shipped built-in list requires:

- provenance README;
- license status;
- copied license file when vendoring a direct source;
- curated one-word-per-line snapshot;
- source snapshot for direct-derived lists;
- blocklist file when filtering removes words;
- SHA256SUMS coverage;
- Rust array matching the curated snapshot;
- stable length/first/last boundary test;
- full `index_of(word(i)) == i` test;
- duplicate-free test;
- parser canonical-name and alias test;
- role coverage test.

Every shipped preset requires:

- documented role sequence;
- exact capacity report;
- boundary ID encode/decode tests;
- slack rejection test when range is smaller than capacity;
- phrase sample review record.

Every user-defined wordlist feature requires:

- strict validation with no silent mutation;
- clear error messages without leaking full phrases in decode errors;
- line-numbered load errors;
- drift fingerprint reporting;
- mixed built-in/user-defined round-trip tests.

## Open Decisions Assigned To Phases

| Decision | Phase | Recommendation |
|---|---:|---|
| Is `descriptor` distinct enough from `adjective`? | 0 | Keep only if documented as Glitch-predicate-derived, possibly participial descriptors. |
| Exact public constructor names for dynamic lists | 0 | Implemented as `OwnedWordList`, `DynamicWordListSequence`, `DynamicWordListSlot`, and `WordListError`. |
| Whether CLI role hints for user lists ship | 5 or later | Defer; default user lists to `Either`. |
| Whether dynamic lookup gets an index map | 5 or later | Start linear; add per-list lookup map only if benchmarks justify it. |
| Final `food` source mix | 4 | Prefer USDA CC0 as seed; keep FoodOn reference-only unless CC BY posture is accepted. |
| Final `plant` scope | 4 | Common friendly ornamental plants, flowers, trees, houseplants, and garden or landscape nouns; disjoint from `food`. |
| Semantic preset grouping in CLI | 6 | Add only if preset output becomes hard to scan. |

## Suggested Commit Slices

1. Contract metadata: role enum, `role()`, tests, docs.
2. Harness foundation: validation and reproduction of existing lists.
3. Friendly-words source snapshots and `object`/`descriptor` arrays.
4. Built-in variants and direct-derived presets.
5. Authored small semantic lists.
6. Plant/food curation.
7. Dynamic user-list adapter.
8. CLI `--list` support and fingerprint reporting.
9. README/help/release polish.

Each slice should pass the relevant phase gate before the next slice starts.
