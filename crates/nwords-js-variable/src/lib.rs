//! Dictionary-free WebAssembly binding for Rust's wide `variable-v1` codec.
#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use nwords::wide_variable::{
    OwnedWordMap, OwnedWordMapError, WideId, WideVariableError, WideVariablePositional,
};
use wasm_bindgen::prelude::wasm_bindgen;

const MAX_LISTS: usize = 32;
const MAX_WORDS: usize = 64;
const MAX_TRANSPORT_BYTES: usize = 1_200_000;

fn error(code: &'static str, field: &'static str, position: Option<usize>) -> String {
    let position = position.map_or(String::new(), |index| format!(r#","position":{index}"#));
    format!(r#"{{"code":"{code}","field":"{field}"{position}}}"#)
}

fn variable_error(source: WideVariableError, field: &'static str) -> String {
    match source {
        WideVariableError::OutOfRange => error("OUT_OF_RANGE", field, None),
        WideVariableError::UnknownWord { position }
        | WideVariableError::InvalidWordMap { position } => {
            error("INVALID_PHRASE", "phrase", Some(position))
        }
        WideVariableError::InvalidWordCount { .. } => error("INVALID_PHRASE", "phrase", None),
        WideVariableError::NotByteAligned => error("NOT_BYTE_ALIGNED", field, None),
        WideVariableError::InvalidUtf8 => error("INVALID_UTF8", field, None),
        WideVariableError::InvalidBits | WideVariableError::InvalidBytes => {
            error("INVALID_INPUT", field, None)
        }
        WideVariableError::InvalidInteger | WideVariableError::IntegerTooLong => {
            error("INVALID_INPUT", field, None)
        }
        WideVariableError::InvalidPattern
        | WideVariableError::InvalidBounds
        | WideVariableError::InvalidDictionary { .. } => error("INVALID_SHAPE", field, None),
    }
}

fn valid_name(name: &str) -> bool {
    name.len() <= 64
        && name.bytes().enumerate().all(|(index, byte)| match index {
            0 => byte.is_ascii_lowercase(),
            _ => byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-' || byte == b'_',
        })
        && !name.is_empty()
}

fn resolve_map(definitions: &str, pattern: &str) -> Result<(OwnedWordMap, usize), String> {
    if definitions.is_empty() || definitions.len() > MAX_TRANSPORT_BYTES {
        return Err(error("INVALID_SHAPE", "shape", None));
    }
    let mut names = BTreeMap::new();
    let mut lists = Vec::new();
    for line in definitions.split('\n') {
        let mut fields = line.split('\t');
        let name = fields.next().unwrap_or_default();
        if !valid_name(name) || names.insert(name.to_owned(), lists.len()).is_some() {
            return Err(error("INVALID_SHAPE", "shape", None));
        }
        lists.push(fields.map(str::to_owned).collect::<Vec<_>>());
        if lists.len() > MAX_LISTS {
            return Err(error("INVALID_SHAPE", "shape", None));
        }
    }
    let slots = pattern.split(',').collect::<Vec<_>>();
    if slots.len() < 2 || slots.len() > MAX_LISTS {
        return Err(error("INVALID_SHAPE", "shape", None));
    }
    let mut positions = Vec::with_capacity(slots.len());
    for (position, name) in slots.into_iter().enumerate() {
        if !valid_name(name) {
            return Err(error("INVALID_SHAPE", "shape", Some(position)));
        }
        positions.push(
            *names
                .get(name)
                .ok_or_else(|| error("INVALID_SHAPE", "shape", Some(position)))?,
        );
    }
    if names.values().any(|index| !positions.contains(index)) {
        return Err(error("INVALID_SHAPE", "shape", None));
    }
    let suffix = positions.len() - 1;
    let map = OwnedWordMap::new(lists, positions.clone()).map_err(|source| {
        let list = match source {
            OwnedWordMapError::InvalidLength { list }
            | OwnedWordMapError::InvalidToken { list, .. }
            | OwnedWordMapError::DuplicateToken { list } => Some(list),
            OwnedWordMapError::InvalidPosition { position } => {
                return error("INVALID_SHAPE", "shape", Some(position));
            }
            OwnedWordMapError::TooLarge => None,
        };
        let position = list.and_then(|list| positions.iter().position(|slot| *slot == list));
        error("INVALID_SHAPE", "shape", position)
    })?;
    Ok((map, suffix))
}

/// Reusable Rust-owned wordsets and wide variable-v1 codec.
#[wasm_bindgen]
pub struct VariableCodec {
    codec: WideVariablePositional<OwnedWordMap>,
    max_phrase_bytes: usize,
}

#[wasm_bindgen]
impl VariableCodec {
    /// Validates ordered wordsets and the explicit repeat/suffix pattern.
    #[wasm_bindgen(constructor)]
    pub fn new(
        definitions: &str,
        pattern: &str,
        minimum: u32,
        max_words: u32,
        range: Option<String>,
    ) -> Result<VariableCodec, String> {
        if minimum > 1 || max_words == 0 || max_words as usize > MAX_WORDS {
            return Err(error("INVALID_SHAPE", "shape", None));
        }
        let (map, suffix) = resolve_map(definitions, pattern)?;
        let range = range
            .as_deref()
            .map(WideId::parse_decimal)
            .transpose()
            .map_err(|source| variable_error(source, "range"))?;
        let codec =
            WideVariablePositional::new(map, suffix, minimum as usize, range, max_words as usize)
                .map_err(|source| variable_error(source, "shape"))?;
        Ok(Self {
            codec,
            max_phrase_bytes: 4096usize.max(max_words as usize * 65 - 1),
        })
    }

    /// Encodes an unsigned big-endian integer supplied by JavaScript.
    pub fn encode_id(&self, bytes: &[u8]) -> Result<String, String> {
        if bytes.len() > 512 {
            return Err(error("INVALID_INPUT", "id", None));
        }
        self.codec
            .encode(&WideId::from_be_bytes(bytes))
            .map_err(|source| variable_error(source, "id"))
    }

    /// Decodes a phrase to minimal unsigned big-endian bytes.
    pub fn decode_phrase(&self, phrase: &str) -> Result<Vec<u8>, String> {
        self.phrase(phrase)?;
        self.codec
            .decode_phrase(phrase)
            .map(|id| id.to_be_bytes())
            .map_err(|source| variable_error(source, "phrase"))
    }

    /// Encodes an exact finite bitstring.
    pub fn encode_bits(&self, bits: &str) -> Result<String, String> {
        self.codec
            .encode_bits(bits)
            .map_err(|source| variable_error(source, "bits"))
    }

    /// Decodes a phrase as an exact finite bitstring.
    pub fn decode_bits(&self, phrase: &str) -> Result<String, String> {
        self.phrase(phrase)?;
        self.codec
            .decode_bits(phrase)
            .map_err(|source| variable_error(source, "phrase"))
    }

    /// Encodes at most 512 bytes, preserving leading zero bytes.
    pub fn encode_bytes(&self, bytes: &[u8]) -> Result<String, String> {
        self.codec
            .encode_bytes(bytes)
            .map_err(|source| variable_error(source, "bytes"))
    }

    /// Decodes a phrase as whole bytes.
    pub fn decode_bytes(&self, phrase: &str) -> Result<Vec<u8>, String> {
        self.phrase(phrase)?;
        self.codec
            .decode_bytes(phrase)
            .map_err(|source| variable_error(source, "phrase"))
    }

    /// Encodes UTF-8 text without normalization.
    pub fn encode_text(&self, text: &str) -> Result<String, String> {
        self.codec
            .encode_text(text)
            .map_err(|source| variable_error(source, "text"))
    }

    /// Decodes a phrase as strict UTF-8 text without normalization.
    pub fn decode_text(&self, phrase: &str) -> Result<String, String> {
        self.phrase(phrase)?;
        self.codec
            .decode_text(phrase)
            .map_err(|source| variable_error(source, "phrase"))
    }

    /// Returns exact capacity and accepted range as canonical decimal strings.
    pub fn describe_json(&self) -> String {
        format!(
            r#"{{"range":"{}","capacity":"{}","maxWords":{},"requiredWords":{},"maxBits":{}}}"#,
            self.codec.range().to_decimal_string(),
            self.codec.capacity().to_decimal_string(),
            self.codec.max_words(),
            self.codec.required_words(),
            self.codec.max_bits(),
        )
    }
}

impl VariableCodec {
    fn phrase(&self, phrase: &str) -> Result<(), String> {
        if phrase.len() > self.max_phrase_bytes {
            Err(error("INVALID_PHRASE", "phrase", None))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_abi_has_no_builtin_lists() {
        let codec = VariableCodec::new(
            "adjective\table\twild\nanimal\taardvark\tcat",
            "adjective,animal",
            1,
            32,
            None,
        )
        .unwrap();
        assert_eq!(codec.encode_id(&[0]).unwrap(), "able aardvark");
        assert_eq!(codec.codec.max_bits(), 31);
        assert_eq!(
            error("INVALID_PHRASE", "phrase", Some(1)),
            r#"{"code":"INVALID_PHRASE","field":"phrase","position":1}"#
        );
        assert_eq!(
            codec.decode_phrase("able aardvark").unwrap(),
            Vec::<u8>::new()
        );
        assert_eq!(
            codec
                .decode_bits(&codec.encode_bits("00001011").unwrap())
                .unwrap(),
            "00001011"
        );
        assert_eq!(
            codec
                .decode_text(&codec.encode_text("🦊").unwrap())
                .unwrap(),
            "🦊"
        );
        assert!(VariableCodec::new(
            "adjective\table\table\nanimal\taardvark\tcat",
            "adjective,animal",
            1,
            32,
            None
        )
        .is_err());
        assert!(VariableCodec::new(
            "adjective\table\twild\nanimal\taardvark\tcat",
            "adjective,missing",
            1,
            32,
            None
        )
        .is_err());
    }
}
