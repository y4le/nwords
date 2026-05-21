use alloc::{string::String, vec::Vec};

use nwords_core::{
    BaseN, Error, Formatter, IdentityPermutation, MixedRadix, Permutation, Result, WordMap,
};

use crate::AsciiSpace;

/// Ready-made positional N-word codec over a word map.
#[derive(Debug, Clone)]
pub struct Positional<W, F = AsciiSpace, P = IdentityPermutation> {
    word_map: W,
    formatter: F,
    permutation: P,
    codec: BaseN,
}

impl<W> Positional<W, AsciiSpace, IdentityPermutation>
where
    W: WordMap,
{
    /// Creates a positional codec using ASCII-space display and identity permutation.
    pub fn new(word_map: W, word_count: usize, range: u128) -> Result<Self> {
        let permutation = IdentityPermutation::new(range);
        Self::with_formatter_and_permutation(word_map, AsciiSpace, permutation, word_count, range)
    }
}

impl<W, F, P> Positional<W, F, P>
where
    W: WordMap,
    F: Formatter,
    P: Permutation,
{
    /// Creates a positional codec from all component parts.
    pub fn with_formatter_and_permutation(
        word_map: W,
        formatter: F,
        permutation: P,
        word_count: usize,
        range: u128,
    ) -> Result<Self> {
        let base = word_map.len(0);
        let codec = BaseN::new(base, word_count, range)?;
        if permutation.domain() != range {
            return Err(Error::InvalidPermutationDomain {
                got: permutation.domain(),
                expected: range,
            });
        }
        for position in 0..word_count {
            let len = word_map.len(position);
            if len != base {
                return Err(Error::InconsistentDictionarySize {
                    position,
                    got: len,
                    expected: base,
                });
            }
        }
        Ok(Self {
            word_map,
            formatter,
            permutation,
            codec,
        })
    }

    /// Returns the underlying positional symbol codec.
    pub const fn symbol_codec(&self) -> BaseN {
        self.codec
    }

    /// Encodes an ID into a phrase.
    pub fn encode(&self, id: u128) -> Result<String> {
        let permuted = self.permutation.permute(id)?;
        let symbols = self.codec.encode_id(permuted)?;
        let mut words = Vec::with_capacity(symbols.len());

        for (position, symbol) in symbols.iter().copied().enumerate() {
            let len = self.word_map.len(position);
            if symbol as usize >= len {
                return Err(Error::SymbolOutOfRange {
                    index: symbol,
                    position,
                    len,
                });
            }
            let word =
                self.word_map
                    .word(symbol as usize, position)
                    .ok_or(Error::SymbolOutOfRange {
                        index: symbol,
                        position,
                        len,
                    })?;
            words.push(word);
        }

        Ok(self.formatter.join(&words))
    }

    /// Decodes already-split phrase words into an ID.
    pub fn decode_words(&self, words: &[&str]) -> Result<u128> {
        if words.len() != self.codec.word_count() {
            return Err(Error::InvalidWordCount { got: words.len() });
        }

        let mut symbols = Vec::with_capacity(words.len());
        for (position, word) in words.iter().enumerate() {
            let symbol = self
                .word_map
                .index_of(word, position)
                .ok_or(Error::UnknownWord { position })?;
            symbols.push(u32::try_from(symbol).map_err(|_| Error::SymbolOutOfRange {
                index: u32::MAX,
                position,
                len: self.word_map.len(position),
            })?);
        }

        let permuted = self.codec.decode_symbols(&symbols)?;
        self.permutation.invert(permuted)
    }
}

/// Ready-made mixed-radix positional codec over a position-aware word map.
#[derive(Debug, Clone)]
pub struct MixedPositional<W, F = AsciiSpace, P = IdentityPermutation> {
    word_map: W,
    formatter: F,
    permutation: P,
    codec: MixedRadix,
}

impl<W> MixedPositional<W, AsciiSpace, IdentityPermutation>
where
    W: WordMap,
{
    /// Creates a mixed positional codec using ASCII-space display and identity permutation.
    pub fn new(word_map: W, word_count: usize, range: u128) -> Result<Self> {
        let permutation = IdentityPermutation::new(range);
        Self::with_formatter_and_permutation(word_map, AsciiSpace, permutation, word_count, range)
    }
}

impl<W, F, P> MixedPositional<W, F, P>
where
    W: WordMap,
    F: Formatter,
    P: Permutation,
{
    /// Creates a mixed positional codec from all component parts.
    pub fn with_formatter_and_permutation(
        word_map: W,
        formatter: F,
        permutation: P,
        word_count: usize,
        range: u128,
    ) -> Result<Self> {
        let mut dictionary_sizes = Vec::with_capacity(word_count);
        for position in 0..word_count {
            dictionary_sizes.push(word_map.len(position));
        }
        let codec = MixedRadix::new(&dictionary_sizes, range)?;
        if permutation.domain() != range {
            return Err(Error::InvalidPermutationDomain {
                got: permutation.domain(),
                expected: range,
            });
        }
        Ok(Self {
            word_map,
            formatter,
            permutation,
            codec,
        })
    }

    /// Returns the underlying mixed-radix symbol codec.
    pub fn symbol_codec(&self) -> &MixedRadix {
        &self.codec
    }

    /// Encodes an ID into a phrase.
    pub fn encode(&self, id: u128) -> Result<String> {
        let permuted = self.permutation.permute(id)?;
        let symbols = self.codec.encode_id(permuted)?;
        let mut words = Vec::with_capacity(symbols.len());

        for (position, symbol) in symbols.iter().copied().enumerate() {
            let len = self.word_map.len(position);
            if symbol as usize >= len {
                return Err(Error::SymbolOutOfRange {
                    index: symbol,
                    position,
                    len,
                });
            }
            let word =
                self.word_map
                    .word(symbol as usize, position)
                    .ok_or(Error::SymbolOutOfRange {
                        index: symbol,
                        position,
                        len,
                    })?;
            words.push(word);
        }

        Ok(self.formatter.join(&words))
    }

    /// Decodes already-split phrase words into an ID.
    pub fn decode_words(&self, words: &[&str]) -> Result<u128> {
        if words.len() != self.codec.word_count() {
            return Err(Error::InvalidWordCount { got: words.len() });
        }

        let mut symbols = Vec::with_capacity(words.len());
        for (position, word) in words.iter().enumerate() {
            let symbol = self
                .word_map
                .index_of(word, position)
                .ok_or(Error::UnknownWord { position })?;
            symbols.push(u32::try_from(symbol).map_err(|_| Error::SymbolOutOfRange {
                index: u32::MAX,
                position,
                len: self.word_map.len(position),
            })?);
        }

        let permuted = self.codec.decode_symbols(&symbols)?;
        self.permutation.invert(permuted)
    }
}

#[cfg(test)]
mod tests {
    use super::{MixedPositional, Positional};
    use nwords_core::{AffinePermutation, Error, Linear, WordMap};

    const WORDS: &[&str] = &[
        "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine",
    ];

    #[test]
    fn positional_codec_round_trips_boundary_ids() {
        let codec = Positional::new(Linear::new(WORDS), 3, 1000).expect("codec");

        assert_eq!(codec.encode(0).expect("encode"), "zero zero zero");
        assert_eq!(codec.encode(999).expect("encode"), "nine nine nine");
        assert_eq!(codec.decode_words(&["zero", "zero", "zero"]), Ok(0));
        assert_eq!(codec.decode_words(&["nine", "nine", "nine"]), Ok(999));
    }

    #[test]
    fn positional_codec_rejects_first_invalid_representation() {
        let codec = Positional::new(Linear::new(WORDS), 3, 100).expect("codec");

        assert_eq!(
            codec.decode_words(&["one", "zero", "zero"]),
            Err(Error::IndexOutOfRange {
                value: 100,
                range: 100
            })
        );
    }

    #[test]
    fn positional_codec_hides_unknown_word_text() {
        let codec = Positional::new(Linear::new(WORDS), 3, 1000).expect("codec");

        assert_eq!(
            codec.decode_words(&["zero", "not-in-error", "zero"]),
            Err(Error::UnknownWord { position: 1 })
        );
    }

    #[test]
    fn positional_codec_rejects_non_uniform_maps() {
        struct NonUniform;

        impl WordMap for NonUniform {
            fn len(&self, position: usize) -> usize {
                if position == 1 {
                    9
                } else {
                    10
                }
            }

            fn word(&self, _index: usize, _position: usize) -> Option<&str> {
                None
            }

            fn index_of(&self, _word: &str, _position: usize) -> Option<usize> {
                None
            }
        }

        assert_eq!(
            Positional::new(NonUniform, 3, 1000).map(|_| ()),
            Err(Error::InconsistentDictionarySize {
                position: 1,
                got: 9,
                expected: 10
            })
        );
    }

    #[test]
    fn positional_codec_rejects_mismatched_permutation_domain() {
        let permutation = AffinePermutation::new(1, 0, 999).expect("permutation");

        assert_eq!(
            Positional::with_formatter_and_permutation(
                Linear::new(WORDS),
                crate::AsciiSpace,
                permutation,
                3,
                1000
            )
            .map(|_| ()),
            Err(Error::InvalidPermutationDomain {
                got: 999,
                expected: 1000
            })
        );
    }

    #[derive(Debug, Clone, Copy)]
    struct AdjectiveAnimalWords;

    impl WordMap for AdjectiveAnimalWords {
        fn len(&self, position: usize) -> usize {
            match position {
                0 => 3,
                1 => 5,
                _ => 0,
            }
        }

        fn word(&self, index: usize, position: usize) -> Option<&str> {
            match position {
                0 => ["brave", "calm", "swift"].get(index).copied(),
                1 => ["ant", "bear", "cat", "dog", "eel"].get(index).copied(),
                _ => None,
            }
        }

        fn index_of(&self, word: &str, position: usize) -> Option<usize> {
            match position {
                0 => ["brave", "calm", "swift"]
                    .iter()
                    .position(|candidate| *candidate == word),
                1 => ["ant", "bear", "cat", "dog", "eel"]
                    .iter()
                    .position(|candidate| *candidate == word),
                _ => None,
            }
        }
    }

    #[test]
    fn mixed_positional_codec_round_trips_boundary_ids() {
        let codec = MixedPositional::new(AdjectiveAnimalWords, 2, 15).expect("codec");

        assert_eq!(codec.symbol_codec().bases(), &[3, 5]);
        assert_eq!(codec.encode(0).expect("encode"), "brave ant");
        assert_eq!(codec.encode(14).expect("encode"), "swift eel");
        assert_eq!(codec.decode_words(&["brave", "ant"]), Ok(0));
        assert_eq!(codec.decode_words(&["swift", "eel"]), Ok(14));
    }

    #[test]
    fn mixed_positional_codec_rejects_first_invalid_representation() {
        let codec = MixedPositional::new(AdjectiveAnimalWords, 2, 12).expect("codec");

        assert_eq!(
            codec.decode_words(&["swift", "cat"]),
            Err(Error::IndexOutOfRange {
                value: 12,
                range: 12
            })
        );
    }

    #[test]
    fn mixed_positional_codec_hides_unknown_word_text() {
        let codec = MixedPositional::new(AdjectiveAnimalWords, 2, 15).expect("codec");

        assert_eq!(
            codec.decode_words(&["brave", "not-in-error"]),
            Err(Error::UnknownWord { position: 1 })
        );
    }

    #[test]
    fn mixed_positional_codec_rejects_invalid_position_size() {
        struct InvalidPosition;

        impl WordMap for InvalidPosition {
            fn len(&self, position: usize) -> usize {
                if position == 0 {
                    1
                } else {
                    5
                }
            }

            fn word(&self, _index: usize, _position: usize) -> Option<&str> {
                None
            }

            fn index_of(&self, _word: &str, _position: usize) -> Option<usize> {
                None
            }
        }

        assert_eq!(
            MixedPositional::new(InvalidPosition, 2, 5).map(|_| ()),
            Err(Error::InvalidDictionarySize { got: 1 })
        );
    }

    #[test]
    fn mixed_positional_codec_rejects_mismatched_permutation_domain() {
        let permutation = AffinePermutation::new(1, 0, 14).expect("permutation");

        assert_eq!(
            MixedPositional::with_formatter_and_permutation(
                AdjectiveAnimalWords,
                crate::AsciiSpace,
                permutation,
                2,
                15
            )
            .map(|_| ()),
            Err(Error::InvalidPermutationDomain {
                got: 14,
                expected: 15
            })
        );
    }
}
