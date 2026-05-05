---
name: nwords-architecture
description: >
  Use when planning or modifying nwords APIs, codecs, wordlists, BIP-39
  behavior, positional ID encodings, dictionary size choices, word-count
  choices, or stats/helper calculations for capacity and entropy.
---

# nwords Architecture Skill

Use this skill before making architecture decisions for the `nwords` library or
before recommending a dictionary size, phrase length, or ID range.

## Start Here

Read these files first:

- `docs/research/architecture-decisions.md` for settled V1 contracts.
- `docs/research/execution-plan.md` for phase boundaries.
- `docs/research/stats-helper-notes.md` for capacity and entropy planning.
- `docs/research/test-plan.md` for required vector and property tests.
- `docs/code_standards.md` for implementation rules.

If these files conflict, prefer `architecture-decisions.md` for API contracts
and `execution-plan.md` for sequencing.

## Decision Flow

1. Identify the scheme.
   - BIP-39: use BIP-39 spec-fixed word counts.
   - Positional N-word: use stats helper calculations.
   - Niceware, Proquint, PGP, SLIP-39: post-V1 unless the plan explicitly says
     otherwise.
2. State which inputs are fixed:
   - ID range or desired decimal digit count.
   - Dictionary size.
   - Word count.
   - Mixed per-position dictionary sizes, if applicable.
3. Use `nwords::stats` once implemented. Before implementation, use the formulas
   in `docs/research/stats-helper-notes.md`.
4. Record outputs using precise terms:
   - capacity;
   - ID range;
   - slack;
   - acceptance ratio;
   - uniform-sample entropy bits.
5. Confirm whether the result is exact `u128` math or a log-domain estimate.

## Required Terminology

- Capacity is how many phrase states a shape can represent.
- ID range is the accepted domain, usually `[0, range)`.
- Slack is capacity minus accepted range.
- Entropy bits are valid only when IDs are uniformly sampled from the stated
  range.

Do not call sequential, assigned, user-chosen, or biased IDs "secure" because
the phrase shape has a large capacity.

## BIP-39 Rules

BIP-39 does not use free-form dictionary-size planning.

| Words | Entropy bits | Checksum bits |
|---:|---:|---:|
| 12 | 128 | 4 |
| 15 | 160 | 5 |
| 18 | 192 | 6 |
| 21 | 224 | 7 |
| 24 | 256 | 8 |

Reject non-spec BIP-39 word counts such as 13, 14, or 25. Japanese display uses
U+3000 by default; ASCII-space display is a separate rust-bitcoin parity mode.

## Positional Planning Rules

For a uniform dictionary:

```text
capacity = dictionary_size ^ word_count
required_words = ceil_log_base(range, dictionary_size)
required_dictionary_size = ceil_root(range, word_count)
slack = capacity - range
acceptance_ratio = range / capacity
```

For a mixed positional dictionary:

```text
capacity = product(dictionary_size_at_position)
```

If multiple variables are unspecified, return a bounded candidate table instead
of pretending there is one correct answer.

## V1 Boundaries

- Exact state counts stop at `u128`.
- Values above `u128::MAX` use `CapacityClass::BeyondU128`.
- BigInt exact math, CLI reports, security recommendation policy, non-identity
  permutations, `phf`, SLIP-39, Niceware, Proquint, and PGP word lists are
  post-V1 unless the user explicitly changes scope.
- Keep the stats kernel small and dependency-light.

## Before Finishing A Change

- Update `architecture-decisions.md` if a settled contract changes.
- Update `execution-plan.md` if phase order or scope changes.
- Update `test-plan.md` when adding or changing behavior.
- Check that docs do not blur capacity with entropy.
