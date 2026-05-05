# Stats helper notes

Planning notes for the `nwords` capacity, entropy, and dictionary-size helper.
This follows the Claude/Codex Parley consensus from May 2026.

## Purpose

The helper should answer design questions before a user commits to a dictionary
shape or phrase length:

- How many words are needed for a target ID range and dictionary size?
- What dictionary size is needed for a target ID range and word count?
- What range can be represented by a fixed dictionary size and word count?
- What tradeoffs are available when multiple inputs are not fixed?
- What BIP-39 word counts are valid for a desired entropy target?

The helper is part of the library's value, not just example code, because these
calculations drive safe use of positional codecs.

## Terminology

Use these terms consistently:

- **Capacity**: number of phrase states a shape can represent.
  - Uniform positional shape: `dictionary_size ^ word_count`.
  - Mixed positional shape: product of `len(position)` for each position.
- **ID range**: accepted application domain, usually `[0, range)`.
- **Slack**: capacity minus range. Slack states are rejected by the codec.
- **Acceptance ratio**: `range / capacity`.
- **Uniform-sample entropy bits**: `log2(range)` only when IDs are sampled
  uniformly from the range.

Do not describe sequential, assigned, user-chosen, or otherwise biased IDs as
having security entropy. They may have representational capacity.

## BIP-39 Is A Separate Case

BIP-39 uses a fixed 2048-word dictionary and fixed word counts. The mnemonic
length includes checksum bits, so BIP-39 entropy is not `word_count * 11`.

| Words | Entropy bits | Checksum bits | Total bits |
|---:|---:|---:|---:|
| 12 | 128 | 4 | 132 |
| 15 | 160 | 5 | 165 |
| 18 | 192 | 6 | 198 |
| 21 | 224 | 7 | 231 |
| 24 | 256 | 8 | 264 |

The BIP-39 helper should refuse non-spec word counts rather than rounding them.
It can offer "minimum legal BIP-39 word count for desired entropy bits" by
choosing from the table above.
The V1 API exposes this as `nwords::stats::bip39::for_word_count` and
`nwords::stats::bip39::minimum_word_count_for_entropy_bits` through the
umbrella crate when both `stats` and `bip39` are enabled.

## Placement

Use a staged split:

1. `nwords-core::stats` private or `pub(crate)` exact kernel.
2. Public planner surface in `nwords-core::stats` once the kernel is stable.
3. BIP-39 adapter behind the `bip39` feature, outside the pure core.
4. Umbrella re-export under `nwords::stats`, with optional `std` advisory
   formatting.

This keeps pure integer planning available to `no_std + alloc` users while
preventing BIP-39 spec rules or advisory formatting from leaking into the
lowest-level math.

## Exact Boundary

V1 exact state counts use `u128`. Anything above `u128::MAX` is represented as
a log-domain estimate or bound, not by a big-integer dependency.

Suggested shape:

```rust
pub struct Log2Estimate {
    pub lower_bits: u32,
    pub upper_bits: u32,
}

pub enum CapacityClass {
    Exact(u128),
    BeyondU128 { log2: Log2Estimate },
}
```

For exact values, derive bit estimates from the exact count. For beyond-`u128`
values, keep the exact kernel conservative and let the `std` advisory layer add
friendlier approximate formatting later.

## Minimal API Sketch

The names can change during implementation, but V1 should cover these shapes:

```rust
pub struct UniformShape {
    pub dictionary_size: NonZeroUsize,
    pub word_count: usize,
}

pub struct MixedShape<'a> {
    pub dictionary_sizes: &'a [NonZeroUsize],
}

pub struct CapacityReport {
    pub capacity: CapacityClass,
    pub word_count: usize,
}

pub struct PositionalReport {
    pub capacity: CapacityClass,
    pub range: u128,
    pub slack: Option<u128>,
    pub acceptance_ratio: Option<RatioU128>,
}

pub enum PlanTarget {
    Range(u128),
    DecimalDigits(u32),
    StateBits(u32),
}

pub enum PlanSolution {
    RequiredWords { word_count: usize, capacity: CapacityClass },
    RequiredDictionarySize { dictionary_size: usize, capacity: CapacityClass },
    RepresentableRange { capacity: CapacityClass },
    Tradeoffs(Vec<PlanCandidate>),
}
```

Keep exact ratio data as integer numerator/denominator. Decimal rendering can
wait for the `std` advisory layer.

`StateBits(n)` means a target range of at least `2^n` representable states. It
is not a security claim unless the caller also states a uniform random sampling
model.

## Core Calculations

The exact kernel should provide:

- `checked_pow_u128(base, exponent) -> Option<u128>`.
- `capacity_uniform(dictionary_size, word_count) -> CapacityClass`.
- `capacity_mixed(dictionary_sizes) -> CapacityClass`.
- `ceil_log_base(range, base) -> usize`.
- `ceil_root(range, word_count) -> usize`.
- `slack(capacity, range) -> Option<u128>`.
- `acceptance_ratio(capacity, range) -> Option<RatioU128>`.

Planning behavior:

- Fixed `range` and `dictionary_size`: solve minimum `word_count`.
- Fixed `range` and `word_count`: solve minimum `dictionary_size`.
- Fixed `dictionary_size` and `word_count`: return capacity.
- If several inputs are unknown, return a bounded candidate table rather than a
  fake single answer.

## Tests

Required tests for the stats helper:

- Hand-computed capacities for small shapes, such as `2^8`, `10^6`, `256^4`,
  `2048^11`, and the transition where `2048^12` exceeds `u128`.
- `ceil_log_base` and `ceil_root` boundary cases where exact powers and
  one-over-exact-power inputs behave differently.
- Positional slack and acceptance ratio for capacity equal to range, range below
  capacity, and range above capacity.
- Mixed positional capacity where per-position dictionary sizes differ.
- BIP-39 table accepts only 12, 15, 18, 21, and 24 words.

Property tests:

- For exact capacities, `ceil_log_base(range, base)` returns the smallest `k`
  such that `base^k >= range`.
- `ceil_root(range, words)` returns the smallest base such that
  `base^words >= range`.
- If `range <= capacity`, then `slack + range == capacity`.

## Deferred

Post-V1 items:

- Big-integer exact capacity math beyond `u128`.
- CLI/report generator. Current CLI planning lives in
  [`cli-plan.md`](cli-plan.md).
- Security recommendation policy.
- Pretty table formatting beyond small rustdoc examples.
- Non-uniform sampling entropy.
- Mixed-positional ergonomic builders.

## Parley Record

- Workflow: architecture-plan.
- Collaborator: Claude.
- Result: consensus.
- Transcript:
  `/tmp/parley/682705f93766/runs/workflow-architecture-plan-3e159b91/transcript.md`.
