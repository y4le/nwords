//! Capacity and phrase-shape planning helpers.

pub use nwords_core::stats::*;

/// BIP-39 word-count and entropy planning helpers.
#[cfg(feature = "bip39")]
pub mod bip39 {
    pub use nwords_schemes::bip39::stats::*;
}
