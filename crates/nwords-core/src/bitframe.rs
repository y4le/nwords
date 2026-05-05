use crate::{Error, Result};

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Owned or borrowed bitstream with an explicit live bit length.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BitFrame<B: AsRef<[u8]> = Vec<u8>> {
    bytes: B,
    bit_len: usize,
}

/// Borrowed bit frame view.
pub type BitView<'a> = BitFrame<&'a [u8]>;

impl<B: AsRef<[u8]>> BitFrame<B> {
    /// Creates a frame covering every bit in `bytes`.
    pub fn from_bytes(bytes: B) -> Result<Self> {
        let bit_len = bytes
            .as_ref()
            .len()
            .checked_mul(8)
            .ok_or(Error::BitFrameOverflow)?;
        Ok(Self { bytes, bit_len })
    }

    /// Creates a frame with an explicit live bit length.
    pub fn from_bytes_with_bit_len(bytes: B, bit_len: usize) -> Result<Self> {
        let capacity_bits = bytes
            .as_ref()
            .len()
            .checked_mul(8)
            .ok_or(Error::BitFrameOverflow)?;
        if bit_len > capacity_bits {
            return Err(Error::BitFrameOverflow);
        }
        Ok(Self { bytes, bit_len })
    }

    /// Returns the backing bytes.
    pub fn as_bytes(&self) -> &[u8] {
        self.bytes.as_ref()
    }

    /// Returns the number of live bits in the frame.
    pub fn bit_len(&self) -> usize {
        self.bit_len
    }

    /// Returns true when the frame has no live bits.
    pub fn is_empty(&self) -> bool {
        self.bit_len == 0
    }

    /// Returns the bit at `index`, using big-endian bit order inside each byte.
    pub fn bit(&self, index: usize) -> Option<bool> {
        if index >= self.bit_len {
            return None;
        }
        let byte = self.bytes.as_ref()[index / 8];
        let offset = 7 - (index % 8);
        Some(((byte >> offset) & 1) == 1)
    }

    /// Returns the backing byte container.
    pub fn into_bytes(self) -> B {
        self.bytes
    }
}

#[cfg(feature = "alloc")]
impl BitFrame<Vec<u8>> {
    /// Creates an empty owned frame.
    pub fn new() -> Self {
        Self {
            bytes: Vec::new(),
            bit_len: 0,
        }
    }

    /// Creates an empty owned frame with enough capacity for `bit_capacity`.
    pub fn with_bit_capacity(bit_capacity: usize) -> Self {
        Self {
            bytes: Vec::with_capacity(bit_capacity.div_ceil(8)),
            bit_len: 0,
        }
    }

    /// Appends one bit.
    pub fn push_bit(&mut self, bit: bool) -> Result<()> {
        if self.bit_len % 8 == 0 {
            self.bytes.push(0);
        }
        if bit {
            let byte_index = self.bit_len / 8;
            let bit_offset = 7 - (self.bit_len % 8);
            self.bytes[byte_index] |= 1 << bit_offset;
        } else {
            let byte_index = self.bit_len / 8;
            let bit_offset = 7 - (self.bit_len % 8);
            self.bytes[byte_index] &= !(1 << bit_offset);
        }
        self.bit_len = self.bit_len.checked_add(1).ok_or(Error::BitFrameOverflow)?;
        Ok(())
    }

    /// Appends the low `bit_count` bits of `value`, most significant bit first.
    pub fn push_bits(&mut self, value: u128, bit_count: usize) -> Result<()> {
        if bit_count > 128 {
            return Err(Error::BitFrameOverflow);
        }
        if bit_count == 0 {
            return Ok(());
        }
        if bit_count < 128 && value >= (1u128 << bit_count) {
            return Err(Error::BitFrameOverflow);
        }
        for bit_index in (0..bit_count).rev() {
            self.push_bit(((value >> bit_index) & 1) == 1)?;
        }
        Ok(())
    }
}

#[cfg(feature = "alloc")]
impl Default for BitFrame<Vec<u8>> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::BitFrame;
    use crate::Error;

    #[test]
    fn borrowed_frame_reads_big_endian_bits() {
        let frame = BitFrame::from_bytes_with_bit_len(&[0b1010_0001], 8).expect("valid frame");

        assert_eq!(frame.bit_len(), 8);
        assert_eq!(frame.bit(0), Some(true));
        assert_eq!(frame.bit(1), Some(false));
        assert_eq!(frame.bit(2), Some(true));
        assert_eq!(frame.bit(7), Some(true));
        assert_eq!(frame.bit(8), None);
    }

    #[test]
    fn rejects_live_bits_past_backing_bytes() {
        let err = BitFrame::from_bytes_with_bit_len(&[0], 9).expect_err("invalid frame");
        assert_eq!(err, Error::BitFrameOverflow);
    }

    #[test]
    fn pushes_across_eight_bit_boundary() {
        let mut frame = BitFrame::new();

        frame.push_bits(0b1010_1010, 8).expect("push byte");
        frame.push_bit(true).expect("push boundary bit");

        assert_eq!(frame.bit_len(), 9);
        assert_eq!(frame.as_bytes(), &[0b1010_1010, 0b1000_0000]);
        assert_eq!(frame.bit(8), Some(true));
    }

    #[test]
    fn pushes_across_eleven_bit_boundary() {
        let mut frame = BitFrame::new();

        frame.push_bits(0b101_0101_1100, 11).expect("push 11 bits");

        assert_eq!(frame.bit_len(), 11);
        assert_eq!(frame.as_bytes(), &[0b1010_1011, 0b1000_0000]);
    }

    #[test]
    fn pushes_sixteen_bits() {
        let mut frame = BitFrame::new();

        frame.push_bits(0xabcd, 16).expect("push 16 bits");

        assert_eq!(frame.bit_len(), 16);
        assert_eq!(frame.as_bytes(), &[0xab, 0xcd]);
    }

    #[test]
    fn rejects_value_wider_than_bit_count() {
        let mut frame = BitFrame::new();

        let err = frame.push_bits(4, 2).expect_err("value does not fit");

        assert_eq!(err, Error::BitFrameOverflow);
    }

    #[test]
    fn pushing_false_clears_stale_trailing_bits() {
        let mut frame = BitFrame::from_bytes_with_bit_len(vec![0xff], 4).expect("valid frame");

        frame.push_bit(false).expect("push false bit");

        assert_eq!(frame.bit(4), Some(false));
        assert_eq!(frame.as_bytes(), &[0b1111_0111]);
    }

    #[test]
    fn pushing_zero_bits_is_a_no_op() {
        let mut frame = BitFrame::new();

        frame.push_bits(u128::MAX, 0).expect("zero-bit push");

        assert!(frame.is_empty());
        assert!(frame.as_bytes().is_empty());
    }

    #[test]
    fn from_bytes_covers_all_backing_bits() {
        let frame = BitFrame::from_bytes([0xaa, 0x55]).expect("valid frame");

        assert_eq!(frame.bit_len(), 16);
        assert_eq!(frame.as_bytes(), &[0xaa, 0x55]);
    }
}
