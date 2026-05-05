use alloc::{string::String, vec::Vec};

use nwords_core::{BaseN, Error, Formatter, IdentityPermutation, Permutation, Result, WordMap};

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

#[cfg(test)]
mod tests {
    use super::Positional;
    use nwords_core::{Error, Linear, WordMap};

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
}
