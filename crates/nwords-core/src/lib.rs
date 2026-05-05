#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Core traits and primitives for `nwords`.

#[cfg(feature = "alloc")]
extern crate alloc;

mod bitframe;
mod error;
mod permutation;
#[cfg(feature = "stats")]
pub mod stats;
mod symbol;
mod traits;
mod wordmap;

pub use bitframe::{BitFrame, BitView};
pub use error::{Error, Result};
pub use permutation::IdentityPermutation;
#[cfg(all(feature = "alloc", feature = "stats"))]
pub use symbol::BaseN;
#[cfg(feature = "alloc")]
pub use symbol::BigEndian11Bit;
pub use traits::{Permutation, WordMap};
pub use wordmap::{Linear, Sorted};

#[cfg(feature = "alloc")]
pub use traits::{Formatter, SchemeFrame, SymbolCodec, TextNormalizer, WordParser};

/// Common imports for users implementing or composing codecs.
pub mod prelude {
    pub use crate::{
        BitFrame, BitView, Error, IdentityPermutation, Linear, Permutation, Result, Sorted, WordMap,
    };

    #[cfg(feature = "alloc")]
    pub use crate::{
        BigEndian11Bit, Formatter, SchemeFrame, SymbolCodec, TextNormalizer, WordParser,
    };

    #[cfg(all(feature = "alloc", feature = "stats"))]
    pub use crate::BaseN;
}
