#![forbid(unsafe_code)]

use std::fmt;

use nwords::{
    core::{AffinePermutation, WordMap},
    positional::Positional,
    schemes::AsciiSpace,
    stats::{self, CapacityClass, PlanSolution, PlanTarget},
    word_bytes::WordBytes,
    wordlists::bip39::English,
};
use wasm_bindgen::prelude::wasm_bindgen;

const DICTIONARY_NAME: &str = "bip39-en-positional";
const POSITIONAL_CAVEAT: &str =
    "BIP-39 wordlist used as a positional dictionary, not a BIP-39 mnemonic.";
const SPREAD_CAVEAT: &str =
    "Spread presets are deterministic permutations; they are not encryption and add no entropy.";
const TEXT_CAVEAT: &str = "Text is encoded as byte-exact UTF-8 with no Unicode normalization.";

const IDENTITY_PERMUTATION: PermutationKind = PermutationKind::Identity;
const DECIMAL_SPREAD: PermutationKind = PermutationKind::SpreadAffine {
    multiplier: 65_537,
    offset: 314_159,
};
const U32_SPREAD: PermutationKind = PermutationKind::SpreadAffine {
    multiplier: 2_654_435_761,
    offset: 1_013_904_223,
};
const U64_SPREAD: PermutationKind = PermutationKind::SpreadAffine {
    multiplier: 6_364_136_223_846_793_005,
    offset: 1_442_695_040_888_963_407,
};
const DEC18_SPREAD: PermutationKind = PermutationKind::SpreadAffine {
    multiplier: 6_364_136_223_846_793_007,
    offset: 1_442_695_040_888_963_407,
};

const PRESETS: &[Preset] = &[
    Preset {
        name: "dec6",
        range: 1_000_000,
        words: 2,
        permutation: IDENTITY_PERMUTATION,
    },
    Preset {
        name: "dec6-spread",
        range: 1_000_000,
        words: 2,
        permutation: DECIMAL_SPREAD,
    },
    Preset {
        name: "dec9",
        range: 1_000_000_000,
        words: 3,
        permutation: IDENTITY_PERMUTATION,
    },
    Preset {
        name: "dec9-spread",
        range: 1_000_000_000,
        words: 3,
        permutation: DECIMAL_SPREAD,
    },
    Preset {
        name: "u32",
        range: 1u128 << 32,
        words: 3,
        permutation: IDENTITY_PERMUTATION,
    },
    Preset {
        name: "u32-spread",
        range: 1u128 << 32,
        words: 3,
        permutation: U32_SPREAD,
    },
    Preset {
        name: "u64",
        range: 1u128 << 64,
        words: 6,
        permutation: IDENTITY_PERMUTATION,
    },
    Preset {
        name: "u64-spread",
        range: 1u128 << 64,
        words: 6,
        permutation: U64_SPREAD,
    },
    Preset {
        name: "dec18",
        range: 1_000_000_000_000_000_000,
        words: 6,
        permutation: IDENTITY_PERMUTATION,
    },
    Preset {
        name: "dec18-spread",
        range: 1_000_000_000_000_000_000,
        words: 6,
        permutation: DEC18_SPREAD,
    },
];

/// Returns metadata and caveats for the static web demo.
#[wasm_bindgen]
pub fn demo_info_json() -> String {
    let mut out = String::from(r#"{"ok":true"#);
    push_string_field(&mut out, "dictionary", DICTIONARY_NAME);
    push_string_field(&mut out, "positional_caveat", POSITIONAL_CAVEAT);
    push_string_field(&mut out, "spread_caveat", SPREAD_CAVEAT);
    push_string_field(&mut out, "text_caveat", TEXT_CAVEAT);
    out.push('}');
    out
}

/// Returns built-in positional demo presets as JSON.
#[wasm_bindgen]
pub fn presets_json() -> String {
    wrap_result(|| {
        let mut out = String::from(r#"{"ok":true"#);
        push_string_field(&mut out, "dictionary", DICTIONARY_NAME);
        out.push_str(r#","presets":["#);
        for (index, preset) in PRESETS.iter().enumerate() {
            if index > 0 {
                out.push(',');
            }
            let report = Report::new(*preset)?;
            push_preset_json(&mut out, preset, &report);
        }
        out.push_str("]}");
        Ok(out)
    })
}

/// Encodes a numeric ID with a built-in positional preset.
#[wasm_bindgen]
pub fn encode_id_json(id: &str, preset: &str) -> String {
    wrap_result(|| {
        let id = parse_u128(id, "id")?;
        let shape = find_preset(preset).ok_or_else(|| WebError::new("unknown preset"))?;
        let phrase = shape.codec()?.encode(id)?;
        let report = Report::new(shape)?;

        let mut out = String::from(r#"{"ok":true"#);
        push_string_field(&mut out, "mode", "encode");
        push_report_fields(&mut out, &report);
        push_u128_field(&mut out, "id", id);
        push_string_field(&mut out, "phrase", &phrase);
        out.push('}');
        Ok(out)
    })
}

/// Decodes a positional phrase into a numeric ID with a built-in preset.
#[wasm_bindgen]
pub fn decode_id_json(phrase: &str, preset: &str) -> String {
    wrap_result(|| {
        let shape = find_preset(preset).ok_or_else(|| WebError::new("unknown preset"))?;
        let words = split_phrase(phrase)?;
        let refs = words.iter().map(String::as_str).collect::<Vec<_>>();
        let id = shape.codec()?.decode_words(&refs)?;
        let report = Report::new(shape)?;

        let mut out = String::from(r#"{"ok":true"#);
        push_string_field(&mut out, "mode", "decode");
        push_report_fields(&mut out, &report);
        push_u128_field(&mut out, "id", id);
        push_string_field(&mut out, "phrase", &words.join(" "));
        out.push('}');
        Ok(out)
    })
}

/// Plans a positional shape over the BIP-39 English positional dictionary.
#[wasm_bindgen]
pub fn plan_json(range: &str, words: &str) -> String {
    wrap_result(|| {
        let range = parse_u128(range, "range")?;
        let words = if words.trim().is_empty() {
            match stats::required_words(PlanTarget::Range(range), English.len(0))? {
                PlanSolution::RequiredWords { word_count, .. } => word_count,
                _ => return Err(WebError::new("unexpected planner result")),
            }
        } else {
            parse_usize(words, "words")?
        };

        let report = Report::custom(words, range)?;
        let mut out = String::from(r#"{"ok":true"#);
        push_string_field(&mut out, "mode", "plan");
        push_report_fields(&mut out, &report);
        out.push('}');
        Ok(out)
    })
}

/// Encodes arbitrary UTF-8 text with `word-bytes-v1`.
#[wasm_bindgen]
pub fn encode_text_json(text: &str) -> String {
    wrap_result(|| {
        let phrase = WordBytes::new(English).encode_text(text)?;
        let mut out = String::from(r#"{"ok":true"#);
        push_string_field(&mut out, "mode", "text_encode");
        push_string_field(&mut out, "scheme", "word-bytes-v1");
        push_string_field(&mut out, "text", text);
        push_string_field(&mut out, "phrase", &phrase);
        push_usize_field(&mut out, "byte_length", text.len());
        out.push('}');
        Ok(out)
    })
}

/// Decodes a `word-bytes-v1` phrase into UTF-8 text.
#[wasm_bindgen]
pub fn decode_text_json(phrase: &str) -> String {
    wrap_result(|| {
        let words = split_phrase(phrase)?;
        let refs = words.iter().map(String::as_str).collect::<Vec<_>>();
        let text = WordBytes::new(English).decode_text(&refs)?;
        let mut out = String::from(r#"{"ok":true"#);
        push_string_field(&mut out, "mode", "text_decode");
        push_string_field(&mut out, "scheme", "word-bytes-v1");
        push_string_field(&mut out, "text", &text);
        push_string_field(&mut out, "phrase", &words.join(" "));
        push_usize_field(&mut out, "byte_length", text.len());
        out.push('}');
        Ok(out)
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Preset {
    name: &'static str,
    range: u128,
    words: usize,
    permutation: PermutationKind,
}

impl Preset {
    fn codec(
        self,
    ) -> Result<Positional<English, AsciiSpace, AffinePermutation>, nwords::core::Error> {
        let permutation = self.permutation.permutation(self.range)?;
        Positional::with_formatter_and_permutation(
            English,
            AsciiSpace,
            permutation,
            self.words,
            self.range,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PermutationKind {
    Identity,
    SpreadAffine { multiplier: u128, offset: u128 },
}

impl PermutationKind {
    const fn name(self) -> &'static str {
        match self {
            Self::Identity => "identity",
            Self::SpreadAffine { .. } => "spread-affine-v1",
        }
    }

    fn permutation(self, range: u128) -> Result<AffinePermutation, nwords::core::Error> {
        match self {
            Self::Identity => AffinePermutation::new(1, 0, range),
            Self::SpreadAffine { multiplier, offset } => {
                AffinePermutation::new(multiplier, offset, range)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Report {
    preset_name: Option<&'static str>,
    dictionary: &'static str,
    permutation: &'static str,
    words: usize,
    range: u128,
    capacity: CapacityClass,
}

impl Report {
    fn new(preset: Preset) -> Result<Self, WebError> {
        Self::from_parts(
            Some(preset.name),
            preset.permutation.name(),
            preset.words,
            preset.range,
        )
    }

    fn custom(words: usize, range: u128) -> Result<Self, WebError> {
        Self::from_parts(None, IDENTITY_PERMUTATION.name(), words, range)
    }

    fn from_parts(
        preset_name: Option<&'static str>,
        permutation: &'static str,
        words: usize,
        range: u128,
    ) -> Result<Self, WebError> {
        if words == 0 {
            return Err(WebError::new("words must be greater than zero"));
        }
        if range == 0 {
            return Err(WebError::new("range must be greater than zero"));
        }
        let capacity = stats::capacity_uniform(English.len(0), words)?;
        Ok(Self {
            preset_name,
            dictionary: DICTIONARY_NAME,
            permutation,
            words,
            range,
            capacity,
        })
    }

    fn representable(&self) -> bool {
        match self.capacity {
            CapacityClass::Exact(capacity) => self.range <= capacity,
            CapacityClass::BeyondU128 { .. } => true,
        }
    }

    fn capacity_string(&self) -> String {
        capacity_to_string(self.capacity)
    }

    fn slack_string(&self) -> Option<String> {
        stats::slack(self.capacity, self.range).map(|slack| slack.to_string())
    }

    fn acceptance_ratio_string(&self) -> Option<String> {
        stats::acceptance_ratio(self.capacity, self.range)
            .map(|ratio| format!("{}/{}", ratio.numerator, ratio.denominator))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WebError {
    message: String,
}

impl WebError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for WebError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl From<nwords::core::Error> for WebError {
    fn from(value: nwords::core::Error) -> Self {
        Self::new(value.to_string())
    }
}

fn wrap_result(f: impl FnOnce() -> Result<String, WebError>) -> String {
    match f() {
        Ok(output) => output,
        Err(error) => {
            let mut out = String::from(r#"{"ok":false"#);
            push_string_field(&mut out, "error", &error.to_string());
            out.push('}');
            out
        }
    }
}

fn find_preset(name: &str) -> Option<Preset> {
    PRESETS.iter().copied().find(|preset| preset.name == name)
}

fn parse_u128(input: &str, label: &str) -> Result<u128, WebError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(WebError::new(format!("{label} is required")));
    }
    trimmed
        .parse::<u128>()
        .map_err(|_| WebError::new(format!("{label} must be an unsigned decimal integer")))
}

fn parse_usize(input: &str, label: &str) -> Result<usize, WebError> {
    let value = parse_u128(input, label)?;
    usize::try_from(value).map_err(|_| WebError::new(format!("{label} value is too large")))
}

fn split_phrase(phrase: &str) -> Result<Vec<String>, WebError> {
    let words = phrase
        .split_whitespace()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if words.is_empty() {
        Err(WebError::new("phrase must contain at least one word"))
    } else {
        Ok(words)
    }
}

fn capacity_to_string(capacity: CapacityClass) -> String {
    match capacity {
        CapacityClass::Exact(capacity) => capacity.to_string(),
        CapacityClass::BeyondU128 { log2 } => format!(
            "beyond_u128(log2_lower={},log2_upper={})",
            log2.lower_bits, log2.upper_bits
        ),
    }
}

fn push_preset_json(out: &mut String, preset: &Preset, report: &Report) {
    out.push('{');
    push_first_string_field(out, "name", preset.name);
    push_string_field(out, "dictionary", report.dictionary);
    push_string_field(out, "permutation", report.permutation);
    push_usize_field(out, "words", report.words);
    push_u128_field(out, "range", report.range);
    push_string_field(out, "capacity", &report.capacity_string());
    push_string_option_field(out, "slack", report.slack_string());
    push_string_option_field(out, "acceptance_ratio", report.acceptance_ratio_string());
    push_bool_field(out, "representable", report.representable());
    out.push('}');
}

fn push_report_fields(out: &mut String, report: &Report) {
    let preset = report.preset_name.unwrap_or("custom");
    push_string_field(out, "preset", preset);
    push_string_field(out, "dictionary", report.dictionary);
    push_string_field(out, "permutation", report.permutation);
    push_usize_field(out, "words", report.words);
    push_u128_field(out, "range", report.range);
    push_string_field(out, "capacity", &report.capacity_string());
    push_bool_field(out, "representable", report.representable());
    push_string_option_field(out, "slack", report.slack_string());
    push_string_option_field(out, "acceptance_ratio", report.acceptance_ratio_string());
}

fn push_first_string_field(out: &mut String, name: &str, value: &str) {
    push_json_string(out, name);
    out.push(':');
    push_json_string(out, value);
}

fn push_string_field(out: &mut String, name: &str, value: &str) {
    out.push(',');
    push_first_string_field(out, name, value);
}

fn push_string_option_field(out: &mut String, name: &str, value: Option<String>) {
    out.push(',');
    push_json_string(out, name);
    out.push(':');
    if let Some(value) = value {
        push_json_string(out, &value);
    } else {
        out.push_str("null");
    }
}

fn push_u128_field(out: &mut String, name: &str, value: u128) {
    push_string_field(out, name, &value.to_string());
}

fn push_usize_field(out: &mut String, name: &str, value: usize) {
    out.push(',');
    push_json_string(out, name);
    out.push(':');
    out.push_str(&value.to_string());
}

fn push_bool_field(out: &mut String, name: &str, value: bool) {
    out.push(',');
    push_json_string(out, name);
    out.push(':');
    out.push_str(if value { "true" } else { "false" });
}

fn push_json_string(out: &mut String, value: &str) {
    out.push('"');
    for character in value.chars() {
        match character {
            '"' => out.push_str(r#"\""#),
            '\\' => out.push_str(r#"\\"#),
            '\n' => out.push_str(r#"\n"#),
            '\r' => out.push_str(r#"\r"#),
            '\t' => out.push_str(r#"\t"#),
            character if character <= '\u{1f}' => {
                out.push_str(&format!(r#"\u{:04x}"#, character as u32));
            }
            character => out.push(character),
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use super::{
        decode_id_json, decode_text_json, encode_id_json, encode_text_json, plan_json, presets_json,
    };

    #[test]
    fn id_phrase_round_trips_through_json_api() {
        let encoded = encode_id_json("42", "dec6");

        assert!(encoded.contains(r#""ok":true"#));
        assert!(encoded.contains(r#""phrase":"#));

        let decoded = decode_id_json("abandon aim", "dec6");

        assert!(decoded.contains(r#""ok":true"#));
        assert!(decoded.contains(r#""id":"42""#));
    }

    #[test]
    fn text_phrase_round_trips_through_json_api() {
        let encoded = encode_text_json("hello");

        assert!(encoded.contains(r#""ok":true"#));
        assert!(encoded.contains(r#""scheme":"word-bytes-v1""#));

        let decoded = decode_text_json("abandon abandon access speak fine curtain rose");

        assert!(decoded.contains(r#""ok":true"#));
        assert!(decoded.contains(r#""text":"hello""#));
    }

    #[test]
    fn presets_and_plans_report_capacity_terms() {
        let presets = presets_json();
        assert!(presets.contains(r#""name":"dec6-spread""#));
        assert!(presets.contains(r#""acceptance_ratio":"1000000/4194304""#));

        let plan = plan_json("1000000", "");
        assert!(plan.contains(r#""words":2"#));
        assert!(plan.contains(r#""slack":"3194304""#));
    }

    #[test]
    fn errors_are_json() {
        let encoded = encode_id_json("1000000", "dec6");

        assert!(encoded.contains(r#""ok":false"#));
        assert!(encoded.contains("outside range"));
    }
}
