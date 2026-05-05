#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Ready-made schemes for `nwords`.

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
mod format;

#[cfg(feature = "alloc")]
pub use format::AsciiSpace;

/// BIP-39 scheme adapters.
#[cfg(feature = "bip39-english")]
pub mod bip39;

/// Positional N-word scheme adapters.
#[cfg(feature = "positional")]
pub mod positional;

/// Arbitrary byte phrase adapters.
#[cfg(feature = "word-bytes")]
pub mod word_bytes;
