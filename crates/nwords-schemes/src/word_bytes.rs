use alloc::{string::String, vec::Vec};

use nwords_core::{
    BigEndian11Bit, BitFrame, BitView, Error, Formatter, Result, SymbolCodec, WordMap,
};

use crate::AsciiSpace;

const LENGTH_PREFIX_BYTES: usize = 4;
const LENGTH_PREFIX_BITS: usize = LENGTH_PREFIX_BYTES * 8;
const SYMBOL_BITS: usize = 11;
const WORD_BYTES_V1_DICTIONARY_SIZE: usize = 2048;

/// Byte-oriented word phrase codec.
///
/// `WordBytes` frames arbitrary bytes as `word-bytes-v1`:
///
/// ```text
/// 32-bit big-endian byte length + payload bytes + zero pad bits
/// ```
///
/// The padded bit frame is encoded as 11-bit symbols and mapped through a
/// 2048-word dictionary. This is not a BIP-39 mnemonic.
#[derive(Debug, Clone)]
pub struct WordBytes<W, F = AsciiSpace> {
    word_map: W,
    formatter: F,
    symbol_codec: BigEndian11Bit,
}

impl<W> WordBytes<W, AsciiSpace>
where
    W: WordMap,
{
    /// Creates a byte-word codec using ASCII-space display.
    pub const fn new(word_map: W) -> Self {
        Self {
            word_map,
            formatter: AsciiSpace,
            symbol_codec: BigEndian11Bit,
        }
    }
}

impl<W, F> WordBytes<W, F>
where
    W: WordMap,
    F: Formatter,
{
    /// Creates a byte-word codec from explicit components.
    pub const fn with_formatter(word_map: W, formatter: F) -> Self {
        Self {
            word_map,
            formatter,
            symbol_codec: BigEndian11Bit,
        }
    }

    /// Encodes arbitrary bytes into a word phrase.
    pub fn encode_bytes(&self, payload: &[u8]) -> Result<String> {
        let frame = encode_frame(payload)?;
        let view = BitFrame::from_bytes_with_bit_len(frame.as_bytes(), frame.bit_len())?;
        let symbols = self.symbol_codec.pack(view)?;
        self.encode_symbols(&symbols)
    }

    /// Decodes already-split phrase words into bytes.
    pub fn decode_words(&self, words: &[&str]) -> Result<Vec<u8>> {
        let mut symbols = Vec::with_capacity(words.len());
        for (position, word) in words.iter().enumerate() {
            let symbol = self
                .word_map
                .index_of(word, position)
                .ok_or(Error::UnknownWord { position })?;
            if symbol >= WORD_BYTES_V1_DICTIONARY_SIZE {
                return Err(Error::SymbolOutOfRange {
                    index: u32::try_from(symbol).unwrap_or(u32::MAX),
                    position,
                    len: WORD_BYTES_V1_DICTIONARY_SIZE,
                });
            }
            symbols.push(symbol as u32);
        }

        let frame = self.symbol_codec.unpack(&symbols)?;
        let view = BitFrame::from_bytes_with_bit_len(frame.as_bytes(), frame.bit_len())?;
        decode_frame(view)
    }

    /// Encodes UTF-8 text as bytes into a word phrase.
    pub fn encode_text(&self, text: &str) -> Result<String> {
        self.encode_bytes(text.as_bytes())
    }

    /// Decodes words into UTF-8 text.
    pub fn decode_text(&self, words: &[&str]) -> Result<String> {
        String::from_utf8(self.decode_words(words)?).map_err(|_| Error::InvalidUtf8)
    }

    fn encode_symbols(&self, symbols: &[u32]) -> Result<String> {
        let mut words = Vec::with_capacity(symbols.len());
        for (position, symbol) in symbols.iter().copied().enumerate() {
            let len = self.word_map.len(position);
            if len != WORD_BYTES_V1_DICTIONARY_SIZE {
                return Err(Error::InconsistentDictionarySize {
                    position,
                    got: len,
                    expected: WORD_BYTES_V1_DICTIONARY_SIZE,
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
}

fn encode_frame(payload: &[u8]) -> Result<BitFrame> {
    let length = u32::try_from(payload.len()).map_err(|_| Error::LengthOverflow)?;
    let live_bits = payload_live_bits(payload.len())?;
    let padded_bits = live_bits
        .checked_add(pad_bits(live_bits))
        .ok_or(Error::BitFrameOverflow)?;
    let mut frame = BitFrame::with_bit_capacity(padded_bits);

    for byte in length.to_be_bytes() {
        frame.push_bits(u128::from(byte), 8)?;
    }
    for &byte in payload {
        frame.push_bits(u128::from(byte), 8)?;
    }
    while frame.bit_len() % SYMBOL_BITS != 0 {
        frame.push_bit(false)?;
    }

    Ok(frame)
}

fn decode_frame(frame: BitView<'_>) -> Result<Vec<u8>> {
    if frame.bit_len() % SYMBOL_BITS != 0 {
        return Err(Error::InvalidFrame {
            bit_len: frame.bit_len(),
        });
    }
    if frame.bit_len() < LENGTH_PREFIX_BITS {
        return Err(Error::LengthMismatch);
    }

    let length = usize::try_from(read_u32_be(&frame)?).map_err(|_| Error::LengthOverflow)?;
    let live_bits = payload_live_bits(length)?;
    if frame.bit_len() < live_bits {
        return Err(Error::LengthMismatch);
    }

    let expected_bits = live_bits
        .checked_add(pad_bits(live_bits))
        .ok_or(Error::BitFrameOverflow)?;
    if frame.bit_len() != expected_bits {
        if frame.bit_len() > expected_bits && all_zero_bits(&frame, live_bits, frame.bit_len())? {
            return Err(Error::NonCanonicalPadding);
        }
        return Err(Error::LengthMismatch);
    }
    if !all_zero_bits(&frame, live_bits, frame.bit_len())? {
        return Err(Error::NonCanonicalPadding);
    }

    let mut output = Vec::with_capacity(length);
    let mut bit_index = LENGTH_PREFIX_BITS;
    for _ in 0..length {
        output.push(read_u8(&frame, bit_index)?);
        bit_index += 8;
    }
    Ok(output)
}

fn payload_live_bits(length: usize) -> Result<usize> {
    length
        .checked_mul(8)
        .and_then(|bits| bits.checked_add(LENGTH_PREFIX_BITS))
        .ok_or(Error::LengthOverflow)
}

fn pad_bits(live_bits: usize) -> usize {
    (SYMBOL_BITS - (live_bits % SYMBOL_BITS)) % SYMBOL_BITS
}

fn read_u32_be(frame: &BitView<'_>) -> Result<u32> {
    let mut value = 0u32;
    for index in 0..LENGTH_PREFIX_BITS {
        value <<= 1;
        if frame.bit(index).ok_or(Error::BitFrameOverflow)? {
            value |= 1;
        }
    }
    Ok(value)
}

fn read_u8(frame: &BitView<'_>, start: usize) -> Result<u8> {
    let mut value = 0u8;
    for offset in 0..8 {
        value <<= 1;
        if frame.bit(start + offset).ok_or(Error::BitFrameOverflow)? {
            value |= 1;
        }
    }
    Ok(value)
}

fn all_zero_bits(frame: &BitView<'_>, start: usize, end: usize) -> Result<bool> {
    for index in start..end {
        if frame.bit(index).ok_or(Error::BitFrameOverflow)? {
            return Ok(false);
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::{decode_frame, encode_frame, WordBytes};
    use nwords_core::{BitFrame, Error};
    use nwords_wordlists::bip39::English;

    fn codec() -> WordBytes<English> {
        WordBytes::new(English)
    }

    fn words(phrase: &str) -> Vec<&str> {
        phrase.split_whitespace().collect()
    }

    #[test]
    fn vectors_round_trip_with_expected_phrases() {
        let cases: &[(&[u8], &str)] = &[
            (b"", "abandon abandon abandon"),
            (b"f", "abandon abandon able slot"),
            (b"fo", "abandon abandon above smoke useless"),
            (b"foo", "abandon abandon absorb smoke want length"),
            (b"hello", "abandon abandon access speak fine curtain rose"),
            (&[0x00], "abandon abandon able abandon"),
            (&[0xff], "abandon abandon about wrap"),
            (
                &[0xde, 0xad, 0xbe, 0xef],
                "abandon abandon abuse run swim jealous",
            ),
        ];

        let codec = codec();
        for &(payload, expected) in cases {
            let phrase = codec.encode_bytes(payload).expect("encode");
            assert_eq!(phrase, expected);
            assert_eq!(codec.decode_words(&words(&phrase)), Ok(payload.to_vec()));
        }
    }

    #[test]
    fn text_round_trips_without_normalization() {
        let codec = codec();
        for text in ["hello", "hello, 世界", "é", "e\u{301}"] {
            let phrase = codec.encode_text(text).expect("encode");
            assert_eq!(codec.decode_text(&words(&phrase)).as_deref(), Ok(text));
        }

        let precomposed = codec.encode_text("é").expect("precomposed");
        let decomposed = codec.encode_text("e\u{301}").expect("decomposed");
        assert_ne!(precomposed, decomposed);
    }

    #[test]
    fn unknown_words_are_sanitized() {
        let codec = codec();

        assert_eq!(
            codec.decode_words(&["abandon", "not-in-error", "abandon"]),
            Err(Error::UnknownWord { position: 1 })
        );
    }

    #[test]
    fn invalid_utf8_is_rejected_by_text_adapter() {
        let codec = codec();
        let phrase = codec.encode_bytes(&[0xff]).expect("encode");

        assert_eq!(codec.decode_text(&words(&phrase)), Err(Error::InvalidUtf8));
    }

    #[test]
    fn non_zero_pad_bits_are_rejected() {
        let frame = encode_frame(b"").expect("frame");
        let mut bytes = frame.into_bytes();
        bytes[4] |= 0b1000_0000;
        let view = BitFrame::from_bytes_with_bit_len(&bytes[..], 33).expect("view");

        assert_eq!(decode_frame(view), Err(Error::NonCanonicalPadding));
    }

    #[test]
    fn trailing_all_zero_word_is_rejected() {
        let mut frame = encode_frame(b"").expect("frame");
        for _ in 0..11 {
            frame.push_bit(false).expect("pad");
        }
        let view =
            BitFrame::from_bytes_with_bit_len(frame.as_bytes(), frame.bit_len()).expect("view");

        assert_eq!(decode_frame(view), Err(Error::NonCanonicalPadding));
    }

    #[test]
    fn declared_length_longer_than_payload_is_rejected() {
        let mut frame = BitFrame::new();
        frame.push_bits(2, 32).expect("length");
        frame.push_bits(u128::from(b'f'), 8).expect("payload");
        while frame.bit_len() % 11 != 0 {
            frame.push_bit(false).expect("pad");
        }
        let view =
            BitFrame::from_bytes_with_bit_len(frame.as_bytes(), frame.bit_len()).expect("view");

        assert_eq!(decode_frame(view), Err(Error::LengthMismatch));
    }

    #[test]
    fn declared_length_shorter_than_payload_is_rejected() {
        let mut frame = BitFrame::new();
        frame.push_bits(0, 32).expect("length");
        frame.push_bits(u128::from(b'f'), 8).expect("payload");
        while frame.bit_len() % 11 != 0 {
            frame.push_bit(false).expect("pad");
        }
        let view =
            BitFrame::from_bytes_with_bit_len(frame.as_bytes(), frame.bit_len()).expect("view");

        assert_eq!(decode_frame(view), Err(Error::LengthMismatch));
    }

    #[test]
    fn non_symbol_aligned_frame_is_rejected() {
        let view = BitFrame::from_bytes_with_bit_len(&[0, 0, 0, 0][..], 32).expect("view");

        assert_eq!(decode_frame(view), Err(Error::InvalidFrame { bit_len: 32 }));
    }
}
