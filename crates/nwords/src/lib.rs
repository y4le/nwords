#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Bidirectional word and phrase codecs for entropy and IDs.

#[cfg(feature = "alloc")]
extern crate alloc;

pub use nwords_core as core;

/// Capacity and phrase-shape planning helpers.
#[cfg(feature = "stats")]
pub mod stats;

/// Ready-made scheme adapters.
pub use nwords_schemes as schemes;

/// Built-in wordlists.
#[cfg(feature = "bip39-english")]
pub use nwords_wordlists as wordlists;

/// BIP-39 seed derivation.
#[cfg(feature = "bip39-seed")]
pub use nwords_bip39_seed as bip39_seed;
