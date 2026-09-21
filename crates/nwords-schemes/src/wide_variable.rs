//! Arbitrary-width `variable-v1` with exact bit, byte, and UTF-8 views.
//!
//! The existing `u128` codec remains available. This codec uses the same
//! shortest-tier ordering and accepts IDs that need more than 128 bits.

use alloc::{string::String, vec, vec::Vec};
use core::{cmp::Ordering, fmt};
use nwords_core::WordMap;

/// A checked wide-variable operation failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WideVariableError {
    /// The pattern needs a repeated list and at least one suffix list.
    InvalidPattern,
    /// A word bound or accepted range is inconsistent with the shape.
    InvalidBounds,
    /// A list has fewer than two words or more than `u32::MAX` words.
    InvalidDictionary {
        /// Position of the invalid list in the pattern.
        position: usize,
    },
    /// A decimal ID was not a canonical nonnegative integer.
    InvalidInteger,
    /// A canonical decimal ID exceeds the parser's 10,000-digit limit.
    IntegerTooLong,
    /// An ID or phrase exceeds the accepted range.
    OutOfRange,
    /// A phrase has too few or too many words.
    InvalidWordCount {
        /// Number of supplied words.
        got: usize,
    },
    /// A word is absent from the list at its position.
    UnknownWord {
        /// Zero-based phrase position.
        position: usize,
    },
    /// A map did not return the requested word or index.
    InvalidWordMap {
        /// Zero-based phrase position.
        position: usize,
    },
    /// A bit view contains a nonbinary character or exceeds 4096 bits.
    InvalidBits,
    /// A byte view exceeds 512 bytes.
    InvalidBytes,
    /// A phrase's bit view does not contain whole bytes.
    NotByteAligned,
    /// A phrase's bytes are not valid UTF-8.
    InvalidUtf8,
}

impl fmt::Display for WideVariableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPattern => f.write_str("invalid leading-repeat pattern"),
            Self::InvalidBounds => f.write_str("invalid variable-v1 bounds"),
            Self::InvalidDictionary { position } => write!(f, "invalid dictionary at {position}"),
            Self::InvalidInteger => f.write_str("expected a canonical nonnegative integer"),
            Self::IntegerTooLong => f.write_str("decimal integer exceeds 10,000 digits"),
            Self::OutOfRange => f.write_str("ID is outside the accepted range"),
            Self::InvalidWordCount { got } => write!(f, "invalid word count: {got}"),
            Self::UnknownWord { position } => write!(f, "unknown word at {position}"),
            Self::InvalidWordMap { position } => write!(f, "invalid word map at {position}"),
            Self::InvalidBits => f.write_str("invalid or oversized bit view"),
            Self::InvalidBytes => f.write_str("oversized byte view"),
            Self::NotByteAligned => f.write_str("bit view is not byte-aligned"),
            Self::InvalidUtf8 => f.write_str("decoded bytes are not UTF-8"),
        }
    }
}

impl core::error::Error for WideVariableError {}

/// Result of an arbitrary-width variable-v1 operation.
pub type Result<T> = core::result::Result<T, WideVariableError>;

/// A nonnegative integer represented by little-endian 32-bit limbs.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WideId {
    limbs: Vec<u32>,
}

impl Ord for WideId {
    fn cmp(&self, other: &Self) -> Ordering {
        self.compare(other)
    }
}

impl PartialOrd for WideId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl WideId {
    /// Returns zero.
    pub fn zero() -> Self {
        Self::default()
    }

    /// Converts a nonnegative fixed-width integer.
    pub fn from_u128(value: u128) -> Self {
        Self::from_be_bytes(&value.to_be_bytes())
    }

    /// Parses canonical decimal notation, including `0`, through 10,000 digits.
    pub fn parse_decimal(text: &str) -> Result<Self> {
        if text.len() > 10_000 {
            return Err(WideVariableError::IntegerTooLong);
        }
        if text.is_empty()
            || (text.len() > 1 && text.starts_with('0'))
            || !text.bytes().all(|byte| byte.is_ascii_digit())
        {
            return Err(WideVariableError::InvalidInteger);
        }
        let mut value = Self::zero();
        for digit in text.bytes() {
            value.mul_small(10);
            value.add_small(u32::from(digit - b'0'));
        }
        Ok(value)
    }

    /// Converts unsigned big-endian bytes, preserving their numerical value.
    pub fn from_be_bytes(bytes: &[u8]) -> Self {
        let mut value = Self::zero();
        for byte in bytes {
            value.mul_small(256);
            value.add_small(u32::from(*byte));
        }
        value
    }

    /// Returns minimal unsigned big-endian bytes; zero is an empty slice.
    pub fn to_be_bytes(&self) -> Vec<u8> {
        let mut value = self.clone();
        let mut bytes = Vec::new();
        while !value.is_zero() {
            bytes.push(value.div_rem_small(256) as u8);
        }
        bytes.reverse();
        bytes
    }

    /// Returns canonical decimal notation.
    pub fn to_decimal_string(&self) -> String {
        if self.is_zero() {
            return String::from("0");
        }
        let mut value = self.clone();
        let mut digits = Vec::new();
        while !value.is_zero() {
            digits.push(b'0' + value.div_rem_small(10) as u8);
        }
        digits.reverse();
        digits.into_iter().map(char::from).collect()
    }

    /// Returns the number of significant bits, or zero for zero.
    pub fn bit_len(&self) -> usize {
        self.limbs.last().map_or(0, |top| {
            (self.limbs.len() - 1) * 32 + (32 - top.leading_zeros()) as usize
        })
    }

    /// Maps an exact bitstring `b` to `int('1' + b) - 1`.
    pub fn from_bit_view(bits: &str) -> Result<Self> {
        if bits.len() > 4096 || !bits.bytes().all(|byte| byte == b'0' || byte == b'1') {
            return Err(WideVariableError::InvalidBits);
        }
        let mut value = Self::from_u128(1);
        for bit in bits.bytes() {
            value.mul_small(2);
            value.add_small(u32::from(bit - b'0'));
        }
        value.sub_one();
        Ok(value)
    }

    /// Recovers the exact bitstring, including leading zeros.
    pub fn to_bit_view(&self) -> Result<String> {
        let mut value = self.clone();
        value.add_small(1);
        if value.bit_len() > 4097 {
            return Err(WideVariableError::InvalidBits);
        }
        let mut bits = String::new();
        for byte in value.to_be_bytes() {
            for shift in (0..8).rev() {
                bits.push(if (byte >> shift) & 1 == 0 { '0' } else { '1' });
            }
        }
        let significant = bits.trim_start_matches('0');
        let view = &significant[1..];
        Ok(String::from(view))
    }

    fn is_zero(&self) -> bool {
        self.limbs.is_empty()
    }

    fn compare(&self, other: &Self) -> Ordering {
        match self.limbs.len().cmp(&other.limbs.len()) {
            Ordering::Equal => self.limbs.iter().rev().cmp(other.limbs.iter().rev()),
            different => different,
        }
    }

    fn normalize(&mut self) {
        while self.limbs.last() == Some(&0) {
            self.limbs.pop();
        }
    }

    fn add_small(&mut self, amount: u32) {
        let mut carry = u64::from(amount);
        let mut index = 0;
        while carry != 0 {
            if index == self.limbs.len() {
                self.limbs.push(0);
            }
            let sum = u64::from(self.limbs[index]) + carry;
            self.limbs[index] = sum as u32;
            carry = sum >> 32;
            index += 1;
        }
    }

    fn add_assign(&mut self, other: &Self) {
        let mut carry = 0u64;
        let count = self.limbs.len().max(other.limbs.len());
        self.limbs.resize(count, 0);
        for index in 0..count {
            let sum = u64::from(self.limbs[index])
                + u64::from(other.limbs.get(index).copied().unwrap_or(0))
                + carry;
            self.limbs[index] = sum as u32;
            carry = sum >> 32;
        }
        if carry != 0 {
            self.limbs.push(carry as u32);
        }
    }

    fn sub_one(&mut self) {
        debug_assert!(!self.is_zero());
        for limb in &mut self.limbs {
            if *limb != 0 {
                *limb -= 1;
                self.normalize();
                return;
            }
            *limb = u32::MAX;
        }
    }

    fn mul_small(&mut self, factor: u32) {
        if factor == 0 {
            self.limbs.clear();
            return;
        }
        let mut carry = 0u64;
        for limb in &mut self.limbs {
            let product = u64::from(*limb) * u64::from(factor) + carry;
            *limb = product as u32;
            carry = product >> 32;
        }
        if carry != 0 {
            self.limbs.push(carry as u32);
        }
    }

    fn div_rem_small(&mut self, divisor: u32) -> u32 {
        let mut remainder = 0u64;
        for limb in self.limbs.iter_mut().rev() {
            let value = (remainder << 32) | u64::from(*limb);
            *limb = (value / u64::from(divisor)) as u32;
            remainder = value % u64::from(divisor);
        }
        self.normalize();
        remainder as u32
    }
}

/// A prepared arbitrary-width `variable-v1` codec over an ordered word map.
#[derive(Debug, Clone)]
pub struct WideVariablePositional<W> {
    map: W,
    suffix_words: usize,
    minimum: usize,
    range: WideId,
    max_words: usize,
    capacity: WideId,
}

impl<W: WordMap> WideVariablePositional<W> {
    /// Validates the shape and computes exact capacity through `max_words`.
    ///
    /// Rust supports up to 1024 total words. `range` is an optional exclusive
    /// upper bound; omitting it accepts the entire word-bounded capacity.
    pub fn new(
        map: W,
        suffix_words: usize,
        minimum: usize,
        range: Option<WideId>,
        max_words: usize,
    ) -> Result<Self> {
        if suffix_words == 0 || suffix_words > 1023 || minimum > 1 {
            return Err(WideVariableError::InvalidPattern);
        }
        if max_words < suffix_words + minimum || max_words > 1024 {
            return Err(WideVariableError::InvalidBounds);
        }
        for position in 0..=suffix_words {
            let size = map.len(position);
            if !(2..=u32::MAX as usize).contains(&size) {
                return Err(WideVariableError::InvalidDictionary { position });
            }
        }
        let base = map.len(0) as u32;
        let mut tier = WideId::from_u128(1);
        for position in 1..=suffix_words {
            tier.mul_small(map.len(position) as u32);
        }
        if minimum == 1 {
            tier.mul_small(base);
        }
        let mut capacity = WideId::zero();
        for words in suffix_words + minimum..=max_words {
            capacity.add_assign(&tier);
            if words < max_words {
                tier.mul_small(base);
            }
        }
        let range = range.unwrap_or_else(|| capacity.clone());
        if range.is_zero() || range.compare(&capacity) == Ordering::Greater {
            return Err(WideVariableError::InvalidBounds);
        }
        Ok(Self {
            map,
            suffix_words,
            minimum,
            range,
            max_words,
            capacity,
        })
    }

    /// Returns the exclusive accepted range.
    pub fn range(&self) -> &WideId {
        &self.range
    }

    /// Returns exact word-bounded capacity.
    pub fn capacity(&self) -> &WideId {
        &self.capacity
    }

    /// Returns the maximum total phrase length.
    pub fn max_words(&self) -> usize {
        self.max_words
    }

    /// Returns the maximum words needed inside the accepted range.
    pub fn required_words(&self) -> usize {
        let mut value = self.range.clone();
        value.sub_one();
        for position in (1..=self.suffix_words).rev() {
            value.div_rem_small(self.map.len(position) as u32);
        }
        if self.minimum == 1 {
            value.div_rem_small(self.map.len(0) as u32);
        }
        let mut count = self.suffix_words + self.minimum;
        let base = self.map.len(0) as u32;
        while !value.is_zero() {
            value.sub_one();
            value.div_rem_small(base);
            count += 1;
        }
        count
    }

    /// Encodes an integer using the published shortest-tier mapping.
    pub fn encode(&self, id: &WideId) -> Result<String> {
        if id.compare(&self.range) != Ordering::Less {
            return Err(WideVariableError::OutOfRange);
        }
        let mut quotient = id.clone();
        let mut tail = vec![0usize; self.suffix_words + self.minimum];
        for position in (1..=self.suffix_words).rev() {
            tail[position - 1 + self.minimum] =
                quotient.div_rem_small(self.map.len(position) as u32) as usize;
        }
        let base = self.map.len(0) as u32;
        if self.minimum == 1 {
            tail[0] = quotient.div_rem_small(base) as usize;
        }
        let mut prefix = Vec::new();
        while !quotient.is_zero() {
            quotient.sub_one();
            prefix.push(quotient.div_rem_small(base) as usize);
        }
        if prefix.len() + tail.len() > self.max_words {
            return Err(WideVariableError::OutOfRange);
        }
        let mut words = Vec::with_capacity(prefix.len() + tail.len());
        for index in prefix.into_iter().rev() {
            words.push(self.word(index, 0, words.len())?);
        }
        for (position, index) in tail.into_iter().enumerate() {
            let slot = if position < self.minimum {
                0
            } else {
                position - self.minimum + 1
            };
            words.push(self.word(index, slot, words.len())?);
        }
        Ok(words.join(" "))
    }

    /// Decodes already split words and rejects slack above `range`.
    pub fn decode_words(&self, words: &[&str]) -> Result<WideId> {
        if words.len() < self.suffix_words + self.minimum || words.len() > self.max_words {
            return Err(WideVariableError::InvalidWordCount { got: words.len() });
        }
        let prefix = words.len() - self.suffix_words - self.minimum;
        let mut value = WideId::zero();
        for (position, word) in words.iter().enumerate() {
            let slot = if position < prefix + self.minimum {
                0
            } else {
                position - prefix - self.minimum + 1
            };
            let base = self.map.len(slot);
            let index = self
                .map
                .index_of(word, slot)
                .ok_or(WideVariableError::UnknownWord { position })?;
            if index >= base {
                return Err(WideVariableError::InvalidWordMap { position });
            }
            value.mul_small(base as u32);
            value.add_small(index as u32 + u32::from(position < prefix));
        }
        if value.compare(&self.range) != Ordering::Less {
            return Err(WideVariableError::OutOfRange);
        }
        Ok(value)
    }

    /// Splits phrase whitespace and decodes the resulting tokens.
    pub fn decode_phrase(&self, phrase: &str) -> Result<WideId> {
        let count = phrase.split_whitespace().count();
        if count > self.max_words {
            return Err(WideVariableError::InvalidWordCount { got: count });
        }
        let words = phrase.split_whitespace().collect::<Vec<_>>();
        self.decode_words(&words)
    }

    /// Encodes an exact finite bitstring, including its leading zeros.
    pub fn encode_bits(&self, bits: &str) -> Result<String> {
        self.encode(&WideId::from_bit_view(bits)?)
    }

    /// Decodes a phrase as an exact finite bitstring.
    pub fn decode_bits(&self, phrase: &str) -> Result<String> {
        self.decode_phrase(phrase)?.to_bit_view()
    }

    /// Encodes up to 512 bytes through the exact bit view.
    pub fn encode_bytes(&self, bytes: &[u8]) -> Result<String> {
        if bytes.len() > 512 {
            return Err(WideVariableError::InvalidBytes);
        }
        let mut bits = String::with_capacity(bytes.len() * 8);
        for byte in bytes {
            for shift in (0..8).rev() {
                bits.push(if (byte >> shift) & 1 == 0 { '0' } else { '1' });
            }
        }
        self.encode_bits(&bits)
    }

    /// Decodes a phrase as whole bytes, retaining leading zero bytes.
    pub fn decode_bytes(&self, phrase: &str) -> Result<Vec<u8>> {
        let bits = self.decode_bits(phrase)?;
        if bits.len() % 8 != 0 {
            return Err(WideVariableError::NotByteAligned);
        }
        Ok(bits
            .as_bytes()
            .chunks(8)
            .map(|chunk| {
                chunk
                    .iter()
                    .fold(0u8, |value, bit| (value << 1) | (bit - b'0'))
            })
            .collect())
    }

    /// Encodes UTF-8 bytes without normalization.
    pub fn encode_text(&self, text: &str) -> Result<String> {
        self.encode_bytes(text.as_bytes())
    }

    /// Decodes UTF-8 without normalization.
    pub fn decode_text(&self, phrase: &str) -> Result<String> {
        String::from_utf8(self.decode_bytes(phrase)?).map_err(|_| WideVariableError::InvalidUtf8)
    }

    fn word(&self, index: usize, slot: usize, position: usize) -> Result<&str> {
        self.map
            .word(index, slot)
            .ok_or(WideVariableError::InvalidWordMap { position })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decimal_and_bit_roundtrips() {
        for decimal in [
            "0",
            "1",
            "340282366920938463463374607431768211455",
            "99999999999999999999999999999999999999999999999999999999999999999",
        ] {
            let id = WideId::parse_decimal(decimal).unwrap();
            assert_eq!(id.to_decimal_string(), decimal);
            assert_eq!(WideId::from_be_bytes(&id.to_be_bytes()), id);
            assert_eq!(
                WideId::from_bit_view(&id.to_bit_view().unwrap()).unwrap(),
                id
            );
        }
        for bits in ["", "0", "00000000", "01011", &"1".repeat(255)] {
            assert_eq!(
                WideId::from_bit_view(bits).unwrap().to_bit_view().unwrap(),
                bits
            );
        }
        for bad in ["", "00", "+1", "-1", "01", "1.0"] {
            assert!(WideId::parse_decimal(bad).is_err());
        }
        assert_eq!(
            WideId::parse_decimal(&"9".repeat(10_001)),
            Err(WideVariableError::IntegerTooLong)
        );
        assert!(WideId::from_u128(1) < WideId::from_u128(u128::MAX));
        assert_eq!(
            WideId::parse_decimal(&"9".repeat(2000))
                .unwrap()
                .to_bit_view(),
            Err(WideVariableError::InvalidBits)
        );
    }
}
