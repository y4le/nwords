//! Internal decimal-string ABI for the JavaScript package. Codecs and word
//! ordering remain owned by nwords; this crate only validates and binds inputs.
#![forbid(unsafe_code)]

use nwords::{
    core::Error,
    positional::MixedPositional,
    stats::{capacity_mixed, CapacityClass},
    wordlists::named::{NamedWordList, WordListRole, WordListSequence},
};
use wasm_bindgen::prelude::wasm_bindgen;

const MAX_POSITIONS: usize = 32;
const MAX_PHRASE_BYTES: usize = 4096;
const SUPPORTED: &[NamedWordList] = &[
    NamedWordList::Adjective,
    NamedWordList::Animal,
    NamedWordList::Color,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Code {
    InvalidInput,
    UnknownList,
    InvalidShape,
    CapacityOverflow,
    OutOfRange,
    InvalidPhrase,
    Internal,
}

impl Code {
    fn name(self) -> &'static str {
        match self {
            Self::InvalidInput => "INVALID_INPUT",
            Self::UnknownList => "UNKNOWN_LIST",
            Self::InvalidShape => "INVALID_SHAPE",
            Self::CapacityOverflow => "CAPACITY_OVERFLOW",
            Self::OutOfRange => "OUT_OF_RANGE",
            Self::InvalidPhrase => "INVALID_PHRASE",
            Self::Internal => "INTERNAL_ERROR",
        }
    }

    fn message(self) -> &'static str {
        match self {
            Self::InvalidInput => "Expected a canonical unsigned decimal integer within u128.",
            Self::UnknownList => "Unsupported canonical word-list name.",
            Self::InvalidShape => {
                "Shape must have 1 to 32 lists and a positive range within capacity."
            }
            Self::CapacityOverflow => "Shape capacity exceeds u128; supply an explicit range.",
            Self::OutOfRange => "Value is outside the accepted range.",
            Self::InvalidPhrase => "Phrase is invalid for the selected shape.",
            Self::Internal => "Unexpected codec error.",
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct BindingError {
    code: Code,
    field: &'static str,
    position: Option<usize>,
}

impl BindingError {
    fn new(code: Code, field: &'static str) -> Self {
        Self {
            code,
            field,
            position: None,
        }
    }

    fn at(mut self, position: usize) -> Self {
        self.position = Some(position);
        self
    }

    fn codec(error: Error, field: &'static str) -> Self {
        match error {
            Error::UnknownWord { position } => Self::new(Code::InvalidPhrase, field).at(position),
            Error::InvalidWordCount { .. } => Self::new(Code::InvalidPhrase, field),
            Error::IndexOutOfRange { .. } | Error::BitFrameOverflow => {
                Self::new(Code::OutOfRange, field)
            }
            Error::InvalidRange { .. } | Error::RangeExceedsCapacity { .. } => {
                Self::new(Code::InvalidShape, "range")
            }
            Error::InvalidDictionarySize { .. } => Self::new(Code::InvalidShape, "shape"),
            Error::InvalidEntropyLength { .. }
            | Error::InvalidBitLength { .. }
            | Error::InvalidPermutation { .. }
            | Error::InvalidPermutationDomain { .. }
            | Error::InconsistentDictionarySize { .. }
            | Error::InvalidChecksum
            | Error::SymbolOutOfRange { .. }
            | Error::InvalidFrame { .. }
            | Error::NonCanonicalPadding
            | Error::LengthMismatch
            | Error::LengthOverflow
            | Error::InvalidUtf8
            | Error::NormalizationUnavailable => Self::new(Code::Internal, field),
        }
    }
}

// All interpolated strings are static error metadata, canonical names, decimal
// numbers, or encoded words from our lowercase ASCII lists. User text never
// enters a JSON response; no general-purpose JSON serializer is needed here.
fn envelope(result: Result<String, BindingError>) -> String {
    match result {
        Ok(value) => format!(r#"{{"ok":true,"value":{value}}}"#),
        Err(error) => {
            let position = error
                .position
                .map_or(String::new(), |p| format!(r#", "position":{p}"#));
            format!(
                r#"{{"ok":false,"error":{{"code":"{}","message":"{}","field":"{}"{position}}}}}"#,
                error.code.name(),
                error.code.message(),
                error.field
            )
        }
    }
}

fn decimal(input: &str, field: &'static str) -> Result<u128, BindingError> {
    if input.is_empty()
        || input.len() > 39
        || !input.bytes().all(|b| b.is_ascii_digit())
        || (input.len() > 1 && input.starts_with('0'))
    {
        return Err(BindingError::new(Code::InvalidInput, field));
    }
    input
        .parse()
        .map_err(|_| BindingError::new(Code::InvalidInput, field))
}

struct Shape {
    lists: Vec<NamedWordList>,
    range: u128,
    capacity: CapacityClass,
}

impl Shape {
    fn resolve(csv: &str, range: Option<&str>) -> Result<Self, BindingError> {
        if csv.is_empty() || csv.len() > MAX_POSITIONS * 10 {
            return Err(BindingError::new(Code::InvalidShape, "shape"));
        }
        let mut lists = Vec::new();
        for (position, name) in csv.split(',').enumerate() {
            if position >= MAX_POSITIONS || name.is_empty() {
                return Err(BindingError::new(Code::InvalidShape, "shape"));
            }
            let list = SUPPORTED
                .iter()
                .find(|list| list.name() == name)
                .ok_or_else(|| BindingError::new(Code::UnknownList, "shape").at(position))?;
            lists.push(*list);
        }
        let sizes = lists.iter().map(|list| list.len()).collect::<Vec<_>>();
        let capacity =
            capacity_mixed(&sizes).map_err(|error| BindingError::codec(error, "shape"))?;
        let range = match range {
            Some(value) => decimal(value, "range")?,
            None => match capacity {
                CapacityClass::Exact(value) => value,
                CapacityClass::BeyondU128 { .. } => {
                    return Err(BindingError::new(Code::CapacityOverflow, "range"))
                }
            },
        };
        Ok(Self {
            lists,
            range,
            capacity,
        })
    }

    fn codec(&self) -> Result<MixedPositional<WordListSequence<'_>>, BindingError> {
        MixedPositional::new(
            WordListSequence::new(&self.lists),
            self.lists.len(),
            self.range,
        )
        .map_err(|error| BindingError::codec(error, "shape"))
    }
}

/// Returns metadata for the three supported canonical lists.
#[wasm_bindgen]
pub fn lists_json() -> String {
    let lists = SUPPORTED
        .iter()
        .map(|list| {
            let role = match list.role() {
                WordListRole::Head => "head",
                WordListRole::Modifier => "modifier",
                WordListRole::Either => "either",
            };
            format!(
                r#"{{"name":"{}","size":"{}","role":"{role}"}}"#,
                list.name(),
                list.len()
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    envelope(Ok(format!("[{lists}]")))
}

/// Describes an ordered comma-separated canonical shape and optional decimal range.
#[wasm_bindgen]
pub fn describe_shape_json(lists: &str, range: Option<String>) -> String {
    envelope((|| {
        let shape = Shape::resolve(lists, range.as_deref())?;
        shape.codec()?; // Description must validate the range without encoding an ID.
        let lists = shape
            .lists
            .iter()
            .map(|list| format!("\"{}\"", list.name()))
            .collect::<Vec<_>>()
            .join(",");
        let capacity = match shape.capacity {
            CapacityClass::Exact(value) => format!(r#"{{"kind":"exact","value":"{value}"}}"#),
            CapacityClass::BeyondU128 { log2 } => format!(
                r#"{{"kind":"beyond-u128","log2":{{"lower":{},"upper":{}}}}}"#,
                log2.lower_bits, log2.upper_bits
            ),
        };
        Ok(format!(
            r#"{{"lists":[{lists}],"range":"{}","capacity":{capacity}}}"#,
            shape.range
        ))
    })())
}

/// Encodes a canonical decimal ID. Integers never cross the ABI as primitive u128.
#[wasm_bindgen]
pub fn encode_id_json(id: &str, lists: &str, range: Option<String>) -> String {
    envelope((|| {
        let id = decimal(id, "id")?;
        let shape = Shape::resolve(lists, range.as_deref())?;
        let phrase = shape
            .codec()?
            .encode(id)
            .map_err(|error| BindingError::codec(error, "id"))?;
        Ok(format!("\"{phrase}\""))
    })())
}

/// Decodes Rust whitespace-separated words with exact case-sensitive lookup.
#[wasm_bindgen]
pub fn decode_phrase_json(phrase: &str, lists: &str, range: Option<String>) -> String {
    envelope((|| {
        if phrase.len() > MAX_PHRASE_BYTES {
            return Err(BindingError::new(Code::InvalidPhrase, "phrase"));
        }
        let shape = Shape::resolve(lists, range.as_deref())?;
        let words = phrase
            .split_whitespace()
            .take(MAX_POSITIONS + 1)
            .collect::<Vec<_>>();
        let id = shape
            .codec()?
            .decode_words(&words)
            .map_err(|error| BindingError::codec(error, "phrase"))?;
        Ok(format!("\"{id}\""))
    })())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_committed_cli_vectors() {
        for row in include_str!("../../../tests/vectors/js/named-shapes.tsv")
            .lines()
            .filter(|row| !row.starts_with('#'))
        {
            let fields = row.split('\t').collect::<Vec<_>>();
            assert_eq!(fields.len(), 5);
            let range = (fields[2] != "-").then(|| fields[2].to_owned());
            assert_eq!(
                encode_id_json(fields[3], fields[1], range.clone()),
                format!(r#"{{"ok":true,"value":"{}"}}"#, fields[4]),
                "{}",
                fields[0]
            );
            assert_eq!(
                decode_phrase_json(fields[4], fields[1], range),
                format!(r#"{{"ok":true,"value":"{}"}}"#, fields[3]),
                "{}",
                fields[0]
            );
        }
    }

    #[test]
    fn checks_canonical_decimal_at_the_abi() {
        for invalid in [
            "+1",
            "01",
            " 1",
            "1 ",
            "1\n",
            "",
            "1e3",
            "-0",
            "-1",
            "340282366920938463463374607431768211456",
        ] {
            assert!(encode_id_json(invalid, "animal", None).contains("INVALID_INPUT"));
            assert!(
                describe_shape_json("animal", Some(invalid.to_owned())).contains("INVALID_INPUT")
            );
        }
        assert_eq!(decimal(&u128::MAX.to_string(), "id"), Ok(u128::MAX));
        assert!(describe_shape_json("animal", Some("0".to_owned())).contains("INVALID_SHAPE"));
        assert!(describe_shape_json("animal", Some("334".to_owned())).contains("INVALID_SHAPE"));
    }

    #[test]
    fn exposes_true_capacity_and_rejects_slack_and_overflow() {
        let shape = vec!["animal"; 16].join(",");
        assert!(describe_shape_json(&shape, None).contains("CAPACITY_OVERFLOW"));
        let info = describe_shape_json(&shape, Some(u128::MAX.to_string()));
        assert!(info.contains(r#""kind":"beyond-u128""#));
        assert!(info.contains(&format!(r#""range":"{}""#, u128::MAX)));
        assert!(
            encode_id_json(&u128::MAX.to_string(), &shape, Some(u128::MAX.to_string()))
                .contains("OUT_OF_RANGE")
        );
        assert!(
            decode_phrase_json("zebra", "animal", Some("100".to_owned())).contains("OUT_OF_RANGE")
        );
        assert!(decode_phrase_json(
            &vec!["zebra"; 16].join(" "),
            &shape,
            Some(u128::MAX.to_string())
        )
        .contains("OUT_OF_RANGE"));
        assert!(describe_shape_json("animal,animal", None).contains(r#""value":"110889""#));
    }

    #[test]
    fn bounds_and_sanitizes_inputs_without_changing_the_grammar() {
        for shape in ["", &vec!["animal"; 33].join(",")] {
            assert!(describe_shape_json(shape, None).contains("INVALID_SHAPE"));
        }
        for name in ["animals", "bip39-en", "descriptor", "secret\"list"] {
            let shape = format!("animal,{name}");
            let error = describe_shape_json(&shape, None);
            assert!(error.contains("UNKNOWN_LIST"));
            assert!(error.contains(r#""position":1"#));
            assert!(!error.contains(name));
        }
        let error = decode_phrase_json("able secret-token", "adjective,animal", None);
        assert!(error.contains("INVALID_PHRASE"));
        assert!(error.contains(r#""position":1"#));
        assert!(!error.contains("secret-token"));
        for phrase in [
            "Able aardvark",
            "able-aardvark",
            "aardvark",
            "",
            &"a".repeat(4097),
        ] {
            assert!(decode_phrase_json(phrase, "adjective,animal", None).contains("INVALID_PHRASE"));
        }
        assert_eq!(
            decode_phrase_json("\t able\u{2003}aardvark \n", "adjective,animal", None),
            r#"{"ok":true,"value":"0"}"#
        );
    }
}
