# ID-to-words CLI plan

Planning consensus from Codex/Claude Parley for a small `nwords` CLI.

Transcript:
`/tmp/parley/682705f93766/runs/workflow-architecture-plan-753f44d3/transcript.md`.

## Purpose

The CLI should make the positional ID codec easy to use from a shell:

- encode an integer ID into a fixed-width word phrase;
- decode a fixed-width word phrase back into an integer ID;
- offer sane presets for common ID ranges;
- expose planning data when requested.

This CLI is not a BIP-39 wallet-mnemonic generator. When it uses the BIP-39
English wordlist, it uses that wordlist as a positional dictionary only. It
does not add BIP-39 checksum bits, validate BIP-39 mnemonic entropy, or derive
BIP-39 wallet seeds.

Every help string, preset listing, and README example that mentions BIP-39 in
this CLI must include this distinction:

```text
BIP-39 wordlist used as a positional dictionary, not a BIP-39 mnemonic.
```

## Crate Shape

Add a workspace crate:

```text
crates/nwords-cli
```

Package name:

```text
nwords-cli
```

Binary name:

```text
nwords
```

The CLI crate may require `std`. It should depend on the public `nwords`
umbrella crate rather than reaching into lower-level crates unless a specific
gap appears.

Recommended initial dependency stance:

- Phase 1: no CLI parser dependency; hand-roll the tiny command surface.
- Phase 3: revisit a parser dependency when the full flag surface exists.
- If a dependency becomes useful, prefer a tiny parser such as `lexopt` before
  considering `clap`.

No CLI dependency should pull complexity into the library crates.

## Semantics

The CLI wraps `nwords::positional::Positional`.

For a uniform shape:

```text
capacity = dictionary_size ^ word_count
accepted ID range = [0, range)
slack = capacity - range
acceptance ratio = range / capacity
```

Encoding rejects IDs outside `[0, range)`. Decoding rejects phrase states that
map into slack. These are positional-codec errors, not BIP-39 mnemonic errors.

Exact CLI planning remains bounded by the existing `u128` range model. Do not
ship a `u128` preset for now because a full `[0, 2^128)` domain cannot be
represented as an exclusive `u128` upper bound.

For ordered named-list shapes such as `adjective,animal`, the CLI wraps
`nwords::positional::MixedPositional` over
`nwords::wordlists::named::WordListSequence`. These shapes have intrinsic
position counts; users provide `--shape` and omit `--words`. If `--range` is
omitted for explicit custom `--shape`, `--words`, or legacy `--dict` inputs,
the CLI uses the exact full phrase capacity as the accepted ID range. If that
capacity is `BeyondU128`, encode/decode reject omitted `--range` and require an
explicit finite accepted range. Providing `--range` narrows the accepted domain
and keeps slack rejection semantics. The older `--dict adjective-animal` form
remains a compatibility alias.

Preset names keep their configured ranges. For example, `--preset dec6` means
range `[0, 1_000_000)`, not the full capacity of its underlying two-word
BIP-39 positional shape.

Do not describe preset capacity as security entropy unless the input IDs are
uniformly sampled from the stated range.

## Phase 1: Minimal Round Trip

Goal: a small useful binary with one preset.

Commands:

```sh
nwords encode 42 --preset u32
nwords decode <words...> --preset u32
```

Phase-1 preset:

| Preset | Range | Dictionary | Words | Capacity | Slack | Acceptance |
|---|---:|---|---:|---:|---:|---:|
| `u32` | `2^32` | `bip39-en,bip39-en,bip39-en` | 3 | `8_589_934_592` | `4_294_967_296` | `1/2` |

Legacy dictionary identifier:

```text
bip39-en-positional
```

This name is intentionally explicit. It avoids implying that the output is a
BIP-39 mnemonic.

Output behavior:

- `encode` writes only the phrase to stdout by default.
- `decode` writes only the integer ID to stdout by default.
- Errors go to stderr and return a non-zero exit code.
- Errors must not echo raw input phrases or unknown words.

Suggested exit codes:

| Code | Meaning |
|---:|---|
| 0 | success |
| 1 | encode/decode failure, such as out-of-range ID or unknown word |
| 2 | usage/configuration error, such as unknown preset or missing argument |

Phase-1 stop condition:

- `encode` and `decode` round-trip through the `u32` preset.
- Invalid IDs fail.
- Invalid word counts fail.
- Unknown words fail without leaking the raw word in the error.
- `cargo test -p nwords-cli`.

Defer from Phase 1:

- multiple presets;
- `presets` command;
- `plan` command;
- `--explain`;
- alternate dictionaries;
- custom dictionary files;
- parser dependency decision.

## Phase 2: Presets And Planning

Add:

```sh
nwords presets
nwords plan --preset u32
nwords plan --range <range>
nwords plan --shape <lists>
nwords help <command>
```

Recommended presets, all using repeated `bip39-en` shapes:

| Preset | Range | Words | Capacity | Slack | Acceptance |
|---|---:|---:|---:|---:|---:|
| `dec6` | `1_000_000` | 2 | `4_194_304` | `3_194_304` | `1_000_000 / 4_194_304` |
| `dec9` | `1_000_000_000` | 3 | `8_589_934_592` | `7_589_934_592` | `1_000_000_000 / 8_589_934_592` |
| `u32` | `4_294_967_296` | 3 | `8_589_934_592` | `4_294_967_296` | `1/2` |
| `u64` | `18_446_744_073_709_551_616` | 6 | `73_786_976_294_838_206_464` | `55_340_232_221_128_654_848` | `1/4` |
| `dec18` | `1_000_000_000_000_000_000` | 6 | `73_786_976_294_838_206_464` | `72_786_976_294_838_206_464` | `1_000_000_000_000_000_000 / 73_786_976_294_838_206_464` |

`presets` should print:

- preset name;
- shape;
- word count;
- accepted range;
- capacity;
- slack;
- acceptance ratio.

`plan --preset <name>` should report the same precise terms for one preset.
`plan --range <range>` should use `nwords::stats` to choose the required word
count for the default BIP-39 English list size. If `--words <N>` is supplied, it
should report whether that word count can represent the range. It should use
precise terms: capacity, range, slack, and acceptance ratio.
`plan --shape <lists>` should report shape-only stats without requiring a
range: ordered list names, per-position list sizes, word count, and total
capacity.
`help <command>` should print detailed command help with usage, options,
example commands, and one-line descriptions for those examples.

README updates belong in this phase, including the BIP-39-positional caveat.

## Phase 3: Custom Options

Add:

```sh
nwords encode 123 --range 1000000 --words 2
nwords decode <words...> --range 1000000 --words 2
nwords encode 4194303 --words 2
nwords decode "zoo zoo" --words 2
nwords encode 123 --preset dec6 --explain
nwords encode 123 --range 100000 --shape adjective,animal
nwords encode 249416 --shape adjective,animal
nwords encode 1337 --range 1e6 --shape color,adjective,animal
```

Custom flags:

| Flag | Meaning |
|---|---|
| `--range <R>` | optional accepted exclusive ID range `[0, R)`; accepts decimal digits or exact scientific shorthand such as `1e6`; omit with explicit custom shapes to use exact full capacity |
| `--words <N>` | fixed output word count |
| `--shape <lists>` | comma-separated ordered named word lists |
| `--dict <name>` | legacy named dictionary alias |
| `--preset <name>` | named range/shape/permutation bundle |
| `--explain` | print planning data along with encode/decode output |

Initial shape/list choices:

| Name | Meaning |
|---|---|
| `bip39-en` | BIP-39 English wordlist used as a positional dictionary |
| `adjective` | Curated adjective list |
| `animal` | Curated animal list |
| `color` | Upstream color list |
| `adjective,animal` | Curated two-position adjective-animal shape |

Potential later choices:

- `bip39-ja-positional`, only after separator/display behavior is settled;
- user-provided word-list files, only after the built-in CLI surface is stable.

Additive adjective-animal presets:

| Preset | Range | Dictionary | Words | Capacity | Slack |
|---|---:|---|---:|---:|---:|
| `aa` | `249_417` | `adjective,animal` | 2 | `249_417` | 0 |
| `dec5-aa` | `100_000` | `adjective,animal` | 2 | `249_417` | `149_417` |
| `color-aa` | `12_969_684` | `color,adjective,animal` | 3 | `12_969_684` | 0 |

Preset definitions are kept as Rust constants in V1 using fields equivalent to
`name`, `range`, `shape`, and `permutation`. A separate YAML copy is deferred
because it would drift unless it drives code generation, and a YAML parser
dependency is not justified for the built-in table.

Parser dependency decision:

- If Phase 3 flags remain simple, keep manual parsing.
- If help, unknown flag diagnostics, and option parsing become noisy, use a
  tiny parser such as `lexopt`.
- Avoid `clap` unless the CLI grows enough that its dependency surface is
  clearly justified.

## Output Formats

Default output stays script-friendly:

```text
<phrase>
```

or:

```text
<id>
```

`--explain` may use a stable text block:

```text
phrase: abandon ability ...
preset: u32
shape: bip39-en,bip39-en,bip39-en
words: 3
range: 4294967296
capacity: 8589934592
slack: 4294967296
acceptance_ratio: 4294967296/8589934592
```

JSON output is deferred. If added later, it should be explicit with `--json`,
not the default.

## Test Plan

Use standard library integration tests around the compiled binary. Avoid test
helper dependencies unless they remove real friction.

Required Phase-1 tests:

- `encode 0 --preset u32` succeeds.
- `encode 42 --preset u32` then `decode ... --preset u32` returns `42`.
- `encode 4294967295 --preset u32` succeeds.
- `encode 4294967296 --preset u32` fails.
- decoding the first slack state for `u32` fails.
- unknown words fail without echoing the raw unknown word.
- missing command, missing ID, unknown preset, and unknown command return usage
  failures.

Required Phase-2 tests:

- `presets` lists every preset and the BIP-39-positional caveat.
- each preset round-trips boundary IDs `0` and `range - 1`.
- `plan --range` returns the same word count as `nwords::stats::required_words`.
- `plan --range --words` reports range/capacity/slack accurately.
- `--range` accepts exact shorthand such as `1e6` and rejects non-integer
  shorthand expansions.
- `help <command>` and nested help such as `help bytes encode` print detailed
  usage examples.

Required Phase-3 tests:

- custom `--range` and `--words` round-trip.
- custom encode/decode without `--range` uses exact full capacity when capacity
  fits in `u128`.
- custom encode/decode without `--range` rejects `BeyondU128` shapes with a
  clear message requiring `--range`.
- `--explain` reports capacity, range, slack, and acceptance ratio.
- alternate shape/list names either work or fail with a usage error.

## Deferred Questions

- Full `u128` domain support. This needs a representation for exclusive
  `2^128` or a separate inclusive-domain path.
- Non-BIP-39 dictionaries.
- User-provided word-list files.
- JSON output.
- Shell completions.
- Security recommendation policy such as "choose at least N bits".

## Implementation Notes

The CLI should be implemented in small commits:

1. Add `nwords-cli` skeleton and `u32` encode/decode.
2. Add preset table and planning output.
3. Add custom flags and `--explain`.
4. Update README and agent guidance after the user-facing CLI is stable.

Before each implementation commit, run the local gates relevant to the changed
surface and get Claude review if continuing the commit-gated workflow.
