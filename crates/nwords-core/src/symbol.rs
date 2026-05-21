use crate::{BitFrame, BitView, Error, Result, SymbolCodec};

use alloc::vec::Vec;

/// Big-endian 11-bit symbol codec used by BIP-39.
#[derive(Debug, Clone, Copy, Default)]
pub struct BigEndian11Bit;

impl SymbolCodec for BigEndian11Bit {
    fn pack(&self, frame: BitView<'_>) -> Result<Vec<u32>> {
        if frame.bit_len() % 11 != 0 {
            return Err(Error::InvalidBitLength {
                got: frame.bit_len(),
                multiple: 11,
            });
        }

        let mut symbols = Vec::with_capacity(frame.bit_len() / 11);
        for start in (0..frame.bit_len()).step_by(11) {
            let mut symbol = 0u32;
            for offset in 0..11 {
                symbol <<= 1;
                if frame.bit(start + offset).ok_or(Error::BitFrameOverflow)? {
                    symbol |= 1;
                }
            }
            symbols.push(symbol);
        }
        Ok(symbols)
    }

    fn unpack(&self, symbols: &[u32]) -> Result<BitFrame> {
        let mut frame = BitFrame::with_bit_capacity(symbols.len() * 11);
        for (position, &symbol) in symbols.iter().enumerate() {
            if symbol >= 2048 {
                return Err(Error::SymbolOutOfRange {
                    index: symbol,
                    position,
                    len: 2048,
                });
            }
            frame.push_bits(u128::from(symbol), 11)?;
        }
        Ok(frame)
    }
}

/// Positional base-N integer codec.
#[cfg(feature = "stats")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BaseN {
    base: u32,
    word_count: usize,
    range: u128,
}

#[cfg(feature = "stats")]
impl BaseN {
    /// Creates a base-N codec for IDs in `0..range`.
    pub fn new(base: usize, word_count: usize, range: u128) -> Result<Self> {
        if base < 2 || base > u32::MAX as usize {
            return Err(Error::InvalidDictionarySize { got: base });
        }
        if word_count == 0 {
            return Err(Error::InvalidWordCount { got: word_count });
        }
        if range == 0 {
            return Err(Error::InvalidRange { got: range });
        }

        let capacity = crate::stats::capacity_uniform(base, word_count)?;
        if let crate::stats::CapacityClass::Exact(capacity) = capacity {
            if range > capacity {
                return Err(Error::RangeExceedsCapacity { range, capacity });
            }
        }

        Ok(Self {
            base: base as u32,
            word_count,
            range,
        })
    }

    /// Returns the base.
    pub const fn base(&self) -> u32 {
        self.base
    }

    /// Returns the fixed word count.
    pub const fn word_count(&self) -> usize {
        self.word_count
    }

    /// Returns the accepted ID range.
    pub const fn range(&self) -> u128 {
        self.range
    }

    /// Encodes an ID into fixed-width base-N symbols.
    pub fn encode_id(&self, id: u128) -> Result<Vec<u32>> {
        if id >= self.range {
            return Err(Error::IndexOutOfRange {
                value: id,
                range: self.range,
            });
        }

        let mut symbols = alloc::vec![0; self.word_count];
        let mut remaining = id;
        for symbol in symbols.iter_mut().rev() {
            *symbol = (remaining % u128::from(self.base)) as u32;
            remaining /= u128::from(self.base);
        }
        Ok(symbols)
    }

    /// Decodes fixed-width base-N symbols into an ID.
    pub fn decode_symbols(&self, symbols: &[u32]) -> Result<u128> {
        if symbols.len() != self.word_count {
            return Err(Error::InvalidWordCount { got: symbols.len() });
        }

        let mut value = 0u128;
        for (position, &symbol) in symbols.iter().enumerate() {
            if symbol >= self.base {
                return Err(Error::SymbolOutOfRange {
                    index: symbol,
                    position,
                    len: self.base as usize,
                });
            }
            value = value
                .checked_mul(u128::from(self.base))
                .and_then(|value| value.checked_add(u128::from(symbol)))
                .ok_or(Error::BitFrameOverflow)?;
        }

        if value >= self.range {
            return Err(Error::IndexOutOfRange {
                value,
                range: self.range,
            });
        }

        Ok(value)
    }

    fn bit_width(&self) -> Result<usize> {
        crate::stats::ceil_log_base(self.range, 2)
    }
}

#[cfg(feature = "stats")]
impl SymbolCodec for BaseN {
    fn pack(&self, frame: BitView<'_>) -> Result<Vec<u32>> {
        let id = frame_to_u128(frame)?;
        self.encode_id(id)
    }

    fn unpack(&self, symbols: &[u32]) -> Result<BitFrame> {
        let id = self.decode_symbols(symbols)?;
        let bit_width = self.bit_width()?;
        let mut frame = BitFrame::with_bit_capacity(bit_width);
        frame.push_bits(id, bit_width)?;
        Ok(frame)
    }
}

/// Positional mixed-radix integer codec.
#[cfg(feature = "stats")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MixedRadix {
    bases: Vec<u32>,
    range: u128,
}

#[cfg(feature = "stats")]
impl MixedRadix {
    /// Creates a mixed-radix codec for IDs in `0..range`.
    pub fn new(dictionary_sizes: &[usize], range: u128) -> Result<Self> {
        if dictionary_sizes.is_empty() {
            return Err(Error::InvalidWordCount { got: 0 });
        }
        if range == 0 {
            return Err(Error::InvalidRange { got: range });
        }

        let mut bases = Vec::with_capacity(dictionary_sizes.len());
        for &size in dictionary_sizes {
            if size < 2 || size > u32::MAX as usize {
                return Err(Error::InvalidDictionarySize { got: size });
            }
            bases.push(size as u32);
        }

        let capacity = crate::stats::capacity_mixed(dictionary_sizes)?;
        if let crate::stats::CapacityClass::Exact(capacity) = capacity {
            if range > capacity {
                return Err(Error::RangeExceedsCapacity { range, capacity });
            }
        }

        Ok(Self { bases, range })
    }

    /// Returns the per-position bases.
    pub fn bases(&self) -> &[u32] {
        &self.bases
    }

    /// Returns the fixed word count.
    pub fn word_count(&self) -> usize {
        self.bases.len()
    }

    /// Returns the accepted ID range.
    pub const fn range(&self) -> u128 {
        self.range
    }

    /// Encodes an ID into fixed-width mixed-radix symbols.
    pub fn encode_id(&self, id: u128) -> Result<Vec<u32>> {
        if id >= self.range {
            return Err(Error::IndexOutOfRange {
                value: id,
                range: self.range,
            });
        }

        let mut symbols = alloc::vec![0; self.bases.len()];
        let mut remaining = id;
        for (symbol, &base) in symbols.iter_mut().rev().zip(self.bases.iter().rev()) {
            *symbol = (remaining % u128::from(base)) as u32;
            remaining /= u128::from(base);
        }
        Ok(symbols)
    }

    /// Decodes fixed-width mixed-radix symbols into an ID.
    pub fn decode_symbols(&self, symbols: &[u32]) -> Result<u128> {
        if symbols.len() != self.bases.len() {
            return Err(Error::InvalidWordCount { got: symbols.len() });
        }

        let mut value = 0u128;
        for (position, (&symbol, &base)) in symbols.iter().zip(self.bases.iter()).enumerate() {
            if symbol >= base {
                return Err(Error::SymbolOutOfRange {
                    index: symbol,
                    position,
                    len: base as usize,
                });
            }
            value = value
                .checked_mul(u128::from(base))
                .and_then(|value| value.checked_add(u128::from(symbol)))
                .ok_or(Error::BitFrameOverflow)?;
        }

        if value >= self.range {
            return Err(Error::IndexOutOfRange {
                value,
                range: self.range,
            });
        }

        Ok(value)
    }

    fn bit_width(&self) -> Result<usize> {
        crate::stats::ceil_log_base(self.range, 2)
    }
}

#[cfg(feature = "stats")]
impl SymbolCodec for MixedRadix {
    fn pack(&self, frame: BitView<'_>) -> Result<Vec<u32>> {
        let id = frame_to_u128(frame)?;
        self.encode_id(id)
    }

    fn unpack(&self, symbols: &[u32]) -> Result<BitFrame> {
        let id = self.decode_symbols(symbols)?;
        let bit_width = self.bit_width()?;
        let mut frame = BitFrame::with_bit_capacity(bit_width);
        frame.push_bits(id, bit_width)?;
        Ok(frame)
    }
}

#[cfg(feature = "stats")]
fn frame_to_u128(frame: BitView<'_>) -> Result<u128> {
    if frame.bit_len() > 128 {
        return Err(Error::BitFrameOverflow);
    }

    let mut value = 0u128;
    for index in 0..frame.bit_len() {
        value <<= 1;
        if frame.bit(index).ok_or(Error::BitFrameOverflow)? {
            value |= 1;
        }
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::{BaseN, BigEndian11Bit, MixedRadix};
    use crate::{BitFrame, Error, SymbolCodec};

    #[test]
    fn big_endian_11_bit_packs_and_unpacks() {
        let mut frame = BitFrame::new();
        frame.push_bits(0, 11).expect("push");
        frame.push_bits(2047, 11).expect("push");
        frame.push_bits(17, 11).expect("push");

        let codec = BigEndian11Bit;
        let symbols = codec
            .pack(BitFrame::from_bytes_with_bit_len(frame.as_bytes(), frame.bit_len()).unwrap())
            .expect("pack");

        assert_eq!(symbols, &[0, 2047, 17]);
        assert_eq!(codec.unpack(&symbols).expect("unpack"), frame);
    }

    #[test]
    fn big_endian_11_bit_rejects_misaligned_bits() {
        let frame = BitFrame::from_bytes_with_bit_len(&[0][..], 8).expect("frame");

        assert_eq!(
            BigEndian11Bit.pack(frame),
            Err(Error::InvalidBitLength {
                got: 8,
                multiple: 11
            })
        );
    }

    #[test]
    fn base_n_round_trips_boundary_ids() {
        let codec = BaseN::new(10, 3, 1000).expect("codec");

        assert_eq!(codec.encode_id(0).expect("encode"), &[0, 0, 0]);
        assert_eq!(codec.encode_id(999).expect("encode"), &[9, 9, 9]);
        assert_eq!(codec.decode_symbols(&[0, 0, 0]), Ok(0));
        assert_eq!(codec.decode_symbols(&[9, 9, 9]), Ok(999));
    }

    #[test]
    fn base_n_rejects_out_of_range_ids() {
        let codec = BaseN::new(10, 3, 100).expect("codec");

        assert_eq!(
            codec.encode_id(100),
            Err(Error::IndexOutOfRange {
                value: 100,
                range: 100
            })
        );
        assert_eq!(
            codec.decode_symbols(&[1, 0, 0]),
            Err(Error::IndexOutOfRange {
                value: 100,
                range: 100
            })
        );
    }

    #[test]
    fn base_n_rejects_symbols_outside_base() {
        let codec = BaseN::new(10, 3, 1000).expect("codec");

        assert_eq!(
            codec.decode_symbols(&[0, 10, 0]),
            Err(Error::SymbolOutOfRange {
                index: 10,
                position: 1,
                len: 10
            })
        );
    }

    #[test]
    fn mixed_radix_round_trips_boundary_ids() {
        let codec = MixedRadix::new(&[3, 5, 7], 100).expect("codec");

        assert_eq!(codec.bases(), &[3, 5, 7]);
        assert_eq!(codec.word_count(), 3);
        assert_eq!(codec.range(), 100);
        assert_eq!(codec.encode_id(0).expect("encode"), &[0, 0, 0]);
        assert_eq!(codec.encode_id(99).expect("encode"), &[2, 4, 1]);
        assert_eq!(codec.decode_symbols(&[0, 0, 0]), Ok(0));
        assert_eq!(codec.decode_symbols(&[2, 4, 1]), Ok(99));
    }

    #[test]
    fn mixed_radix_rejects_out_of_range_ids_and_slack_states() {
        let codec = MixedRadix::new(&[3, 5, 7], 100).expect("codec");

        assert_eq!(
            codec.encode_id(100),
            Err(Error::IndexOutOfRange {
                value: 100,
                range: 100
            })
        );
        assert_eq!(
            codec.decode_symbols(&[2, 4, 2]),
            Err(Error::IndexOutOfRange {
                value: 100,
                range: 100
            })
        );
    }

    #[test]
    fn mixed_radix_rejects_symbols_outside_position_base() {
        let codec = MixedRadix::new(&[3, 5, 7], 100).expect("codec");

        assert_eq!(
            codec.decode_symbols(&[0, 5, 0]),
            Err(Error::SymbolOutOfRange {
                index: 5,
                position: 1,
                len: 5
            })
        );
    }

    #[test]
    fn mixed_radix_validates_shape() {
        assert_eq!(
            MixedRadix::new(&[], 1),
            Err(Error::InvalidWordCount { got: 0 })
        );
        assert_eq!(
            MixedRadix::new(&[3, 1, 7], 1),
            Err(Error::InvalidDictionarySize { got: 1 })
        );
        assert_eq!(
            MixedRadix::new(&[3, 5, 7], 0),
            Err(Error::InvalidRange { got: 0 })
        );
        assert_eq!(
            MixedRadix::new(&[3, 5, 7], 106),
            Err(Error::RangeExceedsCapacity {
                range: 106,
                capacity: 105
            })
        );
    }

    #[test]
    fn mixed_radix_implements_symbol_codec() {
        let codec = MixedRadix::new(&[3, 5, 7], 100).expect("codec");
        let mut frame = BitFrame::new();
        frame.push_bits(42, 7).expect("push");

        let symbols = codec
            .pack(BitFrame::from_bytes_with_bit_len(frame.as_bytes(), frame.bit_len()).unwrap())
            .expect("pack");

        assert_eq!(symbols, &[1, 1, 0]);
        assert_eq!(codec.unpack(&symbols).expect("unpack"), frame);
    }
}
