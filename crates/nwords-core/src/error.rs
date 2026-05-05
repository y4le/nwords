use core::fmt;

/// Result type used by `nwords-core`.
pub type Result<T> = core::result::Result<T, Error>;

/// Error type for core codec operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Entropy length is not supported by the selected scheme.
    InvalidEntropyLength {
        /// Observed entropy length.
        got: usize,
        /// Supported entropy lengths.
        expected: &'static [usize],
    },
    /// Word count is not supported by the selected scheme.
    InvalidWordCount {
        /// Observed word count.
        got: usize,
    },
    /// Bit length is not valid for the selected symbol codec.
    InvalidBitLength {
        /// Observed bit length.
        got: usize,
        /// Required bit-length multiple.
        multiple: usize,
    },
    /// Dictionary size is not valid for the selected codec.
    InvalidDictionarySize {
        /// Observed dictionary size.
        got: usize,
    },
    /// ID range is not valid for the selected codec.
    InvalidRange {
        /// Observed range.
        got: u128,
    },
    /// Configured ID range exceeds exact representational capacity.
    RangeExceedsCapacity {
        /// Configured accepted ID range.
        range: u128,
        /// Exact representational capacity.
        capacity: u128,
    },
    /// Positional dictionary sizes are not uniform.
    InconsistentDictionarySize {
        /// Position whose dictionary size differed.
        position: usize,
        /// Observed dictionary size at `position`.
        got: usize,
        /// Expected dictionary size.
        expected: usize,
    },
    /// A word was not found at the given phrase position.
    UnknownWord {
        /// Zero-based word position in the phrase.
        position: usize,
    },
    /// Checksum validation failed.
    InvalidChecksum,
    /// Integer ID is outside the configured range.
    IndexOutOfRange {
        /// Observed value.
        value: u128,
        /// Exclusive upper bound.
        range: u128,
    },
    /// Symbol index is outside the word map at the given position.
    SymbolOutOfRange {
        /// Observed symbol index.
        index: u32,
        /// Zero-based symbol position.
        position: usize,
        /// Word map length at that position.
        len: usize,
    },
    /// Bit frame construction or mutation would exceed supported bounds.
    BitFrameOverflow,
    /// Requested text normalization is unavailable in the current build.
    NormalizationUnavailable,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidEntropyLength { got, expected } => {
                write!(
                    f,
                    "invalid entropy length {got}; expected one of {expected:?}"
                )
            }
            Self::InvalidWordCount { got } => write!(f, "invalid word count {got}"),
            Self::InvalidBitLength { got, multiple } => {
                write!(
                    f,
                    "invalid bit length {got}; expected multiple of {multiple}"
                )
            }
            Self::InvalidDictionarySize { got } => {
                write!(f, "invalid dictionary size {got}")
            }
            Self::InvalidRange { got } => write!(f, "invalid range {got}"),
            Self::RangeExceedsCapacity { range, capacity } => {
                write!(f, "range {range} exceeds capacity {capacity}")
            }
            Self::InconsistentDictionarySize {
                position,
                got,
                expected,
            } => write!(
                f,
                "dictionary size {got} at position {position} differs from expected size {expected}"
            ),
            Self::UnknownWord { position } => {
                write!(f, "unknown word at position {position}")
            }
            Self::InvalidChecksum => f.write_str("invalid checksum"),
            Self::IndexOutOfRange { value, range } => {
                write!(f, "index {value} is outside range 0..{range}")
            }
            Self::SymbolOutOfRange {
                index,
                position,
                len,
            } => write!(
                f,
                "symbol index {index} at position {position} is outside length {len}"
            ),
            Self::BitFrameOverflow => f.write_str("bit frame overflow"),
            Self::NormalizationUnavailable => f.write_str("normalization unavailable"),
        }
    }
}

impl core::error::Error for Error {}
