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

#[cfg(test)]
mod tests {
    use super::{
        acceptance_ratio, capacity_mixed, capacity_uniform, ceil_log_base, ceil_root,
        checked_pow_u128, slack, CapacityClass, Log2Estimate, RatioU128,
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
