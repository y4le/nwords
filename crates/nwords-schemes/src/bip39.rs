use alloc::{borrow::Cow, boxed::Box, string::String, vec::Vec};

use nwords_core::{
    BigEndian11Bit, BitFrame, BitView, Error, Formatter, Result, SchemeFrame, SymbolCodec, WordMap,
    WordParser,
};
use sha2::{Digest, Sha256};

use crate::AsciiSpace;

const VALID_ENTROPY_BYTES: &[usize] = &[16, 20, 24, 28, 32];

/// BIP-39 entropy/checksum frame.
#[derive(Debug, Clone, Copy, Default)]
pub struct Bip39Frame;

impl SchemeFrame for Bip39Frame {
    fn frame(&self, payload: &[u8]) -> Result<BitFrame> {
        validate_entropy_len(payload.len())?;
        let entropy_bits = payload.len() * 8;
        let checksum_bits = entropy_bits / 32;
        let hash = Sha256::digest(payload);
        let mut frame = BitFrame::with_bit_capacity(entropy_bits + checksum_bits);

        for &byte in payload {
            frame.push_bits(u128::from(byte), 8)?;
        }
        for bit in 0..checksum_bits {
            frame.push_bit(hash_bit(&hash, bit))?;
        }

        Ok(frame)
    }

    fn unframe(&self, frame: BitView<'_>) -> Result<Vec<u8>> {
        let entropy_bits = entropy_bits_from_frame_len(frame.bit_len())?;
        let checksum_bits = entropy_bits / 32;
        let mut entropy = Vec::with_capacity(entropy_bits / 8);

        for byte_start in (0..entropy_bits).step_by(8) {
            let mut byte = 0u8;
            for bit in 0..8 {
                byte <<= 1;
                if frame.bit(byte_start + bit).ok_or(Error::BitFrameOverflow)? {
                    byte |= 1;
                }
            }
            entropy.push(byte);
        }

        let hash = Sha256::digest(&entropy);
        for bit in 0..checksum_bits {
            let actual = frame
                .bit(entropy_bits + bit)
                .ok_or(Error::BitFrameOverflow)?;
            if actual != hash_bit(&hash, bit) {
                return Err(Error::InvalidChecksum);
            }
        }

        Ok(entropy)
    }
}

/// Parser for BIP-39 phrases.
#[derive(Debug, Clone, Copy, Default)]
pub struct Bip39Parser;

impl WordParser for Bip39Parser {
    fn normalize_phrase<'a>(&self, phrase: &'a str) -> Cow<'a, str> {
        normalize_phrase(phrase)
    }

    fn split<'a>(&self, phrase: &'a str) -> Box<dyn Iterator<Item = &'a str> + 'a> {
        Box::new(phrase.split_whitespace())
    }
}

/// Japanese BIP-39 formatter using U+3000 ideographic spaces.
#[cfg(feature = "bip39-japanese")]
#[derive(Debug, Clone, Copy, Default)]
pub struct SpecJapanese;

#[cfg(feature = "bip39-japanese")]
impl Formatter for SpecJapanese {
    fn join(&self, words: &[&str]) -> String {
        join_with(words, '\u{3000}')
    }
}

/// ASCII-space Japanese formatter for rust-bitcoin display parity.
#[cfg(feature = "bip39-japanese")]
#[derive(Debug, Clone, Copy, Default)]
pub struct RustBitcoinDisplay;

#[cfg(feature = "bip39-japanese")]
impl Formatter for RustBitcoinDisplay {
    fn join(&self, words: &[&str]) -> String {
        join_with(words, ' ')
    }
}

/// BIP-39 codec over a word map and formatter.
#[derive(Debug, Clone, Copy)]
pub struct Bip39<W, F = AsciiSpace, P = Bip39Parser> {
    word_map: W,
    formatter: F,
    parser: P,
    frame: Bip39Frame,
    symbol_codec: BigEndian11Bit,
}

impl<W, F, P> Bip39<W, F, P>
where
    W: WordMap,
    F: Formatter,
    P: WordParser,
{
    /// Creates a BIP-39 codec from components.
    pub const fn new(word_map: W, formatter: F, parser: P) -> Self {
        Self {
            word_map,
            formatter,
            parser,
            frame: Bip39Frame,
            symbol_codec: BigEndian11Bit,
        }
    }

    /// Encodes entropy into a mnemonic phrase.
    pub fn encode_entropy(&self, entropy: &[u8]) -> Result<String> {
        let frame = self.frame.frame(entropy)?;
        let view = BitFrame::from_bytes_with_bit_len(frame.as_bytes(), frame.bit_len())?;
        let symbols = self.symbol_codec.pack(view)?;
        let mut words = Vec::with_capacity(symbols.len());

        for (position, symbol) in symbols.iter().copied().enumerate() {
            let len = self.word_map.len(position);
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

    /// Decodes a mnemonic phrase into entropy.
    pub fn decode_phrase(&self, phrase: &str) -> Result<Vec<u8>> {
        let normalized = self.parser.normalize_phrase(phrase);
        let words = self.parser.split(&normalized).collect::<Vec<_>>();
        self.decode_words(&words)
    }

    /// Decodes already-split mnemonic words into entropy.
    pub fn decode_words(&self, words: &[&str]) -> Result<Vec<u8>> {
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

        let frame = self.symbol_codec.unpack(&symbols)?;
        self.frame.unframe(BitFrame::from_bytes_with_bit_len(
            frame.as_bytes(),
            frame.bit_len(),
        )?)
    }
}

/// English BIP-39 codec.
pub type English = Bip39<nwords_wordlists::bip39::English, AsciiSpace, Bip39Parser>;

impl Default for English {
    fn default() -> Self {
        Self::new(nwords_wordlists::bip39::English, AsciiSpace, Bip39Parser)
    }
}

/// Japanese BIP-39 codec.
#[cfg(feature = "bip39-japanese")]
pub type Japanese = Bip39<nwords_wordlists::bip39::Japanese, SpecJapanese, Bip39Parser>;

#[cfg(feature = "bip39-japanese")]
impl Default for Japanese {
    fn default() -> Self {
        Self::new(nwords_wordlists::bip39::Japanese, SpecJapanese, Bip39Parser)
    }
}

fn validate_entropy_len(len: usize) -> Result<()> {
    if VALID_ENTROPY_BYTES.contains(&len) {
        Ok(())
    } else {
        Err(Error::InvalidEntropyLength {
            got: len,
            expected: VALID_ENTROPY_BYTES,
        })
    }
}

fn entropy_bits_from_frame_len(bit_len: usize) -> Result<usize> {
    for &entropy_bytes in VALID_ENTROPY_BYTES {
        let entropy_bits = entropy_bytes * 8;
        if bit_len == entropy_bits + entropy_bits / 32 {
            return Ok(entropy_bits);
        }
    }
    Err(Error::InvalidBitLength {
        got: bit_len,
        multiple: 33,
    })
}

fn hash_bit(hash: &[u8], bit: usize) -> bool {
    ((hash[bit / 8] >> (7 - (bit % 8))) & 1) == 1
}

#[cfg(feature = "bip39-japanese")]
fn join_with(words: &[&str], separator: char) -> String {
    let mut phrase = String::new();
    for (index, word) in words.iter().enumerate() {
        if index > 0 {
            phrase.push(separator);
        }
        phrase.push_str(word);
    }
    phrase
}

#[cfg(feature = "bip39-japanese")]
fn normalize_phrase(phrase: &str) -> Cow<'_, str> {
    use unicode_normalization::UnicodeNormalization;

    Cow::Owned(phrase.nfkd().collect())
}

#[cfg(not(feature = "bip39-japanese"))]
fn normalize_phrase(phrase: &str) -> Cow<'_, str> {
    Cow::Borrowed(phrase)
}

#[cfg(test)]
mod tests {
    use super::{Bip39Frame, English};
    use nwords_core::{BitFrame, Error, SchemeFrame};

    #[test]
    fn english_known_vector_round_trips() {
        let codec = English::default();
        let entropy = [0u8; 16];
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

        assert_eq!(codec.encode_entropy(&entropy), Ok(phrase.into()));
        assert_eq!(codec.decode_phrase(phrase), Ok(entropy.to_vec()));
    }

    #[test]
    fn invalid_checksum_is_rejected() {
        let codec = English::default();
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon";

        assert_eq!(codec.decode_phrase(phrase), Err(Error::InvalidChecksum));
    }

    #[test]
    fn frame_rejects_invalid_entropy_length() {
        assert_eq!(
            Bip39Frame.frame(&[0; 15]),
            Err(Error::InvalidEntropyLength {
                got: 15,
                expected: &[16, 20, 24, 28, 32]
            })
        );
    }

    #[test]
    fn frame_rejects_invalid_bit_length() {
        let frame = BitFrame::from_bytes_with_bit_len(&[0][..], 8).expect("frame");

        assert!(matches!(
            Bip39Frame.unframe(frame),
            Err(Error::InvalidBitLength { got: 8, .. })
        ));
    }

    #[cfg(feature = "bip39-japanese")]
    #[test]
    fn japanese_uses_ideographic_spaces_by_default() {
        let codec = super::Japanese::default();
        let entropy = [0u8; 16];
        let phrase = codec.encode_entropy(&entropy).expect("phrase");

        assert!(phrase.contains('\u{3000}'));
        assert_eq!(codec.decode_phrase(&phrase), Ok(entropy.to_vec()));
    }
}
