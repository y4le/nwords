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

/// Variable-length leading-repeat integer encodings.
#[cfg(feature = "positional")]
pub mod variable;

/// Arbitrary-width variable-v1 IDs and exact bit, byte, and text views.
#[cfg(feature = "positional")]
pub mod wide_variable;

/// Framed byte blocks over arbitrary mixed wordlists.
#[cfg(feature = "word-bytes")]
pub mod radix_bytes;
