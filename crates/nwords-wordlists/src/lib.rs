#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Built-in wordlists for `nwords`.

/// Curated English adjective-animal wordlists.
#[cfg(feature = "adjective-animal")]
pub mod adjective_animal;

/// BIP-39 wordlists.
pub mod bip39;
