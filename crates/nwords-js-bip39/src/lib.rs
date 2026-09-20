//! Checked English BIP-39 binding for the slim JavaScript entry.
#![forbid(unsafe_code)]

use nwords::{core::Error, schemes::bip39::English};
use wasm_bindgen::prelude::wasm_bindgen;

const MAX_MNEMONIC_BYTES: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BindingError {
    code: &'static str,
    message: &'static str,
    field: &'static str,
    position: Option<usize>,
}

impl BindingError {
    fn new(code: &'static str, message: &'static str, field: &'static str) -> Self {
        Self {
            code,
            message,
            field,
            position: None,
        }
    }

    fn from_codec(error: Error, field: &'static str) -> Self {
        match error {
            Error::InvalidEntropyLength { .. } => Self::new(
                "INVALID_ENTROPY_LENGTH",
                "BIP-39 entropy must be 16, 20, 24, 28, or 32 bytes.",
                field,
            ),
            Error::InvalidWordCount { .. } => Self::new(
                "INVALID_WORD_COUNT",
                "BIP-39 mnemonic must have 12, 15, 18, 21, or 24 words.",
                field,
            ),
            Error::UnknownWord { position } => Self {
                position: Some(position),
                ..Self::new("UNKNOWN_WORD", "Unknown BIP-39 English word.", field)
            },
            Error::InvalidChecksum => {
                Self::new("INVALID_CHECKSUM", "BIP-39 checksum does not match.", field)
            }
            _ => Self::new("INTERNAL_ERROR", "Unexpected mnemonic codec error.", field),
        }
    }

    fn json(self) -> String {
        let position = self
            .position
            .map_or(String::new(), |value| format!(r#", "position":{value}"#));
        format!(
            r#"{{"ok":false,"error":{{"code":"{}","message":"{}","field":"{}"{position}}}}}"#,
            self.code, self.message, self.field
        )
    }
}

/// Encode valid BIP-39 entropy as an English mnemonic.
#[wasm_bindgen]
pub fn encode_entropy(entropy: &[u8]) -> Result<String, String> {
    English::default()
        .encode_entropy(entropy)
        .map_err(|error| BindingError::from_codec(error, "entropy").json())
}

/// Decode an English mnemonic and validate its checksum.
#[wasm_bindgen]
pub fn decode_mnemonic(mnemonic: &str) -> Result<Vec<u8>, String> {
    if mnemonic.len() > MAX_MNEMONIC_BYTES {
        return Err(BindingError::new(
            "INVALID_WORD_COUNT",
            "BIP-39 mnemonic must have 12, 15, 18, 21, or 24 words.",
            "mnemonic",
        )
        .json());
    }
    let words = mnemonic
        .split_ascii_whitespace()
        .take(25)
        .collect::<Vec<_>>();
    if !matches!(words.len(), 12 | 15 | 18 | 21 | 24) {
        return Err(BindingError::new(
            "INVALID_WORD_COUNT",
            "BIP-39 mnemonic must have 12, 15, 18, 21, or 24 words.",
            "mnemonic",
        )
        .json());
    }
    English::default()
        .decode_words(&words)
        .map_err(|error| BindingError::from_codec(error, "mnemonic").json())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_vector_round_trips_and_reports_distinct_errors() {
        let entropy = [0u8; 16];
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        assert_eq!(encode_entropy(&entropy).as_deref(), Ok(phrase));
        assert_eq!(decode_mnemonic(phrase), Ok(entropy.to_vec()));
        assert!(encode_entropy(&[0; 15])
            .unwrap_err()
            .contains("INVALID_ENTROPY_LENGTH"));
        assert!(decode_mnemonic("abandon")
            .unwrap_err()
            .contains("INVALID_WORD_COUNT"));
        assert!(decode_mnemonic(&phrase.replace("about", "missing"))
            .unwrap_err()
            .contains("UNKNOWN_WORD"));
        assert!(decode_mnemonic(&phrase.replace("about", "abandon"))
            .unwrap_err()
            .contains("INVALID_CHECKSUM"));
    }
}
