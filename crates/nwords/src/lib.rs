#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Bidirectional word and phrase codecs for entropy and IDs.

#[cfg(feature = "alloc")]
extern crate alloc;

pub use nwords_core as core;

/// BIP-39 phrase codecs and helpers.
#[cfg(feature = "bip39-english")]
pub mod bip39 {
    pub use nwords_schemes::bip39::*;

    /// BIP-39 seed derivation.
    #[cfg(feature = "bip39-seed")]
    pub mod seed {
        pub use nwords_bip39_seed::*;
    }
}

/// Positional N-word codecs.
#[cfg(feature = "positional")]
pub mod positional {
    pub use nwords_schemes::positional::*;
}

/// Arbitrary byte phrase codecs.
#[cfg(feature = "word-bytes")]
pub mod word_bytes {
    pub use nwords_schemes::word_bytes::*;
}

/// Capacity and phrase-shape planning helpers.
#[cfg(feature = "stats")]
pub mod stats;

/// Ready-made scheme adapters.
pub use nwords_schemes as schemes;

/// Built-in wordlists.
#[cfg(any(
    feature = "adjective-animal",
    feature = "bip39-english",
    feature = "named"
))]
pub use nwords_wordlists as wordlists;

/// BIP-39 seed derivation.
#[cfg(feature = "bip39-seed")]
pub use nwords_bip39_seed as bip39_seed;

/// Common imports for encoding, decoding, and planning phrases.
pub mod prelude {
    pub use nwords_core::prelude::*;

    #[cfg(feature = "bip39-english")]
    pub use crate::bip39::{Bip39, Bip39Frame, Bip39Parser, English};

    #[cfg(feature = "bip39-japanese")]
    pub use crate::bip39::{Japanese, RustBitcoinDisplay, SpecJapanese};

    #[cfg(feature = "adjective-animal")]
    pub use crate::wordlists::adjective_animal::AdjectiveAnimal;

    #[cfg(feature = "named")]
    pub use crate::wordlists::named::{NamedWordList, WordListRole, WordListSequence};

    #[cfg(all(feature = "named", feature = "alloc"))]
    pub use crate::wordlists::named::{
        DynamicWordListSequence, DynamicWordListSlot, OwnedWordList, WordListError,
    };

    #[cfg(feature = "positional")]
    pub use crate::positional::{MixedPositional, Positional};

    #[cfg(feature = "word-bytes")]
    pub use crate::word_bytes::WordBytes;
}

/// Variable-length leading-repeat integer encodings.
#[cfg(feature = "positional")]
pub use nwords_schemes::variable;
