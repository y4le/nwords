use crate::{BitFrame, BitView, Result};

#[cfg(feature = "alloc")]
use alloc::{borrow::Cow, boxed::Box, string::String, vec::Vec};

/// Bidirectional, position-aware mapping between a symbol index and a word.
pub trait WordMap {
    /// Returns the word count available at `position`.
    fn len(&self, position: usize) -> usize;

    /// Returns true when no words are available at `position`.
    fn is_empty(&self, position: usize) -> bool {
        self.len(position) == 0
    }

    /// Returns the word for `index` at `position`.
    fn word(&self, index: usize, position: usize) -> Option<&str>;

    /// Returns the symbol index for `word` at `position`.
    fn index_of(&self, word: &str, position: usize) -> Option<usize>;
}

/// Render words into a phrase string.
#[cfg(feature = "alloc")]
pub trait Formatter {
    /// Joins words into a display phrase.
    fn join(&self, words: &[&str]) -> String;
}

/// Normalize and split input phrases for word lookup.
#[cfg(feature = "alloc")]
pub trait WordParser {
    /// Normalizes the entire input phrase before splitting.
    fn normalize_phrase<'a>(&self, phrase: &'a str) -> Cow<'a, str>;

    /// Splits a normalized phrase into word tokens.
    fn split<'a>(&self, phrase: &'a str) -> Box<dyn Iterator<Item = &'a str> + 'a>;
}

/// Normalize text for seed derivation and other text-level transforms.
#[cfg(feature = "alloc")]
pub trait TextNormalizer {
    /// Normalizes text.
    fn normalize<'a>(&self, text: &'a str) -> Cow<'a, str>;
}

/// Pack a bit frame into symbol indices, and unpack symbol indices into bits.
#[cfg(feature = "alloc")]
pub trait SymbolCodec {
    /// Packs bits into symbol indices.
    fn pack(&self, frame: BitView<'_>) -> Result<Vec<u32>>;

    /// Unpacks symbol indices into bits.
    fn unpack(&self, symbols: &[u32]) -> Result<BitFrame>;
}

/// Translates a raw payload into a scheme-specific bit frame, and back.
#[cfg(feature = "alloc")]
pub trait SchemeFrame {
    /// Frames a raw payload.
    fn frame(&self, payload: &[u8]) -> Result<BitFrame>;

    /// Unframes a bit frame into a raw payload.
    fn unframe(&self, frame: BitView<'_>) -> Result<Vec<u8>>;
}

/// Optional reversible permutation over an integer ID domain.
pub trait Permutation {
    /// Returns the exclusive upper bound for accepted IDs.
    fn domain(&self) -> u128;

    /// Maps an ID to its permuted value.
    fn permute(&self, id: u128) -> Result<u128>;

    /// Maps a permuted ID back to its original value.
    fn invert(&self, id: u128) -> Result<u128>;
}
