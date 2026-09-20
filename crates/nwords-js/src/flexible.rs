//! Checked transport for owned dictionaries and variable patterns.
use super::*;
use nwords::{
    radix_bytes::{RadixBytes, RadixBytesError},
    variable::{VariableError, VariablePositional},
    wordlists::named::{DynamicWordListSequence, DynamicWordListSlot, OwnedWordList},
};
const MAX_CUSTOM_BYTES: usize = 1_048_576;
const MAX_CUSTOM_WORDS: usize = 65_536;
const MAX_PAYLOAD_BYTES: usize = 4096;
const MAX_BYTE_PHRASE: usize = 8 * 1_048_576;

// Tabs/newlines frame the private transport. Tokens cannot contain whitespace or
// controls; names and roles have bounded, explicitly checked alphabets.
fn resolve(lists: &str, custom: &str) -> Result<DynamicWordListSequence, BindingError> {
    // Per list: 64 name bytes, name tab, eight role bytes, and a newline.
    // Token separators are covered by MAX_CUSTOM_WORDS.
    if lists.len() > MAX_POSITIONS * 65
        || custom.len() > MAX_CUSTOM_BYTES + MAX_CUSTOM_WORDS + MAX_POSITIONS * 74
    {
        return Err(BindingError::new(Code::InvalidShape, "shape"));
    }
    let mut owned = Vec::new();
    let mut total_words = 0;
    let mut total_bytes = 0;
    if !custom.is_empty() {
        for row in custom.split('\n') {
            if owned.len() >= 32 {
                return Err(BindingError::new(Code::InvalidShape, "shape"));
            }
            let mut columns = row.split('\t');
            let name = columns
                .next()
                .ok_or_else(|| BindingError::new(Code::InvalidShape, "shape"))?;
            if name.len() > 64 {
                return Err(BindingError::new(Code::InvalidShape, "shape"));
            }
            let role = match columns.next() {
                Some("modifier") => WordListRole::Modifier,
                Some("head") => WordListRole::Head,
                Some("either") => WordListRole::Either,
                _ => return Err(BindingError::new(Code::InvalidShape, "shape")),
            };
            let mut words = Vec::new();
            for word in columns {
                total_words += 1;
                total_bytes += word.len();
                if total_words > MAX_CUSTOM_WORDS
                    || total_bytes > MAX_CUSTOM_BYTES
                    || word.len() > 64
                {
                    return Err(BindingError::new(Code::InvalidShape, "shape"));
                }
                words.push(word.to_owned());
            }
            owned.push(
                OwnedWordList::from_tokens(name, role, words)
                    .map_err(|_| BindingError::new(Code::InvalidShape, "shape"))?,
            );
        }
    }
    let mut positions = Vec::new();
    for (position, name) in lists.split(',').enumerate() {
        if position >= MAX_POSITIONS {
            return Err(BindingError::new(Code::InvalidShape, "shape"));
        }
        if let Some(list) = SUPPORTED.iter().find(|list| list.name() == name) {
            positions.push(DynamicWordListSlot::Builtin(*list));
        } else if let Some(index) = owned.iter().position(|list| list.name() == name) {
            positions.push(DynamicWordListSlot::Owned(index));
        } else {
            return Err(BindingError::new(Code::UnknownList, "shape").at(position));
        }
    }
    DynamicWordListSequence::new(owned, positions)
        .map_err(|_| BindingError::new(Code::InvalidShape, "shape"))
}
fn variable_error(error: VariableError, field: &'static str) -> BindingError {
    match error {
        VariableError::Codec(Error::BitFrameOverflow) => {
            BindingError::new(Code::NumericOverflow, field)
        }
        VariableError::Codec(error) => BindingError::codec(error, field),
        VariableError::CapacityOverflow => BindingError::new(Code::CapacityOverflow, "range"),
        _ => BindingError::new(Code::InvalidShape, "shape"),
    }
}
fn capacity_json(capacity: Option<CapacityClass>) -> String {
    match capacity {
        None => r#"{"kind":"unbounded"}"#.into(),
        Some(CapacityClass::Exact(value)) => format!(r#"{{"kind":"exact","value":"{value}"}}"#),
        Some(CapacityClass::BeyondU128 { log2 }) => format!(
            r#"{{"kind":"beyond-u128","log2":{{"lower":{},"upper":{}}}}}"#,
            log2.lower_bits, log2.upper_bits
        ),
    }
}
enum IntegerCodec {
    Fixed(MixedPositional<DynamicWordListSequence>),
    Variable(VariablePositional<DynamicWordListSequence>),
}
/// Internal immutable owned-format implementation.
#[wasm_bindgen]
pub struct FlexibleCodec {
    codec: IntegerCodec,
    range: u128,
    capacity: Option<CapacityClass>,
    required_words: usize,
}
#[wasm_bindgen]
impl FlexibleCodec {
    /// Builds a checked mixed shape, or a leading-repeat pattern when minimum is present.
    #[wasm_bindgen(constructor)]
    pub fn new(
        lists: &str,
        custom: &str,
        range: Option<String>,
        minimum: Option<u32>,
        max_words: Option<u32>,
    ) -> Result<FlexibleCodec, String> {
        (|| {
            let map = resolve(lists, custom)?;
            let count = map.word_count();
            let range = range
                .as_deref()
                .map(|value| decimal(value, "range"))
                .transpose()?;
            if let Some(minimum) = minimum {
                if count < 2 || max_words.is_some_and(|max| max as usize > MAX_POSITIONS) {
                    return Err(BindingError::new(Code::InvalidShape, "shape"));
                }
                let codec = VariablePositional::new(
                    map,
                    count - 1,
                    minimum as usize,
                    range,
                    max_words.map(|n| n as usize),
                )
                .map_err(|e| variable_error(e, "shape"))?;
                if codec.required_words() > MAX_POSITIONS {
                    return Err(BindingError::new(Code::InvalidShape, "shape"));
                }
                let range = codec.range();
                let capacity = codec.capacity();
                let required_words = codec.required_words();
                Ok(Self {
                    codec: IntegerCodec::Variable(codec),
                    range,
                    capacity,
                    required_words,
                })
            } else {
                if max_words.is_some() {
                    return Err(BindingError::new(Code::InvalidShape, "shape"));
                }
                let sizes = (0..count).map(|p| map.len(p)).collect::<Vec<_>>();
                let capacity =
                    capacity_mixed(&sizes).map_err(|e| BindingError::codec(e, "shape"))?;
                let range = match range {
                    Some(value) => value,
                    None => match capacity {
                        CapacityClass::Exact(value) => value,
                        _ => return Err(BindingError::new(Code::CapacityOverflow, "range")),
                    },
                };
                let codec = MixedPositional::new(map, count, range)
                    .map_err(|e| BindingError::codec(e, "shape"))?;
                Ok(Self {
                    codec: IntegerCodec::Fixed(codec),
                    range,
                    capacity: Some(capacity),
                    required_words: count,
                })
            }
        })()
        .map_err(exception)
    }
    /// Encodes a canonical decimal ID.
    pub fn encode_id(&self, id: &str) -> Result<String, String> {
        (|| {
            let id = decimal(id, "id")?;
            match &self.codec {
                IntegerCodec::Fixed(codec) => {
                    codec.encode(id).map_err(|e| BindingError::codec(e, "id"))
                }
                IntegerCodec::Variable(codec) => {
                    codec.encode(id).map_err(|e| variable_error(e, "id"))
                }
            }
        })()
        .map_err(exception)
    }
    /// Decodes a bounded phrase under the explicit mapping.
    pub fn decode_phrase(&self, phrase: &str) -> Result<u128, String> {
        (|| {
            if phrase.len() > MAX_PHRASE_BYTES {
                return Err(BindingError::new(Code::InvalidPhrase, "phrase"));
            }
            let words = phrase
                .split_whitespace()
                .take(MAX_POSITIONS + 1)
                .collect::<Vec<_>>();
            if words.len() > MAX_POSITIONS {
                return Err(BindingError::new(Code::InvalidPhrase, "phrase"));
            }
            match &self.codec {
                IntegerCodec::Fixed(codec) => codec
                    .decode_words(&words)
                    .map_err(|e| BindingError::codec(e, "phrase")),
                IntegerCodec::Variable(codec) => codec
                    .decode_words(&words)
                    .map_err(|e| variable_error(e, "phrase")),
            }
        })()
        .map_err(exception)
    }
    /// Reports capacity independently from accepted range.
    pub fn describe_json(&self) -> String {
        envelope(Ok(format!(
            r#"{{"range":"{}","capacity":{},"requiredWords":{}}}"#,
            self.range,
            capacity_json(self.capacity),
            self.required_words
        )))
    }
}
fn byte_error(error: RadixBytesError, field: &'static str) -> BindingError {
    match error {
        RadixBytesError::Codec(Error::UnknownWord { position }) => {
            BindingError::new(Code::InvalidPhrase, field).at(position)
        }
        RadixBytesError::Codec(Error::InvalidUtf8) => BindingError::new(Code::InvalidUtf8, field),
        RadixBytesError::LimitExceeded => BindingError::new(Code::InvalidInput, field),
        _ => BindingError::new(Code::InvalidPhrase, field),
    }
}
/// Internal reusable framed-byte codec with owned wordsets.
#[wasm_bindgen]
pub struct ByteCodec {
    codec: RadixBytes<DynamicWordListSequence>,
}
#[wasm_bindgen]
impl ByteCodec {
    /// Snapshots the list template; byte framing is always radix-bytes-v1.
    #[wasm_bindgen(constructor)]
    pub fn new(lists: &str, custom: &str) -> Result<ByteCodec, String> {
        (|| {
            let map = resolve(lists, custom)?;
            let count = map.word_count();
            let codec = RadixBytes::new(map, count, MAX_PAYLOAD_BYTES)
                .map_err(|_| BindingError::new(Code::InvalidShape, "shape"))?;
            Ok(Self { codec })
        })()
        .map_err(exception)
    }
    /// Encodes bounded bytes, preserving exact length.
    pub fn encode_bytes(&self, bytes: &[u8]) -> Result<String, String> {
        self.codec
            .encode_bytes(bytes)
            .map_err(|e| exception(byte_error(e, "bytes")))
    }
    /// Decodes a complete bounded frame.
    pub fn decode_bytes(&self, phrase: &str) -> Result<Vec<u8>, String> {
        self.words(phrase)
            .and_then(|words| {
                self.codec
                    .decode_words(&words)
                    .map_err(|e| byte_error(e, "phrase"))
            })
            .map_err(exception)
    }
    /// Encodes already validated UTF-8.
    pub fn encode_text(&self, text: &str) -> Result<String, String> {
        self.encode_bytes(text.as_bytes())
    }
    /// Decodes and checks UTF-8 in Rust.
    pub fn decode_text(&self, phrase: &str) -> Result<String, String> {
        self.words(phrase)
            .and_then(|words| {
                self.codec
                    .decode_text(&words)
                    .map_err(|e| byte_error(e, "phrase"))
            })
            .map_err(exception)
    }
    /// Returns block sizes for consumer planning.
    pub fn describe_json(&self) -> String {
        envelope(Ok(format!(
            r#"{{"scheme":"radix-bytes-v1","minBlockWords":{},"maxBlockWords":{},"blockBytes":{},"maxBytes":{}}}"#,
            self.codec.min_block_words(),
            self.codec.max_block_words(),
            self.codec.block_bytes(),
            MAX_PAYLOAD_BYTES
        )))
    }
}
impl ByteCodec {
    fn words<'a>(&self, phrase: &'a str) -> Result<Vec<&'a str>, BindingError> {
        if phrase.len() > MAX_BYTE_PHRASE {
            return Err(BindingError::new(Code::InvalidPhrase, "phrase"));
        }
        let max = self
            .codec
            .max_word_count(MAX_PAYLOAD_BYTES)
            .map_err(|e| byte_error(e, "phrase"))?;
        let words = phrase.split_whitespace().take(max + 1).collect::<Vec<_>>();
        if words.len() > max {
            return Err(BindingError::new(Code::InvalidPhrase, "phrase"));
        }
        Ok(words)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn owned_transport_and_variable_bounds_are_checked_without_js() {
        let custom = "mods\tmodifier\tcalm\twild\npets\thead\tcat\tdog";
        let codec = FlexibleCodec::new("mods,pets", custom, None, Some(0), Some(3)).unwrap();
        assert_eq!(codec.encode_id("6").unwrap(), "calm calm cat");
        assert_eq!(codec.decode_phrase("calm calm cat").unwrap(), 6);
        assert!(codec.encode_id("14").unwrap_err().contains("OUT_OF_RANGE"));
        assert!(codec.describe_json().contains("\"value\":\"14\""));
        assert!(FlexibleCodec::new("mods,pets", custom, None, Some(0), None).is_err());
        assert!(FlexibleCodec::new("mods,pets", custom, None, Some(2), Some(3)).is_err());
        assert!(FlexibleCodec::new("mods,pets", custom, None, None, Some(3)).is_err());
        assert!(FlexibleCodec::new(
            "mods,pets",
            custom,
            Some(u128::MAX.to_string()),
            Some(0),
            None
        )
        .is_err());
        assert!(FlexibleCodec::new("mods,pets", custom, None, Some(0), Some(33)).is_err());
        let unicode = "tokens\teither\t🦊\t猫\té\te\u{301}\t\"quoted\"\tback\\slash";
        let fixed = FlexibleCodec::new("tokens,tokens", unicode, None, None, None).unwrap();
        for id in 0..36 {
            let phrase = fixed.encode_id(&id.to_string()).unwrap();
            assert_eq!(fixed.decode_phrase(&phrase).unwrap(), id);
        }
        for invalid in [
            "tokens\teither\tone\tone",
            "animal\teither\ta\tb",
            "tokens\teither\ta\u{feff}\tb",
            "tokens\teither\ta b\tc",
            "tokens\tbogus\ta\tb",
        ] {
            assert!(FlexibleCodec::new("tokens", invalid, None, None, None).is_err());
        }
        let error = fixed.decode_phrase("private words").unwrap_err();
        assert!(!error.contains("private"));
    }
    #[test]
    fn full_custom_budget_with_maximal_names_is_accepted() {
        let words = (0..2048)
            .map(|i| format!("{i:016x}"))
            .collect::<Vec<_>>()
            .join("\t");
        let names = (0..32).map(|i| format!("l{i:063}")).collect::<Vec<_>>();
        let mut custom = names
            .iter()
            .map(|name| format!("{name}\tmodifier\t{words}"))
            .collect::<Vec<_>>()
            .join("\n");
        let csv = names.join(",");
        let map = resolve(&csv, &custom).unwrap();
        assert_eq!(map.word_count(), 32);
        assert_eq!(map.len(31), 2048);
        custom.push('x'); // One token byte beyond the advertised 1 MiB budget.
        assert!(resolve(&csv, &custom).is_err());
    }
    #[test]
    fn bytes_are_framed_and_bounded_at_the_abi() {
        let codec = ByteCodec::new("eff-long", "").unwrap();
        for payload in [&[][..], &[0, 0, 255], &[0; 4096]] {
            let phrase = codec.encode_bytes(payload).unwrap();
            assert_eq!(codec.decode_bytes(&phrase).unwrap(), payload);
        }
        assert!(codec.encode_bytes(&[0; 4097]).is_err());
        assert!(codec.decode_bytes(&"abacus ".repeat(4101)).is_err());
        assert!(codec
            .decode_text(&codec.encode_bytes(&[255]).unwrap())
            .unwrap_err()
            .contains("INVALID_UTF8"));
        assert!(codec
            .decode_bytes("sensitive-token")
            .unwrap_err()
            .contains("position"));
    }
}
