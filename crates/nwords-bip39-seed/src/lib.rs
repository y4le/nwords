#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! BIP-39 seed derivation for `nwords`.

#[cfg(feature = "alloc")]
extern crate alloc;
