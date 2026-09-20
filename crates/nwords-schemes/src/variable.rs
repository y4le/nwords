//! `variable-v1`: shortest tiers first, with one leading repeated list.
//!
//! Map position zero is the repeated list; positions 1 through `suffix_words`
//! are the fixed suffix. Minimum repetitions (zero or one) and exact list order
//! define the mapping. Range and maximum words only restrict acceptance.
use alloc::{string::String, vec, vec::Vec};
use core::fmt;
use nwords_core::{
    stats::{capacity_mixed, CapacityClass, Log2Estimate},
    Error, Formatter, WordMap,
};

/// Variable-pattern validation or conversion failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariableError {
    /// A nonempty suffix and minimum of zero or one are required.
    InvalidPattern,
    /// At least one of range and maximum words must be supplied.
    MissingBounds,
    /// Cumulative capacity needs an explicit representable range.
    CapacityOverflow,
    /// Word bound is too small, inconsistent with range, or exceeds 1024.
    InvalidMaxWords,
    /// Underlying lookup, arithmetic, or range failure.
    Codec(Error),
}
impl From<Error> for VariableError {
    fn from(error: Error) -> Self {
        Self::Codec(error)
    }
}
impl fmt::Display for VariableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPattern => f.write_str("invalid leading-repeat pattern"),
            Self::MissingBounds => f.write_str("range or maximum words is required"),
            Self::CapacityOverflow => f.write_str("capacity exceeds u128; specify range"),
            Self::InvalidMaxWords => f.write_str("invalid or insufficient maximum words"),
            Self::Codec(error) => error.fmt(f),
        }
    }
}
impl core::error::Error for VariableError {}
/// Result of a variable-pattern operation.
pub type Result<T> = core::result::Result<T, VariableError>;

/// An owned, reusable variable-length integer codec, without a permutation.
#[derive(Debug, Clone)]
pub struct VariablePositional<W> {
    map: W,
    suffix_words: usize,
    minimum: usize,
    range: u128,
    max_words: usize,
    capacity: Option<CapacityClass>,
}
impl<W: WordMap> VariablePositional<W> {
    /// Validates the pattern and acceptance bounds. No word normalization occurs.
    ///
    /// Rust limits total words to 1024. A range-only format reports unbounded
    /// mathematical capacity; its actual output must fit this implementation cap.
    pub fn new(
        map: W,
        suffix_words: usize,
        minimum: usize,
        range: Option<u128>,
        max_words: Option<usize>,
    ) -> Result<Self> {
        if suffix_words == 0 || suffix_words > 1023 || minimum > 1 {
            return Err(VariableError::InvalidPattern);
        }
        if range.is_none() && max_words.is_none() {
            return Err(VariableError::MissingBounds);
        }
        let max = max_words.unwrap_or(1024);
        if max < suffix_words + minimum || max > 1024 {
            return Err(VariableError::InvalidMaxWords);
        }
        for p in 0..=suffix_words {
            let size = map.len(p);
            if size < 2 || size > u32::MAX as usize {
                return Err(Error::InvalidDictionarySize { got: size }.into());
            }
        }
        let capacity = max_words
            .map(|max| cumulative(&map, suffix_words, minimum, max))
            .transpose()?;
        let range = match range {
            Some(0) => return Err(Error::InvalidRange { got: 0 }.into()),
            Some(value) => value,
            None => match capacity {
                Some(CapacityClass::Exact(value)) => value,
                _ => return Err(VariableError::CapacityOverflow),
            },
        };
        if let Some(CapacityClass::Exact(cap)) = capacity {
            if range > cap {
                return Err(Error::RangeExceedsCapacity {
                    range,
                    capacity: cap,
                }
                .into());
            }
        }
        let codec = Self {
            map,
            suffix_words,
            minimum,
            range,
            max_words: max,
            capacity,
        };
        if codec.words_required(range - 1) > max {
            return Err(VariableError::InvalidMaxWords);
        }
        Ok(codec)
    }
    /// Accepted exclusive upper bound; always positive and representable by u128.
    pub const fn range(&self) -> u128 {
        self.range
    }
    /// Cumulative capacity through an explicit word bound, or None for unbounded.
    pub const fn capacity(&self) -> Option<CapacityClass> {
        self.capacity
    }
    /// Most words needed for the accepted range.
    pub fn required_words(&self) -> usize {
        self.words_required(self.range - 1)
    }
    /// Active maximum total words (including the fixed suffix).
    pub const fn max_words(&self) -> usize {
        self.max_words
    }
    /// Encodes an ID using stable tier ordering, independently of range size.
    pub fn encode(&self, id: u128) -> Result<String> {
        if id >= self.range {
            return Err(Error::IndexOutOfRange {
                value: id,
                range: self.range,
            }
            .into());
        }
        let mut q = id;
        let mut tail = vec![0usize; self.suffix_words + self.minimum];
        for p in (1..=self.suffix_words).rev() {
            tail[p - 1 + self.minimum] = (q % self.map.len(p) as u128) as usize;
            q /= self.map.len(p) as u128;
        }
        let base = self.map.len(0) as u128;
        if self.minimum == 1 {
            tail[0] = (q % base) as usize;
            q /= base;
        }
        let mut prefix = Vec::new();
        while q > 0 {
            q -= 1;
            prefix.push((q % base) as usize);
            q /= base;
        }
        let count = prefix.len() + tail.len();
        if count > self.max_words {
            return Err(VariableError::InvalidMaxWords);
        }
        let mut words = Vec::with_capacity(count);
        for index in prefix.iter().rev().copied() {
            words.push(self.lookup(index, 0, words.len())?);
        }
        for (p, index) in tail.into_iter().enumerate() {
            let slot = if p < self.minimum {
                0
            } else {
                p - self.minimum + 1
            };
            words.push(self.lookup(index, slot, words.len())?);
        }
        Ok(crate::AsciiSpace.join(&words))
    }
    /// Decodes already split tokens, rejecting overflow, excess words, and slack.
    pub fn decode_words(&self, words: &[&str]) -> Result<u128> {
        if words.len() < self.suffix_words + self.minimum || words.len() > self.max_words {
            return Err(Error::InvalidWordCount { got: words.len() }.into());
        }
        let prefix = words.len() - self.suffix_words - self.minimum;
        let mut value = 0u128;
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
                .ok_or(Error::UnknownWord { position })?;
            if index >= base {
                return Err(Error::SymbolOutOfRange {
                    index: u32::try_from(index).unwrap_or(u32::MAX),
                    position,
                    len: base,
                }
                .into());
            }
            let digit = index as u128 + u128::from(position < prefix);
            value = value
                .checked_mul(base as u128)
                .and_then(|v| v.checked_add(digit))
                .ok_or(Error::BitFrameOverflow)?;
        }
        if value >= self.range {
            return Err(Error::IndexOutOfRange {
                value,
                range: self.range,
            }
            .into());
        }
        Ok(value)
    }
    fn lookup(&self, index: usize, slot: usize, position: usize) -> Result<&str> {
        self.map.word(index, slot).ok_or_else(|| {
            Error::SymbolOutOfRange {
                index: index as u32,
                position,
                len: self.map.len(slot),
            }
            .into()
        })
    }
    fn words_required(&self, mut id: u128) -> usize {
        for p in (1..=self.suffix_words).rev() {
            id /= self.map.len(p) as u128;
        }
        if self.minimum == 1 {
            id /= self.map.len(0) as u128;
        }
        let mut count = self.suffix_words + self.minimum;
        while id > 0 {
            id = (id - 1) / self.map.len(0) as u128;
            count += 1;
        }
        count
    }
}
fn cumulative<W: WordMap>(map: &W, suffix: usize, min: usize, max: usize) -> Result<CapacityClass> {
    let top_sizes = (0..max - suffix)
        .map(|_| map.len(0))
        .chain((1..=suffix).map(|p| map.len(p)))
        .collect::<Vec<_>>();
    // The geometric sum is at least the top tier and strictly less than twice it.
    let top = capacity_mixed(&top_sizes)?;
    let log2 = match top {
        CapacityClass::BeyondU128 { log2 } => Log2Estimate {
            lower_bits: log2.lower_bits,
            upper_bits: log2.upper_bits.saturating_add(1),
        },
        CapacityClass::Exact(value) => Log2Estimate {
            lower_bits: 127 - value.leading_zeros(),
            upper_bits: 129 - value.leading_zeros(),
        },
    };
    let beyond = CapacityClass::BeyondU128 { log2 };
    let mut tier = 1u128;
    for p in 1..=suffix {
        let Some(next) = tier.checked_mul(map.len(p) as u128) else {
            return Ok(beyond);
        };
        tier = next;
    }
    if min == 1 {
        let Some(next) = tier.checked_mul(map.len(0) as u128) else {
            return Ok(beyond);
        };
        tier = next;
    }
    let mut total = 0u128;
    for k in min..=max - suffix {
        let Some(next) = total.checked_add(tier) else {
            return Ok(beyond);
        };
        total = next;
        if k < max - suffix {
            let Some(next) = tier.checked_mul(map.len(0) as u128) else {
                return Ok(beyond);
            };
            tier = next;
        }
    }
    Ok(CapacityClass::Exact(total))
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Toy;
    impl WordMap for Toy {
        fn len(&self, p: usize) -> usize {
            if p < 2 {
                2
            } else {
                0
            }
        }
        fn word(&self, i: usize, p: usize) -> Option<&str> {
            [["calm", "wild"], ["cat", "dog"]]
                .get(p)
                .and_then(|words| words.get(i))
                .copied()
        }
        fn index_of(&self, word: &str, p: usize) -> Option<usize> {
            (0..self.len(p)).find(|i| self.word(*i, p) == Some(word))
        }
    }
    fn map() -> Toy {
        Toy
    }
    #[test]
    fn frozen_tiers_and_range_stability() {
        let zero = VariablePositional::new(map(), 1, 0, Some(1000), None).unwrap();
        let one = VariablePositional::new(map(), 1, 1, Some(1000), None).unwrap();
        for (id, a, b) in [
            (0, "cat", "calm cat"),
            (1, "dog", "calm dog"),
            (2, "calm cat", "wild cat"),
            (3, "calm dog", "wild dog"),
            (4, "wild cat", "calm calm cat"),
            (5, "wild dog", "calm calm dog"),
            (6, "calm calm cat", "calm wild cat"),
            (7, "calm calm dog", "calm wild dog"),
            (8, "calm wild cat", "wild calm cat"),
            (9, "calm wild dog", "wild calm dog"),
            (12, "wild wild cat", "calm calm calm cat"),
            (13, "wild wild dog", "calm calm calm dog"),
            (14, "calm calm calm cat", "calm calm wild cat"),
        ] {
            assert_eq!(zero.encode(id).unwrap(), a);
            assert_eq!(one.encode(id).unwrap(), b);
            assert_eq!(
                zero.decode_words(&a.split_whitespace().collect::<Vec<_>>())
                    .unwrap(),
                id
            );
            assert_eq!(
                one.decode_words(&b.split_whitespace().collect::<Vec<_>>())
                    .unwrap(),
                id
            );
        }
        let bounded = VariablePositional::new(map(), 1, 0, None, Some(3)).unwrap();
        assert_eq!(bounded.range(), 14);
        assert_eq!(bounded.capacity(), Some(CapacityClass::Exact(14)));
        assert_eq!(bounded.encode(13).unwrap(), "wild wild dog");
        assert!(bounded.encode(14).is_err());
        let seven = VariablePositional::new(map(), 1, 0, Some(7), Some(3)).unwrap();
        assert_eq!(seven.encode(6).unwrap(), zero.encode(6).unwrap());
        assert!(seven.decode_words(&["calm", "calm", "dog"]).is_err());
        assert!(VariablePositional::new(map(), 1, 0, Some(15), Some(3)).is_err());
    }
    struct Sizes(Vec<usize>);
    impl WordMap for Sizes {
        fn len(&self, p: usize) -> usize {
            self.0.get(p).copied().unwrap_or(0)
        }
        fn word(&self, i: usize, p: usize) -> Option<&str> {
            ["a", "b", "c", "d", "e"]
                .get(i)
                .copied()
                .filter(|_| i < self.len(p))
        }
        fn index_of(&self, w: &str, p: usize) -> Option<usize> {
            (0..self.len(p)).find(|i| self.word(*i, p) == Some(w))
        }
    }
    #[test]
    fn independently_enumerated_mixed_tiers() {
        for min in [0, 1] {
            let codec =
                VariablePositional::new(Sizes(vec![3, 2, 5]), 2, min, None, Some(7)).unwrap();
            let mut offset = 0;
            for repeats in min..=5 {
                let capacity = 10 * 3u128.pow(repeats as u32);
                for local in 0..capacity {
                    let mut q = local;
                    let mut indexes = vec![0; repeats + 2];
                    let mut sizes = vec![3; repeats];
                    sizes.extend([2, 5]);
                    for p in (0..indexes.len()).rev() {
                        indexes[p] = (q % sizes[p]) as usize;
                        q /= sizes[p];
                    }
                    let expected = indexes
                        .iter()
                        .map(|i| ["a", "b", "c", "d", "e"][*i])
                        .collect::<Vec<_>>();
                    assert_eq!(codec.encode(offset + local).unwrap(), expected.join(" "));
                    assert_eq!(codec.decode_words(&expected).unwrap(), offset + local);
                }
                offset += capacity;
            }
            assert_eq!(codec.capacity(), Some(CapacityClass::Exact(offset)));
        }
    }
    #[test]
    fn shared_vectors_and_monotonicity() {
        for row in include_str!("../../../tests/vectors/variable/variable-v1.tsv")
            .lines()
            .filter(|row| !row.starts_with('#'))
        {
            let fields = row.split('\t').collect::<Vec<_>>();
            let id = fields[0].parse().unwrap();
            for min in [0, 1] {
                let codec = VariablePositional::new(map(), 1, min, Some(1000), None).unwrap();
                assert_eq!(codec.encode(id).unwrap(), fields[min + 1]);
            }
        }
        let codec = VariablePositional::new(Sizes(vec![3, 2, 5]), 2, 0, Some(401), None).unwrap();
        let larger =
            VariablePositional::new(Sizes(vec![3, 2, 5]), 2, 0, Some(1000), Some(12)).unwrap();
        let mut previous = 0;
        for id in 0..401 {
            let phrase = codec.encode(id).unwrap();
            let count = phrase.split_whitespace().count();
            assert!(count >= previous);
            previous = count;
            assert_eq!(larger.encode(id).unwrap(), phrase);
            assert_eq!(
                codec
                    .decode_words(&phrase.split_whitespace().collect::<Vec<_>>())
                    .unwrap(),
                id
            );
        }
        let slack = larger.encode(401).unwrap();
        assert!(codec
            .decode_words(&slack.split_whitespace().collect::<Vec<_>>())
            .is_err());
    }

    #[test]
    fn boundaries_and_invalid_grammars() {
        assert!(VariablePositional::new(map(), 1, 0, None, None).is_err());
        assert!(VariablePositional::new(map(), 0, 0, Some(1), None).is_err());
        assert!(VariablePositional::new(map(), 1, 2, Some(1), None).is_err());
        assert!(VariablePositional::new(map(), 1, 1, Some(1), Some(1)).is_err());
        assert!(VariablePositional::new(map(), 1, 0, None, Some(1025)).is_err());
        assert!(VariablePositional::new(map(), 1, 0, None, Some(256)).is_err());
        let codec = VariablePositional::new(map(), 1, 0, Some(u128::MAX), Some(256)).unwrap();
        assert!(matches!(
            codec.capacity(),
            Some(CapacityClass::BeyondU128 { .. })
        ));
        for id in [0, 1, u64::MAX as u128, u128::MAX - 2, u128::MAX - 1] {
            let phrase = codec.encode(id).unwrap();
            assert_eq!(
                codec
                    .decode_words(&phrase.split_whitespace().collect::<Vec<_>>())
                    .unwrap(),
                id
            );
        }
        assert!(codec.decode_words(&vec!["wild"; 257]).is_err());
        assert!(codec.decode_words(&[]).is_err());
        assert_eq!(
            codec.decode_words(&["sensitive"]),
            Err(VariableError::Codec(Error::UnknownWord { position: 0 }))
        );
        let mut overflow = vec!["wild"; 128];
        overflow.push("dog");
        assert_eq!(
            codec.decode_words(&overflow),
            Err(VariableError::Codec(Error::BitFrameOverflow))
        );
        let small = VariablePositional::new(map(), 1, 0, Some(3), Some(256)).unwrap();
        assert_eq!(small.encode(2).unwrap(), "calm cat");
    }
}
