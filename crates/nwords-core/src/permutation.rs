use crate::{Error, Permutation, Result};

/// Identity permutation over a bounded integer domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IdentityPermutation {
    domain: u128,
}

impl IdentityPermutation {
    /// Creates an identity permutation over `0..domain`.
    pub const fn new(domain: u128) -> Self {
        Self { domain }
    }

    fn check(&self, id: u128) -> Result<u128> {
        if id < self.domain {
            Ok(id)
        } else {
            Err(Error::IndexOutOfRange {
                value: id,
                range: self.domain,
            })
        }
    }
}

impl Permutation for IdentityPermutation {
    fn domain(&self) -> u128 {
        self.domain
    }

    fn permute(&self, id: u128) -> Result<u128> {
        self.check(id)
    }

    fn invert(&self, id: u128) -> Result<u128> {
        self.check(id)
    }
}

/// Affine permutation over a bounded integer domain.
///
/// The permutation maps `x` to `(multiplier * x + offset) mod range`.
/// `multiplier` must be coprime to `range` so the mapping is reversible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AffinePermutation {
    multiplier: u128,
    inverse_multiplier: u128,
    offset: u128,
    range: u128,
}

impl AffinePermutation {
    /// Creates an affine permutation over `0..range`.
    pub fn new(multiplier: u128, offset: u128, range: u128) -> Result<Self> {
        if range == 0 {
            return Err(Error::InvalidRange { got: range });
        }
        if gcd(multiplier, range) != 1 {
            return Err(Error::InvalidPermutation { multiplier, range });
        }

        let multiplier = multiplier % range;
        let offset = offset % range;
        let inverse_multiplier = if range == 1 {
            0
        } else {
            modular_inverse(multiplier, range)
                .ok_or(Error::InvalidPermutation { multiplier, range })?
        };

        Ok(Self {
            multiplier,
            inverse_multiplier,
            offset,
            range,
        })
    }

    /// Returns the normalized multiplier.
    pub const fn multiplier(&self) -> u128 {
        self.multiplier
    }

    /// Returns the normalized offset.
    pub const fn offset(&self) -> u128 {
        self.offset
    }

    fn check(&self, id: u128) -> Result<u128> {
        if id < self.range {
            Ok(id)
        } else {
            Err(Error::IndexOutOfRange {
                value: id,
                range: self.range,
            })
        }
    }
}

impl Permutation for AffinePermutation {
    fn domain(&self) -> u128 {
        self.range
    }

    fn permute(&self, id: u128) -> Result<u128> {
        let id = self.check(id)?;
        Ok(add_mod(
            mul_mod(self.multiplier, id, self.range),
            self.offset,
            self.range,
        ))
    }

    fn invert(&self, id: u128) -> Result<u128> {
        let id = self.check(id)?;
        let shifted = sub_mod(id, self.offset, self.range);
        Ok(mul_mod(self.inverse_multiplier, shifted, self.range))
    }
}

fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        let remainder = a % b;
        a = b;
        b = remainder;
    }
    a
}

fn modular_inverse(a: u128, modulus: u128) -> Option<u128> {
    let mut previous_coefficient = 0;
    let mut coefficient = 1;
    let mut previous_remainder = modulus;
    let mut remainder = a % modulus;

    while remainder != 0 {
        let quotient = previous_remainder / remainder;
        let next_coefficient = sub_mod(
            previous_coefficient,
            mul_mod(quotient, coefficient, modulus),
            modulus,
        );
        let next_remainder = previous_remainder - quotient * remainder;

        previous_coefficient = coefficient;
        coefficient = next_coefficient;
        previous_remainder = remainder;
        remainder = next_remainder;
    }

    if previous_remainder == 1 {
        Some(previous_coefficient)
    } else {
        None
    }
}

fn add_mod(a: u128, b: u128, modulus: u128) -> u128 {
    debug_assert!(modulus > 0);
    if modulus == 1 {
        return 0;
    }
    let a = a % modulus;
    let b = b % modulus;
    if a >= modulus - b {
        a - (modulus - b)
    } else {
        a + b
    }
}

fn sub_mod(a: u128, b: u128, modulus: u128) -> u128 {
    debug_assert!(modulus > 0);
    if modulus == 1 {
        return 0;
    }
    let a = a % modulus;
    let b = b % modulus;
    if a >= b {
        a - b
    } else {
        modulus - (b - a)
    }
}

fn mul_mod(mut a: u128, mut b: u128, modulus: u128) -> u128 {
    debug_assert!(modulus > 0);
    if modulus == 1 {
        return 0;
    }
    a %= modulus;
    b %= modulus;

    let mut product = 0;
    while b != 0 {
        if b & 1 == 1 {
            product = add_mod(product, a, modulus);
        }
        b >>= 1;
        if b != 0 {
            a = add_mod(a, a, modulus);
        }
    }
    product
}

#[cfg(test)]
mod tests {
    use super::{AffinePermutation, IdentityPermutation};
    use crate::{Error, Permutation};
    use std::collections::BTreeSet;

    #[test]
    fn identity_round_trips_in_domain() {
        let permutation = IdentityPermutation::new(10);

        assert_eq!(permutation.domain(), 10);
        assert_eq!(permutation.permute(0), Ok(0));
        assert_eq!(permutation.permute(9), Ok(9));
        assert_eq!(permutation.invert(7), Ok(7));
    }

    #[test]
    fn identity_rejects_out_of_domain_ids() {
        let permutation = IdentityPermutation::new(10);

        assert_eq!(
            permutation.permute(10),
            Err(Error::IndexOutOfRange {
                value: 10,
                range: 10
            })
        );
    }

    #[test]
    fn affine_round_trips_in_domain() {
        let permutation = AffinePermutation::new(7, 3, 20).expect("permutation");

        assert_eq!(permutation.domain(), 20);
        assert_eq!(permutation.multiplier(), 7);
        assert_eq!(permutation.offset(), 3);

        for id in 0..20 {
            let permuted = permutation.permute(id).expect("permute");
            assert_eq!(permutation.invert(permuted), Ok(id));
        }
    }

    #[test]
    fn affine_round_trips_large_boundaries() {
        let u64_permutation = AffinePermutation::new(
            6_364_136_223_846_793_005,
            1_442_695_040_888_963_407,
            1u128 << 64,
        )
        .expect("u64 permutation");
        for id in [0, 1, (1u128 << 63) - 1, 1u128 << 63, (1u128 << 64) - 1] {
            let permuted = u64_permutation.permute(id).expect("permute");
            assert_eq!(u64_permutation.invert(permuted), Ok(id));
        }

        let decimal_permutation = AffinePermutation::new(
            6_364_136_223_846_793_007,
            1_442_695_040_888_963_407,
            1_000_000_000_000_000_000,
        )
        .expect("decimal permutation");
        for id in [
            0,
            1,
            499_999_999_999_999_999,
            500_000_000_000_000_000,
            999_999_999_999_999_999,
        ] {
            let permuted = decimal_permutation.permute(id).expect("permute");
            assert_eq!(decimal_permutation.invert(permuted), Ok(id));
        }
    }

    #[test]
    fn affine_is_bijective_for_small_ranges() {
        for range in 1..32 {
            let permutation = AffinePermutation::new(5, 7, range)
                .or_else(|_| AffinePermutation::new(3, 7, range))
                .or_else(|_| AffinePermutation::new(1, 7, range))
                .expect("permutation");
            let mut seen = BTreeSet::new();

            for id in 0..range {
                let permuted = permutation.permute(id).expect("permute");
                assert!(seen.insert(permuted));
                assert_eq!(permutation.invert(permuted), Ok(id));
            }
            assert_eq!(seen.len(), range as usize);
        }
    }

    #[test]
    fn affine_rejects_non_invertible_multiplier() {
        assert_eq!(
            AffinePermutation::new(10, 0, 20),
            Err(Error::InvalidPermutation {
                multiplier: 10,
                range: 20
            })
        );
    }

    #[test]
    fn affine_rejects_zero_range() {
        assert_eq!(
            AffinePermutation::new(1, 0, 0),
            Err(Error::InvalidRange { got: 0 })
        );
    }

    #[test]
    fn affine_rejects_out_of_domain_ids() {
        let permutation = AffinePermutation::new(7, 3, 20).expect("permutation");

        assert_eq!(
            permutation.permute(20),
            Err(Error::IndexOutOfRange {
                value: 20,
                range: 20
            })
        );
        assert_eq!(
            permutation.invert(20),
            Err(Error::IndexOutOfRange {
                value: 20,
                range: 20
            })
        );
    }
}
