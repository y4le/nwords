#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Core traits and primitives for `nwords`.

#[cfg(feature = "alloc")]
extern crate alloc;

mod bitframe;
mod error;
mod permutation;
mod traits;
mod wordmap;

pub use bitframe::{BitFrame, BitView};
pub use error::{Error, Result};
pub use permutation::IdentityPermutation;
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
    pub use crate::{Formatter, SchemeFrame, SymbolCodec, TextNormalizer, WordParser};
}
