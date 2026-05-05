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

#[cfg(test)]
mod tests {
    use super::IdentityPermutation;
    use crate::{Error, Permutation};

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
}
