//! `radix-bytes-v1`: eight-byte blocks plus a length-bearing variable tail.
//!
//! Word positions cycle through the supplied ordered template continuously.
//! Full blocks use the least words whose mixed capacity covers 2^64. A mandatory
//! tail ranks byte strings of length 0–7 shortest first, then ranks that integer
//! across phrase lengths shortest first. Empty input therefore has one word.
use alloc::{string::String, vec::Vec};
use core::fmt;
use nwords_core::{Error, Formatter, WordMap};

/// Byte-format failure without exposing decoded payload values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RadixBytesError {
    /// A cycle needs 1–32 lists of 2 through u32::MAX entries.
    InvalidTemplate,
    /// Payload or phrase exceeds the configured resource bound.
    LimitExceeded,
    /// A block or tail is outside its canonical range.
    NonCanonical {
        /// Starting word position of the invalid block or tail.
        position: usize,
    },
    /// Underlying lookup or UTF-8 validation failure.
    Codec(Error),
}
impl From<Error> for RadixBytesError {
    fn from(error: Error) -> Self {
        Self::Codec(error)
    }
}
impl fmt::Display for RadixBytesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTemplate => f.write_str("invalid byte wordlist template"),
            Self::LimitExceeded => f.write_str("byte codec resource limit exceeded"),
            Self::NonCanonical { position } => {
                write!(f, "noncanonical byte block at word {position}")
            }
            Self::Codec(error) => error.fmt(f),
        }
    }
}
impl core::error::Error for RadixBytesError {}
/// Result of an arbitrary-wordset byte operation.
pub type Result<T> = core::result::Result<T, RadixBytesError>;

// Private block-size parameter allows independent exhaustive tests at toy sizes.
#[derive(Debug, Clone)]
struct Layout {
    radices: Vec<u32>,
    widths: Vec<usize>,
    block_bytes: usize,
}
impl Layout {
    fn new(radices: Vec<u32>, block_bytes: usize) -> Result<Self> {
        if radices.is_empty()
            || radices.len() > 32
            || radices.iter().any(|n| *n < 2)
            || !(2..=8).contains(&block_bytes)
        {
            return Err(RadixBytesError::InvalidTemplate);
        }
        let mut layout = Self {
            radices,
            widths: Vec::new(),
            block_bytes,
        };
        for start in 0..layout.radices.len() {
            let mut cap = 1u128;
            let mut width = 0;
            while cap < layout.block_range() {
                cap *= u128::from(layout.radix(start + width));
                width += 1;
            }
            layout.widths.push(width);
        }
        Ok(layout)
    }
    fn radix(&self, p: usize) -> u32 {
        self.radices[p % self.radices.len()]
    }
    fn width(&self, p: usize) -> usize {
        self.widths[p % self.widths.len()]
    }
    fn block_range(&self) -> u128 {
        1u128 << (self.block_bytes * 8)
    }
    fn max_words(&self, length: usize) -> Result<usize> {
        // A safe bound in O(cycle length), even for a caller's huge max_bytes.
        // Actual full-block widths may vary with the cycle phase.
        let full = (length / self.block_bytes)
            .checked_mul(self.widths.iter().copied().max().unwrap_or(0))
            .ok_or(RadixBytesError::LimitExceeded)?;
        let largest_tail = shorter(length % self.block_bytes + 1) - 1;
        let tail = (0..self.radices.len())
            .map(|p| self.tail_tier(largest_tail, p).0)
            .max()
            .unwrap_or(1);
        full.checked_add(tail).ok_or(RadixBytesError::LimitExceeded)
    }
    fn tail_tier(&self, mut u: u64, p: usize) -> (usize, u64) {
        let mut count = 1;
        let mut cap = u128::from(self.radix(p));
        while u128::from(u) >= cap {
            u -= cap as u64;
            cap *= u128::from(self.radix(p + count));
            count += 1;
        }
        (count, u)
    }
    fn fixed(&self, mut value: u64, p: usize, count: usize, out: &mut Vec<u32>) {
        let begin = out.len();
        out.resize(begin + count, 0);
        for i in (0..count).rev() {
            let radix = u64::from(self.radix(p + i));
            out[begin + i] = (value % radix) as u32;
            value /= radix;
        }
    }
    fn encode(&self, payload: &[u8]) -> Vec<u32> {
        let mut out = Vec::new();
        let mut blocks = payload.chunks_exact(self.block_bytes);
        for block in &mut blocks {
            let value = block.iter().fold(0u64, |v, b| (v << 8) | u64::from(*b));
            let p = out.len();
            self.fixed(value, p, self.width(p), &mut out);
        }
        let tail = blocks.remainder();
        let value = tail.iter().fold(0u64, |v, b| (v << 8) | u64::from(*b)) + shorter(tail.len());
        let p = out.len();
        let (count, local) = self.tail_tier(value, p);
        self.fixed(local, p, count, &mut out);
        out
    }
    fn rank(&self, symbols: &[u32], p: usize) -> Result<u128> {
        let mut value = 0u128;
        for (i, symbol) in symbols.iter().enumerate() {
            let radix = self.radix(p + i);
            if *symbol >= radix {
                return Err(Error::SymbolOutOfRange {
                    index: *symbol,
                    position: p + i,
                    len: radix as usize,
                }
                .into());
            }
            value = value * u128::from(radix) + u128::from(*symbol);
        }
        Ok(value)
    }
    fn decode(&self, symbols: &[u32], max_bytes: usize) -> Result<Vec<u8>> {
        if symbols.is_empty() {
            return Err(RadixBytesError::NonCanonical { position: 0 });
        }
        if symbols.len() > self.max_words(max_bytes)? {
            return Err(RadixBytesError::LimitExceeded);
        }
        let mut p = 0;
        let mut out = Vec::new();
        while symbols.len() - p > self.width(p) {
            let width = self.width(p);
            let value = self.rank(&symbols[p..p + width], p)?;
            if value >= self.block_range() {
                return Err(RadixBytesError::NonCanonical { position: p });
            }
            if out
                .len()
                .checked_add(self.block_bytes)
                .is_none_or(|n| n > max_bytes)
            {
                return Err(RadixBytesError::LimitExceeded);
            }
            out.extend_from_slice(&(value as u64).to_be_bytes()[8 - self.block_bytes..]);
            p += width;
        }
        let tail = &symbols[p..];
        let mut offset = 0u128;
        let mut cap = 1u128;
        for i in 0..tail.len() {
            if i > 0 {
                offset += cap;
            }
            cap *= u128::from(self.radix(p + i));
        }
        let u = offset + self.rank(tail, p)?;
        if u >= u128::from(shorter(self.block_bytes)) {
            return Err(RadixBytesError::NonCanonical { position: p });
        }
        let mut length = 0;
        while length + 1 < self.block_bytes && u >= u128::from(shorter(length + 1)) {
            length += 1;
        }
        if out.len().checked_add(length).is_none_or(|n| n > max_bytes) {
            return Err(RadixBytesError::LimitExceeded);
        }
        let value = (u - u128::from(shorter(length))) as u64;
        out.extend_from_slice(&value.to_be_bytes()[8 - length..]);
        Ok(out)
    }
}
fn shorter(length: usize) -> u64 {
    let mut sum = 0;
    for _ in 0..length {
        sum = sum * 256 + 1;
    }
    sum
}

/// Reusable reversible-byte facade over an arbitrary cyclic word map.
#[derive(Debug, Clone)]
pub struct RadixBytes<W> {
    map: W,
    layout: Layout,
    max_bytes: usize,
}
impl<W: WordMap> RadixBytes<W> {
    /// Constructs a 1–32-list cycle. `max_bytes` is an acceptance bound, not a
    /// mapping input. Word order and list order must be preserved for decoding.
    pub fn new(map: W, positions: usize, max_bytes: usize) -> Result<Self> {
        if positions == 0 || positions > 32 {
            return Err(RadixBytesError::InvalidTemplate);
        }
        let mut radices = Vec::new();
        for p in 0..positions {
            radices.push(u32::try_from(map.len(p)).map_err(|_| RadixBytesError::InvalidTemplate)?);
        }
        let layout = Layout::new(radices, 8)?;
        layout.max_words(max_bytes)?;
        Ok(Self {
            map,
            layout,
            max_bytes,
        })
    }
    /// Full block payload size (eight bytes); every message also has a tail.
    pub const fn block_bytes(&self) -> usize {
        8
    }
    /// Fewest words in a full block at any cycle position.
    pub fn min_block_words(&self) -> usize {
        self.layout.widths.iter().copied().min().unwrap_or(0)
    }
    /// Most words in a full block at any cycle position.
    pub fn max_block_words(&self) -> usize {
        self.layout.widths.iter().copied().max().unwrap_or(0)
    }
    /// Maximum accepted payload length.
    pub const fn max_bytes(&self) -> usize {
        self.max_bytes
    }
    /// Maximum output words for payloads of this length; tail size can depend on
    /// its value. This is not necessarily the exact encoded word count.
    pub fn max_word_count(&self, length: usize) -> Result<usize> {
        if length > self.max_bytes {
            return Err(RadixBytesError::LimitExceeded);
        }
        self.layout.max_words(length)
    }
    /// Encodes empty, odd-length and zero-prefixed data without losing bytes.
    pub fn encode_bytes(&self, payload: &[u8]) -> Result<String> {
        if payload.len() > self.max_bytes {
            return Err(RadixBytesError::LimitExceeded);
        }
        let symbols = self.layout.encode(payload);
        let mut words = Vec::with_capacity(symbols.len());
        for (position, index) in symbols.into_iter().enumerate() {
            words.push(
                self.map
                    .word(index as usize, position % self.layout.radices.len())
                    .ok_or(Error::SymbolOutOfRange {
                        index,
                        position,
                        len: self.layout.radix(position) as usize,
                    })?,
            );
        }
        Ok(crate::AsciiSpace.join(&words))
    }
    /// Decodes exact tokens, rejecting out-of-range blocks and tails. No checksum
    /// is implied: some substitutions or deletions remain valid byte encodings.
    pub fn decode_words(&self, words: &[&str]) -> Result<Vec<u8>> {
        if words.len() > self.max_word_count(self.max_bytes)? {
            return Err(RadixBytesError::LimitExceeded);
        }
        let mut symbols = Vec::with_capacity(words.len());
        for (position, word) in words.iter().enumerate() {
            let index = self
                .map
                .index_of(word, position % self.layout.radices.len())
                .ok_or(Error::UnknownWord { position })?;
            let index = u32::try_from(index).map_err(|_| Error::SymbolOutOfRange {
                index: u32::MAX,
                position,
                len: self.layout.radix(position) as usize,
            })?;
            symbols.push(index);
        }
        self.layout.decode(&symbols, self.max_bytes)
    }
    /// Encodes exact UTF-8 without normalization.
    pub fn encode_text(&self, text: &str) -> Result<String> {
        self.encode_bytes(text.as_bytes())
    }
    /// Decodes and validates UTF-8 without normalization.
    pub fn decode_text(&self, words: &[&str]) -> Result<String> {
        String::from_utf8(self.decode_words(words)?).map_err(|_| Error::InvalidUtf8.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    struct Alphabet(Vec<Vec<String>>);
    impl WordMap for Alphabet {
        fn len(&self, p: usize) -> usize {
            self.0.get(p).map_or(0, Vec::len)
        }
        fn word(&self, i: usize, p: usize) -> Option<&str> {
            self.0.get(p)?.get(i).map(String::as_str)
        }
        fn index_of(&self, w: &str, p: usize) -> Option<usize> {
            self.0.get(p)?.iter().position(|x| x == w)
        }
    }
    fn hex(input: &str) -> Vec<u8> {
        if input == "-" {
            return vec![];
        }
        input
            .as_bytes()
            .chunks_exact(2)
            .map(|pair| u8::from_str_radix(core::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect()
    }
    #[test]
    fn frozen_eff_vectors_from_independent_oracle() {
        let words = include_str!("../../../tests/vectors/eff-long/words.txt")
            .lines()
            .map(String::from)
            .collect();
        let codec = RadixBytes::new(Alphabet(vec![words]), 1, 4096).unwrap();
        for row in include_str!("../../../tests/vectors/radix-bytes/eff-long.tsv")
            .lines()
            .filter(|row| !row.starts_with('#'))
        {
            let (input, phrase) = row.split_once('\t').unwrap();
            let data = hex(input);
            assert_eq!(codec.encode_bytes(&data).unwrap(), phrase);
            assert_eq!(
                codec
                    .decode_words(&phrase.split_whitespace().collect::<Vec<_>>())
                    .unwrap(),
                data
            );
        }
        assert!(codec.decode_words(&["zoom"; 5]).is_err());
        assert!(codec
            .decode_words(&["zoom", "zoom", "zoom", "zoom", "zoom", "abacus"])
            .is_err());
        assert_eq!(codec.encode_bytes(&[]).unwrap(), "abacus");
        assert_eq!(
            codec
                .encode_bytes(&[0; 16])
                .unwrap()
                .split_whitespace()
                .count(),
            11
        );
        let phrase = codec.encode_bytes(&[255]).unwrap();
        assert_eq!(
            codec.decode_text(&phrase.split_whitespace().collect::<Vec<_>>()),
            Err(Error::InvalidUtf8.into())
        );
        assert_ne!(
            codec.encode_text("é").unwrap(),
            codec.encode_text("e\u{301}").unwrap()
        );
    }
    #[test]
    fn mixed_cycles_boundaries_and_long_round_trips() {
        for cycle in [
            vec![2],
            vec![3],
            vec![7776],
            vec![65536],
            vec![u32::MAX],
            vec![3, 300],
            vec![17, 255, 2],
        ] {
            let layout = Layout::new(cycle, 8).unwrap();
            for n in (0..40).chain([255, 256, 1024, 4095, 4096]) {
                for bytes in [
                    vec![0; n],
                    vec![255; n],
                    (0..n).map(|i| (i * 97) as u8).collect(),
                ] {
                    let encoded = layout.encode(&bytes);
                    assert!(encoded.len() <= layout.max_words(n).unwrap());
                    assert_eq!(layout.decode(&encoded, n).unwrap(), bytes);
                    if n > 0 {
                        assert!(layout.decode(&encoded, n - 1).is_err());
                    }
                }
            }
        }
        let layout = Layout::new(vec![3, 300], 8).unwrap();
        assert_ne!(layout.width(0), layout.width(1));
        assert!(layout.decode(&[], 4096).is_err());
        assert!(layout.decode(&[3], 4096).is_err());
        assert!(layout.max_words(usize::MAX).is_err());
        assert!(
            RadixBytes::new(Alphabet(vec![vec!["a".into(), "b".into()]]), 1, usize::MAX).is_err()
        );
    }
    #[test]
    fn exhaustive_toy_cycle_bijection_both_directions() {
        // 3,243,603 phrases; full-block slack, both tail tiers and both directions.
        let layout = Layout::new(vec![3, 300], 2).unwrap();
        let mut counts = [0usize; 3];
        for count in 1..=5 {
            let capacity = (0..count)
                .map(|p| u64::from(layout.radix(p)))
                .product::<u64>();
            for value in 0..capacity {
                let mut symbols = Vec::new();
                layout.fixed(value, 0, count, &mut symbols);
                if let Ok(bytes) = layout.decode(&symbols, 3) {
                    assert_eq!(layout.encode(&bytes), symbols);
                    if bytes.len() < counts.len() {
                        counts[bytes.len()] += 1;
                    }
                }
            }
        }
        assert_eq!(counts, [1, 256, 65536]);
        for value in 0u32..65536 {
            let bytes = (value as u16).to_be_bytes();
            let symbols = layout.encode(&bytes);
            assert_eq!(layout.decode(&symbols, 2).unwrap(), bytes);
        }
    }
}
