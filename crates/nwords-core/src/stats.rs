//! Exact capacity, range, and dictionary planning helpers.

use crate::{Error, Result};

/// Inclusive lower and upper bounds for `log2(capacity)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Log2Estimate {
    /// Inclusive lower bound.
    pub lower_bits: u32,
    /// Inclusive upper bound.
    pub upper_bits: u32,
}

/// Exact or estimated capacity of a phrase shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapacityClass {
    /// Exact state count.
    Exact(u128),
    /// State count is larger than `u128::MAX`.
    BeyondU128 {
        /// Log-domain estimate.
        log2: Log2Estimate,
    },
}

/// Exact integer ratio.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RatioU128 {
    /// Ratio numerator.
    pub numerator: u128,
    /// Ratio denominator.
    pub denominator: u128,
}

/// Uniform positional dictionary shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UniformShape {
    /// Dictionary size used at every position.
    pub dictionary_size: usize,
    /// Fixed word count.
    pub word_count: usize,
}

/// Mixed positional dictionary shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MixedShape<'a> {
    /// Per-position dictionary sizes.
    pub dictionary_sizes: &'a [usize],
}

/// Capacity report for a phrase shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapacityReport {
    /// Exact or estimated capacity.
    pub capacity: CapacityClass,
    /// Fixed word count.
    pub word_count: usize,
}

/// Positional planning report for a configured range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PositionalReport {
    /// Exact or estimated representational capacity.
    pub capacity: CapacityClass,
    /// Accepted ID range, usually `[0, range)`.
    pub range: u128,
    /// Exact slack when capacity is exact.
    pub slack: Option<u128>,
    /// Exact acceptance ratio when capacity is exact.
    pub acceptance_ratio: Option<RatioU128>,
}

/// Planning target for capacity calculations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanTarget {
    /// Target exact ID range.
    Range(u128),
    /// Target range of at least `10^digits` states.
    DecimalDigits(u32),
    /// Target range of at least `2^bits` states.
    StateBits(u32),
}

/// Planner solution for one fixed-input question.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanSolution {
    /// Required word count for fixed range and dictionary size.
    RequiredWords {
        /// Minimum required word count.
        word_count: usize,
        /// Capacity at the returned word count.
        capacity: CapacityClass,
    },
    /// Required dictionary size for fixed range and word count.
    RequiredDictionarySize {
        /// Minimum required dictionary size.
        dictionary_size: usize,
        /// Capacity at the returned dictionary size.
        capacity: CapacityClass,
    },
    /// Representable range for fixed dictionary size and word count.
    RepresentableRange {
        /// Exact or estimated capacity.
        capacity: CapacityClass,
    },
}

/// One candidate in a bounded tradeoff table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlanCandidate {
    /// Dictionary size.
    pub dictionary_size: usize,
    /// Word count.
    pub word_count: usize,
    /// Capacity for this candidate.
    pub capacity: CapacityClass,
}

/// Fixed-capacity tradeoff table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CandidateTable<const N: usize> {
    candidates: [Option<PlanCandidate>; N],
    len: usize,
}

impl<const N: usize> CandidateTable<N> {
    /// Creates an empty table.
    pub const fn new() -> Self {
        Self {
            candidates: [None; N],
            len: 0,
        }
    }

    /// Returns the number of candidates in the table.
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the table has no candidates.
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Iterates over candidates.
    pub const fn iter(&self) -> CandidateIter<'_, N> {
        CandidateIter {
            table: self,
            index: 0,
        }
    }

    fn push_sorted(&mut self, candidate: PlanCandidate) {
        let mut insert_at = self.len;
        for index in 0..self.len {
            let Some(existing) = self.candidates[index] else {
                continue;
            };
            if candidate_sort_key(candidate) < candidate_sort_key(existing) {
                insert_at = index;
                break;
            }
        }

        if insert_at >= N {
            return;
        }
        if self.len < N {
            self.len += 1;
        }
        for index in (insert_at + 1..self.len).rev() {
            self.candidates[index] = self.candidates[index - 1];
        }
        self.candidates[insert_at] = Some(candidate);
    }
}

impl<const N: usize> Default for CandidateTable<N> {
    fn default() -> Self {
        Self::new()
    }
}

/// Iterator over a [`CandidateTable`].
#[derive(Debug, Clone)]
pub struct CandidateIter<'a, const N: usize> {
    table: &'a CandidateTable<N>,
    index: usize,
}

impl<'a, const N: usize> Iterator for CandidateIter<'a, N> {
    type Item = &'a PlanCandidate;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.table.len {
            return None;
        }
        let item = self.table.candidates[self.index].as_ref();
        self.index += 1;
        item
    }
}

/// Computes capacity for a uniform shape.
pub fn capacity_report(shape: UniformShape) -> Result<CapacityReport> {
    Ok(CapacityReport {
        capacity: capacity_uniform(shape.dictionary_size, shape.word_count)?,
        word_count: shape.word_count,
    })
}

/// Computes capacity for a mixed shape.
pub fn mixed_capacity_report(shape: MixedShape<'_>) -> Result<CapacityReport> {
    Ok(CapacityReport {
        capacity: capacity_mixed(shape.dictionary_sizes)?,
        word_count: shape.dictionary_sizes.len(),
    })
}

/// Builds a positional report for a uniform shape and accepted range.
pub fn positional_report(shape: UniformShape, range: u128) -> Result<PositionalReport> {
    let capacity = capacity_uniform(shape.dictionary_size, shape.word_count)?;
    Ok(PositionalReport {
        capacity,
        range,
        slack: slack(capacity, range),
        acceptance_ratio: acceptance_ratio(capacity, range),
    })
}

/// Solves fixed range plus dictionary size to find the required word count.
///
/// ```
/// use nwords_core::stats::{required_words, CapacityClass, PlanSolution, PlanTarget};
///
/// let solution = required_words(PlanTarget::Range(1_000), 10).unwrap();
/// assert_eq!(
///     solution,
///     PlanSolution::RequiredWords {
///         word_count: 3,
///         capacity: CapacityClass::Exact(1_000),
///     }
/// );
/// ```
pub fn required_words(target: PlanTarget, dictionary_size: usize) -> Result<PlanSolution> {
    let range = target.range()?;
    let word_count = ceil_log_base(range, dictionary_size)?;
    Ok(PlanSolution::RequiredWords {
        word_count,
        capacity: capacity_uniform(dictionary_size, word_count)?,
    })
}

/// Solves fixed range plus word count to find the required dictionary size.
///
/// ```
/// use nwords_core::stats::{required_dictionary_size, CapacityClass, PlanSolution, PlanTarget};
///
/// let solution = required_dictionary_size(PlanTarget::Range(1_001), 3).unwrap();
/// assert_eq!(
///     solution,
///     PlanSolution::RequiredDictionarySize {
///         dictionary_size: 11,
///         capacity: CapacityClass::Exact(1_331),
///     }
/// );
/// ```
pub fn required_dictionary_size(target: PlanTarget, word_count: usize) -> Result<PlanSolution> {
    let range = target.range()?;
    let dictionary_size = ceil_root(range, word_count)?;
    Ok(PlanSolution::RequiredDictionarySize {
        dictionary_size,
        capacity: capacity_uniform(dictionary_size, word_count)?,
    })
}

/// Solves fixed dictionary size plus word count to find representable range.
///
/// ```
/// use nwords_core::stats::{representable_range, CapacityClass, PlanSolution};
///
/// let solution = representable_range(10, 3).unwrap();
/// assert_eq!(
///     solution,
///     PlanSolution::RepresentableRange {
///         capacity: CapacityClass::Exact(1_000),
///     }
/// );
/// ```
pub fn representable_range(dictionary_size: usize, word_count: usize) -> Result<PlanSolution> {
    Ok(PlanSolution::RepresentableRange {
        capacity: capacity_uniform(dictionary_size, word_count)?,
    })
}

/// Builds a bounded candidate table when dictionary size and word count are not
/// both fixed.
///
/// ```
/// use nwords_core::stats::{tradeoffs, PlanTarget};
///
/// let table = tradeoffs::<4>(PlanTarget::Range(1_000), &[10, 32], 4).unwrap();
/// let mut iter = table.iter().map(|c| (c.dictionary_size, c.word_count));
/// assert_eq!(iter.next(), Some((32, 2)));
/// assert_eq!(iter.next(), Some((10, 3)));
/// assert_eq!(iter.next(), None);
/// ```
pub fn tradeoffs<const N: usize>(
    target: PlanTarget,
    dictionary_sizes: &[usize],
    max_word_count: usize,
) -> Result<CandidateTable<N>> {
    let mut table = CandidateTable::new();
    for &dictionary_size in dictionary_sizes {
        let (word_count, capacity) = match required_words(target, dictionary_size)? {
            PlanSolution::RequiredWords {
                word_count,
                capacity,
            } => (word_count, capacity),
            _ => return Err(Error::BitFrameOverflow),
        };
        if word_count <= max_word_count {
            table.push_sorted(PlanCandidate {
                dictionary_size,
                word_count,
                capacity,
            });
        }
    }
    Ok(table)
}

impl PlanTarget {
    fn range(self) -> Result<u128> {
        match self {
            Self::Range(range) if range > 0 => Ok(range),
            Self::Range(range) => Err(Error::InvalidRange { got: range }),
            Self::DecimalDigits(digits) => checked_pow_u128(10, digits as usize)
                .filter(|range| *range > 0)
                .ok_or(Error::BitFrameOverflow),
            Self::StateBits(bits) if bits < 128 => Ok(1u128 << bits),
            Self::StateBits(_) => Err(Error::BitFrameOverflow),
        }
    }
}

/// Computes `base.pow(exponent)` using checked `u128` math.
pub fn checked_pow_u128(base: u128, exponent: usize) -> Option<u128> {
    let mut value = 1u128;
    for _ in 0..exponent {
        value = value.checked_mul(base)?;
    }
    Some(value)
}

/// Computes uniform positional capacity.
pub fn capacity_uniform(dictionary_size: usize, word_count: usize) -> Result<CapacityClass> {
    if dictionary_size == 0 {
        return Err(Error::InvalidDictionarySize {
            got: dictionary_size,
        });
    }

    if let Some(capacity) = checked_pow_u128(dictionary_size as u128, word_count) {
        Ok(CapacityClass::Exact(capacity))
    } else {
        Ok(CapacityClass::BeyondU128 {
            log2: uniform_log2_bounds(dictionary_size, word_count),
        })
    }
}

/// Computes mixed positional capacity.
pub fn capacity_mixed(dictionary_sizes: &[usize]) -> Result<CapacityClass> {
    let mut exact = 1u128;
    let mut lower_bits = 0u32;
    let mut upper_bits = 0u32;
    let mut overflowed = false;

    for &size in dictionary_sizes {
        if size == 0 {
            return Err(Error::InvalidDictionarySize { got: size });
        }
        lower_bits = lower_bits.saturating_add(floor_log2(size as u128));
        upper_bits = upper_bits.saturating_add(ceil_log2_integer(size as u128));
        if !overflowed {
            if let Some(next) = exact.checked_mul(size as u128) {
                exact = next;
            } else {
                overflowed = true;
            }
        }
    }

    if overflowed {
        Ok(CapacityClass::BeyondU128 {
            log2: Log2Estimate {
                lower_bits,
                upper_bits,
            },
        })
    } else {
        Ok(CapacityClass::Exact(exact))
    }
}

/// Returns the smallest `k` such that `base.pow(k) >= range`.
pub fn ceil_log_base(range: u128, base: usize) -> Result<usize> {
    if base < 2 {
        return Err(Error::InvalidDictionarySize { got: base });
    }
    if range <= 1 {
        return Ok(0);
    }

    let mut words = 0usize;
    let mut capacity = 1u128;
    while capacity < range {
        words = words.checked_add(1).ok_or(Error::BitFrameOverflow)?;
        match capacity.checked_mul(base as u128) {
            Some(next) => capacity = next,
            None => return Ok(words),
        }
    }
    Ok(words)
}

/// Returns the smallest dictionary size whose `word_count` power covers `range`.
pub fn ceil_root(range: u128, word_count: usize) -> Result<usize> {
    if word_count == 0 {
        return Err(Error::InvalidWordCount { got: word_count });
    }
    if range <= 1 {
        return Ok(1);
    }

    let max_usize = usize::MAX as u128;
    let mut low = 1u128;
    let mut high = range.min(max_usize);

    if pow_covers(high, word_count, range).is_none() {
        return Err(Error::BitFrameOverflow);
    }

    while low < high {
        let mid = low + (high - low) / 2;
        if pow_covers(mid, word_count, range).is_some() {
            high = mid;
        } else {
            low = mid + 1;
        }
    }

    usize::try_from(low).map_err(|_| Error::BitFrameOverflow)
}

/// Computes exact slack when capacity is exact and covers range.
pub fn slack(capacity: CapacityClass, range: u128) -> Option<u128> {
    match capacity {
        CapacityClass::Exact(capacity) => capacity.checked_sub(range),
        CapacityClass::BeyondU128 { .. } => None,
    }
}

/// Computes exact acceptance ratio when capacity is exact and covers range.
pub fn acceptance_ratio(capacity: CapacityClass, range: u128) -> Option<RatioU128> {
    match capacity {
        CapacityClass::Exact(capacity) if range <= capacity => Some(RatioU128 {
            numerator: range,
            denominator: capacity,
        }),
        _ => None,
    }
}

fn uniform_log2_bounds(dictionary_size: usize, word_count: usize) -> Log2Estimate {
    Log2Estimate {
        lower_bits: floor_log2(dictionary_size as u128).saturating_mul(word_count as u32),
        upper_bits: ceil_log2_integer(dictionary_size as u128).saturating_mul(word_count as u32),
    }
}

fn floor_log2(value: u128) -> u32 {
    if value == 0 {
        0
    } else {
        127 - value.leading_zeros()
    }
}

fn ceil_log2_integer(value: u128) -> u32 {
    if value <= 1 {
        0
    } else {
        128 - (value - 1).leading_zeros()
    }
}

fn pow_covers(base: u128, exponent: usize, target: u128) -> Option<()> {
    let mut value = 1u128;
    for _ in 0..exponent {
        value = match value.checked_mul(base) {
            Some(value) => value,
            None => return Some(()),
        };
        if value >= target {
            return Some(());
        }
    }
    if value >= target {
        Some(())
    } else {
        None
    }
}

fn candidate_sort_key(candidate: PlanCandidate) -> (usize, usize) {
    (candidate.word_count, candidate.dictionary_size)
}

#[cfg(test)]
mod tests {
    use super::{
        acceptance_ratio, capacity_mixed, capacity_report, capacity_uniform, ceil_log_base,
        ceil_root, checked_pow_u128, mixed_capacity_report, positional_report, representable_range,
        required_dictionary_size, required_words, slack, tradeoffs, CapacityClass, Log2Estimate,
        MixedShape, PlanSolution, PlanTarget, RatioU128, UniformShape,
    };
    use crate::Error;

    #[test]
    fn checked_power_reports_overflow() {
        assert_eq!(checked_pow_u128(2, 8), Some(256));
        assert_eq!(checked_pow_u128(2048, 11), Some(1u128 << 121));
        assert_eq!(checked_pow_u128(2048, 12), None);
    }

    #[test]
    fn uniform_capacity_tracks_exact_and_beyond_u128() {
        assert_eq!(capacity_uniform(2, 8), Ok(CapacityClass::Exact(256)));
        assert_eq!(capacity_uniform(10, 6), Ok(CapacityClass::Exact(1_000_000)));
        assert_eq!(
            capacity_uniform(256, 4),
            Ok(CapacityClass::Exact(4_294_967_296))
        );
        assert_eq!(
            capacity_uniform(2048, 11),
            Ok(CapacityClass::Exact(1u128 << 121))
        );
        assert_eq!(
            capacity_uniform(2048, 12),
            Ok(CapacityClass::BeyondU128 {
                log2: Log2Estimate {
                    lower_bits: 132,
                    upper_bits: 132
                }
            })
        );
    }

    #[test]
    fn mixed_capacity_tracks_exact_and_beyond_u128() {
        assert_eq!(capacity_mixed(&[2, 3, 5]), Ok(CapacityClass::Exact(30)));
        assert!(matches!(
            capacity_mixed(&[2048; 12]),
            Ok(CapacityClass::BeyondU128 { .. })
        ));
    }

    #[test]
    fn ceil_log_base_handles_boundaries() {
        assert_eq!(ceil_log_base(1, 10), Ok(0));
        assert_eq!(ceil_log_base(1000, 10), Ok(3));
        assert_eq!(ceil_log_base(1001, 10), Ok(4));
    }

    #[test]
    fn ceil_root_handles_boundaries() {
        assert_eq!(ceil_root(1000, 3), Ok(10));
        assert_eq!(ceil_root(1001, 3), Ok(11));
        assert_eq!(ceil_root(1, 3), Ok(1));
    }

    #[test]
    fn ceil_root_treats_overflow_as_covering_target() {
        assert_eq!(ceil_root(u128::MAX, 128), Ok(2));
        assert_eq!(ceil_root(1u128 << 120, 3), Ok(1usize << 40));
    }

    #[test]
    fn rejection_metrics_use_exact_capacity() {
        let capacity = CapacityClass::Exact(1000);

        assert_eq!(slack(capacity, 1000), Some(0));
        assert_eq!(slack(capacity, 900), Some(100));
        assert_eq!(slack(capacity, 1001), None);
        assert_eq!(
            acceptance_ratio(capacity, 900),
            Some(RatioU128 {
                numerator: 900,
                denominator: 1000
            })
        );
    }

    #[test]
    fn report_helpers_wrap_capacity_and_rejection_metrics() {
        assert_eq!(
            capacity_report(UniformShape {
                dictionary_size: 10,
                word_count: 3
            }),
            Ok(super::CapacityReport {
                capacity: CapacityClass::Exact(1000),
                word_count: 3
            })
        );
        assert_eq!(
            mixed_capacity_report(MixedShape {
                dictionary_sizes: &[2, 3, 5]
            }),
            Ok(super::CapacityReport {
                capacity: CapacityClass::Exact(30),
                word_count: 3
            })
        );
        assert_eq!(
            positional_report(
                UniformShape {
                    dictionary_size: 10,
                    word_count: 3
                },
                900
            ),
            Ok(super::PositionalReport {
                capacity: CapacityClass::Exact(1000),
                range: 900,
                slack: Some(100),
                acceptance_ratio: Some(RatioU128 {
                    numerator: 900,
                    denominator: 1000
                })
            })
        );
    }

    #[test]
    fn planner_solves_fixed_input_questions() {
        assert_eq!(
            required_words(PlanTarget::Range(1000), 10),
            Ok(PlanSolution::RequiredWords {
                word_count: 3,
                capacity: CapacityClass::Exact(1000)
            })
        );
        assert_eq!(
            required_dictionary_size(PlanTarget::Range(1001), 3),
            Ok(PlanSolution::RequiredDictionarySize {
                dictionary_size: 11,
                capacity: CapacityClass::Exact(1331)
            })
        );
        assert_eq!(
            representable_range(10, 3),
            Ok(PlanSolution::RepresentableRange {
                capacity: CapacityClass::Exact(1000)
            })
        );
    }

    #[test]
    fn planner_supports_decimal_digits_and_state_bits() {
        assert_eq!(
            required_words(PlanTarget::DecimalDigits(6), 10),
            Ok(PlanSolution::RequiredWords {
                word_count: 6,
                capacity: CapacityClass::Exact(1_000_000)
            })
        );
        assert_eq!(
            required_words(PlanTarget::StateBits(20), 1024),
            Ok(PlanSolution::RequiredWords {
                word_count: 2,
                capacity: CapacityClass::Exact(1_048_576)
            })
        );
    }

    #[test]
    fn planner_rejects_unrepresentable_targets() {
        assert_eq!(
            required_words(PlanTarget::Range(0), 10),
            Err(Error::InvalidRange { got: 0 })
        );
        assert_eq!(
            required_words(PlanTarget::StateBits(128), 10),
            Err(Error::BitFrameOverflow)
        );
    }

    #[test]
    fn tradeoff_candidates_are_bounded_and_sorted() {
        let table = tradeoffs::<2>(PlanTarget::Range(1000), &[10, 32, 100], 4).expect("tradeoffs");
        let mut candidates = table
            .iter()
            .map(|candidate| (candidate.dictionary_size, candidate.word_count));

        assert_eq!(candidates.next(), Some((32, 2)));
        assert_eq!(candidates.next(), Some((100, 2)));
        assert_eq!(candidates.next(), None);
    }

    #[test]
    fn invalid_inputs_are_rejected() {
        assert_eq!(
            capacity_uniform(0, 3),
            Err(Error::InvalidDictionarySize { got: 0 })
        );
        assert_eq!(
            ceil_log_base(10, 1),
            Err(Error::InvalidDictionarySize { got: 1 })
        );
        assert_eq!(ceil_root(10, 0), Err(Error::InvalidWordCount { got: 0 }));
    }
}
