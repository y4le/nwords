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

const DEFAULT_DICTIONARY: &str = "bip39-en-positional";
const POSITIONAL_CAVEAT: &str =
    "BIP-39 wordlist used as a positional dictionary, not a BIP-39 mnemonic.";
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
        dictionary: Dictionary::Bip39EnglishPositional,
        permutation: IDENTITY_PERMUTATION,
    },
    Preset {
        name: "dec6-spread",
        range: 1_000_000,
        words: 2,
        dictionary: Dictionary::Bip39EnglishPositional,
        permutation: DECIMAL_SPREAD,
    },
    Preset {
        name: "dec9",
        range: 1_000_000_000,
        words: 3,
        dictionary: Dictionary::Bip39EnglishPositional,
        permutation: IDENTITY_PERMUTATION,
    },
    Preset {
        name: "dec9-spread",
        range: 1_000_000_000,
        words: 3,
        dictionary: Dictionary::Bip39EnglishPositional,
        permutation: DECIMAL_SPREAD,
    },
    Preset {
        name: "u32",
        range: 1u128 << 32,
        words: 3,
        dictionary: Dictionary::Bip39EnglishPositional,
        permutation: IDENTITY_PERMUTATION,
    },
    Preset {
        name: "u32-spread",
        range: 1u128 << 32,
        words: 3,
        dictionary: Dictionary::Bip39EnglishPositional,
        permutation: U32_SPREAD,
    },
    Preset {
        name: "u64",
        range: 1u128 << 64,
        words: 6,
        dictionary: Dictionary::Bip39EnglishPositional,
        permutation: IDENTITY_PERMUTATION,
    },
    Preset {
        name: "u64-spread",
        range: 1u128 << 64,
        words: 6,
        dictionary: Dictionary::Bip39EnglishPositional,
        permutation: U64_SPREAD,
    },
    Preset {
        name: "dec18",
        range: 1_000_000_000_000_000_000,
        words: 6,
        dictionary: Dictionary::Bip39EnglishPositional,
        permutation: IDENTITY_PERMUTATION,
    },
    Preset {
        name: "dec18-spread",
        range: 1_000_000_000_000_000_000,
        words: 6,
        dictionary: Dictionary::Bip39EnglishPositional,
        permutation: DEC18_SPREAD,
    },
];

/// Captured CLI output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliOutput {
    /// Process exit code.
    pub exit_code: i32,
    /// Data written to stdout.
    pub stdout: String,
    /// Data written to stderr.
    pub stderr: String,
}

/// Runs the CLI with an argument iterator.
pub fn run<I, S>(args: I) -> CliOutput
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let args = args.into_iter().map(Into::into).collect::<Vec<_>>();
    match execute(&args) {
        Ok(stdout) => CliOutput {
            exit_code: 0,
            stdout,
            stderr: String::new(),
        },
        Err(error) => CliOutput {
            exit_code: error.exit_code(),
            stdout: String::new(),
            stderr: format!("{error}\n"),
        },
    }
}

fn execute(args: &[String]) -> Result<String, CliError> {
    let Some(command) = args.first().map(String::as_str) else {
        return Err(CliError::usage(HELP));
    };

    match command {
        "-h" | "--help" | "help" => Ok(HELP.to_owned()),
        "encode" => encode(&args[1..]),
        "decode" => decode(&args[1..]),
        "presets" => presets(&args[1..]),
        "plan" => plan(&args[1..]),
        "bytes" => bytes_command(&args[1..]),
        "text" => text_command(&args[1..]),
        _ => Err(CliError::usage(format!(
            "unknown command `{command}`\n\n{HELP}"
        ))),
    }
}

fn encode(args: &[String]) -> Result<String, CliError> {
    let parsed = ParsedArgs::parse(args)?;
    if parsed.help {
        return Ok(ENCODE_HELP.to_owned());
    }
    if parsed.byte_mode.is_some() {
        return Err(CliError::usage(ENCODE_HELP));
    }
    if parsed.positionals.len() != 1 {
        return Err(CliError::usage(ENCODE_HELP));
    }

    let id = parse_decimal_u128(&parsed.positionals[0], "id")?;
    let shape = Shape::resolve(&parsed)?;
    let phrase = shape
        .codec()
        .and_then(|codec| codec.encode(id))
        .map_err(runtime_error)?;

    if parsed.explain {
        Ok(explain_encode(&shape, id, &phrase)?)
    } else {
        Ok(format!("{phrase}\n"))
    }
}

fn decode(args: &[String]) -> Result<String, CliError> {
    let parsed = ParsedArgs::parse(args)?;
    if parsed.help {
        return Ok(DECODE_HELP.to_owned());
    }
    if parsed.byte_mode.is_some() {
        return Err(CliError::usage(DECODE_HELP));
    }

    let words = split_words(&parsed.positionals);
    if words.is_empty() {
        return Err(CliError::usage(DECODE_HELP));
    }

    let shape = Shape::resolve(&parsed)?;
    let refs = words.iter().map(String::as_str).collect::<Vec<_>>();
    let id = shape
        .codec()
        .and_then(|codec| codec.decode_words(&refs))
        .map_err(runtime_error)?;

    if parsed.explain {
        Ok(explain_decode(&shape, id, &words.join(" "))?)
    } else {
        Ok(format!("{id}\n"))
    }
}

fn presets(args: &[String]) -> Result<String, CliError> {
    let parsed = ParsedArgs::parse(args)?;
    if parsed.help {
        return Ok(PRESETS_HELP.to_owned());
    }
    if !parsed.positionals.is_empty()
        || parsed.has_shape_options()
        || parsed.byte_mode.is_some()
        || parsed.explain
    {
        return Err(CliError::usage(PRESETS_HELP));
    }

    let mut output = String::new();
    output.push_str(POSITIONAL_CAVEAT);
    output.push('\n');
    output.push_str(
        "name\tdictionary\tpermutation\twords\trange\tcapacity\tslack\tacceptance_ratio\n",
    );
    for preset in PRESETS {
        let report = Report::new(
            preset.dictionary,
            preset.permutation,
            preset.words,
            preset.range,
        )?;
        output.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            preset.name,
            preset.dictionary.name(),
            preset.permutation.name(),
            preset.words,
            preset.range,
            report.capacity,
            report.slack(),
            report.acceptance_ratio()
        ));
    }
    Ok(output)
}

fn plan(args: &[String]) -> Result<String, CliError> {
    let parsed = ParsedArgs::parse(args)?;
    if parsed.help {
        return Ok(PLAN_HELP.to_owned());
    }
    if !parsed.positionals.is_empty() || parsed.byte_mode.is_some() || parsed.explain {
        return Err(CliError::usage(PLAN_HELP));
    }

    if let Some(name) = parsed.preset.as_deref() {
        if parsed.range.is_some() || parsed.words.is_some() || parsed.dictionary.is_some() {
            return Err(CliError::usage(
                "do not combine --preset with --range, --words, or --dict",
            ));
        }
        let preset =
            find_preset(name).ok_or_else(|| CliError::usage(format!("unknown preset `{name}`")))?;
        let report = Report::new(
            preset.dictionary,
            preset.permutation,
            preset.words,
            preset.range,
        )?;
        return Ok(format_report_fields("plan", &report, None, None));
    }

    let dictionary = parsed
        .dictionary
        .as_deref()
        .map(Dictionary::parse)
        .transpose()?
        .unwrap_or(Dictionary::Bip39EnglishPositional);
    let range = parsed
        .range
        .ok_or_else(|| CliError::usage("plan requires --range <R>"))?;
    if range == 0 {
        return Err(CliError::usage("range must be greater than zero"));
    }

    let words = if let Some(words) = parsed.words {
        words
    } else {
        match stats::required_words(PlanTarget::Range(range), dictionary.len()) {
            Ok(PlanSolution::RequiredWords { word_count, .. }) => word_count,
            Ok(_) => return Err(CliError::runtime("unexpected stats planner result")),
            Err(error) => return Err(runtime_error(error)),
        }
    };

    let report = Report::new(dictionary, IDENTITY_PERMUTATION, words, range)?;
    Ok(format_report_fields("plan", &report, None, None))
}

fn bytes_command(args: &[String]) -> Result<String, CliError> {
    let Some(command) = args.first().map(String::as_str) else {
        return Err(CliError::usage(BYTES_HELP));
    };

    match command {
        "-h" | "--help" | "help" => Ok(BYTES_HELP.to_owned()),
        "encode" => bytes_encode(&args[1..]),
        "decode" => bytes_decode(&args[1..]),
        _ => Err(CliError::usage(BYTES_HELP)),
    }
}

fn bytes_encode(args: &[String]) -> Result<String, CliError> {
    let parsed = ParsedArgs::parse(args)?;
    if parsed.help {
        return Ok(BYTES_ENCODE_HELP.to_owned());
    }
    reject_shape_options(&parsed, BYTES_ENCODE_HELP)?;
    if parsed.positionals.is_empty() {
        return Err(CliError::usage(BYTES_ENCODE_HELP));
    }

    let mode = parsed
        .byte_mode
        .ok_or_else(|| CliError::usage("bytes encode requires --text or --hex"))?;
    let bytes = match mode {
        ByteMode::Text => join_positionals(&parsed.positionals).into_bytes(),
        ByteMode::Hex => {
            if parsed.positionals.len() != 1 {
                return Err(CliError::usage("bytes encode --hex requires one hex value"));
            }
            parse_hex(&parsed.positionals[0])?
        }
    };
    let phrase = word_bytes_codec()
        .encode_bytes(&bytes)
        .map_err(runtime_error)?;
    Ok(format!("{phrase}\n"))
}

fn bytes_decode(args: &[String]) -> Result<String, CliError> {
    let parsed = ParsedArgs::parse(args)?;
    if parsed.help {
        return Ok(BYTES_DECODE_HELP.to_owned());
    }
    reject_shape_options(&parsed, BYTES_DECODE_HELP)?;

    let words = split_words(&parsed.positionals);
    if words.is_empty() {
        return Err(CliError::usage(BYTES_DECODE_HELP));
    }

    let mode = parsed
        .byte_mode
        .ok_or_else(|| CliError::usage("bytes decode requires --text or --hex"))?;
    let refs = words.iter().map(String::as_str).collect::<Vec<_>>();
    match mode {
        ByteMode::Text => {
            let text = word_bytes_codec()
                .decode_text(&refs)
                .map_err(runtime_error)?;
            Ok(format!("{text}\n"))
        }
        ByteMode::Hex => {
            let bytes = word_bytes_codec()
                .decode_words(&refs)
                .map_err(runtime_error)?;
            Ok(format!("{}\n", format_hex(&bytes)))
        }
    }
}

fn text_command(args: &[String]) -> Result<String, CliError> {
    let Some(command) = args.first().map(String::as_str) else {
        return Err(CliError::usage(TEXT_HELP));
    };

    match command {
        "-h" | "--help" | "help" => Ok(TEXT_HELP.to_owned()),
        "encode" => text_encode(&args[1..]),
        "decode" => text_decode(&args[1..]),
        _ => Err(CliError::usage(TEXT_HELP)),
    }
}

fn text_encode(args: &[String]) -> Result<String, CliError> {
    let parsed = ParsedArgs::parse(args)?;
    if parsed.help {
        return Ok(TEXT_ENCODE_HELP.to_owned());
    }
    reject_shape_options(&parsed, TEXT_ENCODE_HELP)?;
    if parsed.byte_mode.is_some() || parsed.positionals.is_empty() {
        return Err(CliError::usage(TEXT_ENCODE_HELP));
    }

    let text = join_positionals(&parsed.positionals);
    let phrase = word_bytes_codec()
        .encode_text(&text)
        .map_err(runtime_error)?;
    Ok(format!("{phrase}\n"))
}

fn text_decode(args: &[String]) -> Result<String, CliError> {
    let parsed = ParsedArgs::parse(args)?;
    if parsed.help {
        return Ok(TEXT_DECODE_HELP.to_owned());
    }
    reject_shape_options(&parsed, TEXT_DECODE_HELP)?;
    if parsed.byte_mode.is_some() {
        return Err(CliError::usage(TEXT_DECODE_HELP));
    }

    let words = split_words(&parsed.positionals);
    if words.is_empty() {
        return Err(CliError::usage(TEXT_DECODE_HELP));
    }

    let refs = words.iter().map(String::as_str).collect::<Vec<_>>();
    let text = word_bytes_codec()
        .decode_text(&refs)
        .map_err(runtime_error)?;
    Ok(format!("{text}\n"))
}

fn explain_encode(shape: &Shape, id: u128, phrase: &str) -> Result<String, CliError> {
    let report = Report::new(
        shape.dictionary,
        shape.permutation,
        shape.words,
        shape.range,
    )?;
    Ok(format_report_fields(
        "encode",
        &report,
        Some(id),
        Some(phrase),
    ))
}

fn explain_decode(shape: &Shape, id: u128, phrase: &str) -> Result<String, CliError> {
    let report = Report::new(
        shape.dictionary,
        shape.permutation,
        shape.words,
        shape.range,
    )?;
    Ok(format_report_fields(
        "decode",
        &report,
        Some(id),
        Some(phrase),
    ))
}

fn format_report_fields(
    mode: &str,
    report: &Report,
    id: Option<u128>,
    phrase: Option<&str>,
) -> String {
    let mut output = String::new();
    output.push_str(POSITIONAL_CAVEAT);
    output.push('\n');
    output.push_str(&format!("mode: {mode}\n"));
    if let Some(preset) = report.preset_name {
        output.push_str(&format!("preset: {preset}\n"));
    } else {
        output.push_str("preset: custom\n");
    }
    output.push_str(&format!("dictionary: {}\n", report.dictionary.name()));
    output.push_str(&format!("permutation: {}\n", report.permutation.name()));
    output.push_str(&format!("words: {}\n", report.words));
    output.push_str(&format!("range: {}\n", report.range));
    output.push_str(&format!("capacity: {}\n", report.capacity));
    output.push_str(&format!("representable: {}\n", report.representable()));
    output.push_str(&format!("slack: {}\n", report.slack()));
    output.push_str(&format!(
        "acceptance_ratio: {}\n",
        report.acceptance_ratio()
    ));
    if let Some(id) = id {
        output.push_str(&format!("id: {id}\n"));
    }
    if let Some(phrase) = phrase {
        output.push_str(&format!("phrase: {phrase}\n"));
    }
    output
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Preset {
    name: &'static str,
    range: u128,
    words: usize,
    dictionary: Dictionary,
    permutation: PermutationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Dictionary {
    Bip39EnglishPositional,
}

impl Dictionary {
    fn parse(name: &str) -> Result<Self, CliError> {
        match name {
            DEFAULT_DICTIONARY => Ok(Self::Bip39EnglishPositional),
            _ => Err(CliError::usage(format!("unknown dictionary `{name}`"))),
        }
    }

    const fn name(self) -> &'static str {
        match self {
            Self::Bip39EnglishPositional => DEFAULT_DICTIONARY,
        }
    }

    fn len(self) -> usize {
        match self {
            Self::Bip39EnglishPositional => English.len(0),
        }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ByteMode {
    Text,
    Hex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Shape {
    range: u128,
    words: usize,
    dictionary: Dictionary,
    permutation: PermutationKind,
    preset_name: Option<&'static str>,
}

impl Shape {
    fn resolve(parsed: &ParsedArgs) -> Result<Self, CliError> {
        if let Some(name) = parsed.preset.as_deref() {
            if parsed.range.is_some() || parsed.words.is_some() || parsed.dictionary.is_some() {
                return Err(CliError::usage(
                    "do not combine --preset with --range, --words, or --dict",
                ));
            }
            let preset = find_preset(name)
                .ok_or_else(|| CliError::usage(format!("unknown preset `{name}`")))?;
            return Ok(Self {
                range: preset.range,
                words: preset.words,
                dictionary: preset.dictionary,
                permutation: preset.permutation,
                preset_name: Some(preset.name),
            });
        }

        let range = parsed.range.ok_or_else(|| {
            CliError::usage("use --preset <name> or provide --range <R> --words <N>")
        })?;
        let words = parsed.words.ok_or_else(|| {
            CliError::usage("use --preset <name> or provide --range <R> --words <N>")
        })?;
        let dictionary = parsed
            .dictionary
            .as_deref()
            .map(Dictionary::parse)
            .transpose()?
            .unwrap_or(Dictionary::Bip39EnglishPositional);

        Ok(Self {
            range,
            words,
            dictionary,
            permutation: IDENTITY_PERMUTATION,
            preset_name: None,
        })
    }

    fn codec(
        self,
    ) -> Result<Positional<English, AsciiSpace, AffinePermutation>, nwords::core::Error> {
        let permutation = self.permutation.permutation(self.range)?;
        match self.dictionary {
            Dictionary::Bip39EnglishPositional => Positional::with_formatter_and_permutation(
                English,
                AsciiSpace,
                permutation,
                self.words,
                self.range,
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Report {
    dictionary: Dictionary,
    permutation: PermutationKind,
    words: usize,
    range: u128,
    capacity: CapacityDisplay,
    preset_name: Option<&'static str>,
}

impl Report {
    fn new(
        dictionary: Dictionary,
        permutation: PermutationKind,
        words: usize,
        range: u128,
    ) -> Result<Self, CliError> {
        if words == 0 {
            return Err(CliError::usage("words must be greater than zero"));
        }
        if range == 0 {
            return Err(CliError::usage("range must be greater than zero"));
        }
        let capacity = stats::capacity_uniform(dictionary.len(), words).map_err(runtime_error)?;
        Ok(Self {
            dictionary,
            permutation,
            words,
            range,
            capacity: CapacityDisplay(capacity),
            preset_name: find_preset_by_shape(dictionary, permutation, words, range)
                .map(|preset| preset.name),
        })
    }

    fn representable(self) -> &'static str {
        match self.capacity.0 {
            CapacityClass::Exact(capacity) if self.range <= capacity => "yes",
            CapacityClass::Exact(_) => "no",
            CapacityClass::BeyondU128 { .. } => "yes",
        }
    }

    fn slack(self) -> String {
        stats::slack(self.capacity.0, self.range)
            .map(|slack| slack.to_string())
            .unwrap_or_else(|| "n/a".to_owned())
    }

    fn acceptance_ratio(self) -> String {
        stats::acceptance_ratio(self.capacity.0, self.range)
            .map(|ratio| format!("{}/{}", ratio.numerator, ratio.denominator))
            .unwrap_or_else(|| "n/a".to_owned())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CapacityDisplay(CapacityClass);

impl fmt::Display for CapacityDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            CapacityClass::Exact(capacity) => write!(f, "{capacity}"),
            CapacityClass::BeyondU128 { log2 } => write!(
                f,
                "beyond_u128(log2_lower={},log2_upper={})",
                log2.lower_bits, log2.upper_bits
            ),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct ParsedArgs {
    positionals: Vec<String>,
    preset: Option<String>,
    range: Option<u128>,
    words: Option<usize>,
    dictionary: Option<String>,
    byte_mode: Option<ByteMode>,
    explain: bool,
    help: bool,
}

impl ParsedArgs {
    fn parse(args: &[String]) -> Result<Self, CliError> {
        let mut parsed = Self::default();
        let mut index = 0usize;
        let mut flags_enabled = true;

        while index < args.len() {
            let arg = &args[index];
            if flags_enabled && arg == "--" {
                flags_enabled = false;
                index += 1;
                continue;
            }

            if flags_enabled && arg.starts_with("--") {
                match arg.as_str() {
                    "--help" => parsed.help = true,
                    "--explain" => parsed.explain = true,
                    "--text" => set_once(&mut parsed.byte_mode, "--text/--hex", ByteMode::Text)?,
                    "--hex" => set_once(&mut parsed.byte_mode, "--text/--hex", ByteMode::Hex)?,
                    "--preset" => {
                        index += 1;
                        let value = args
                            .get(index)
                            .ok_or_else(|| CliError::usage("--preset requires a value"))?;
                        set_once(&mut parsed.preset, "--preset", value.clone())?;
                    }
                    "--range" => {
                        index += 1;
                        let value = args
                            .get(index)
                            .ok_or_else(|| CliError::usage("--range requires a value"))?;
                        let range = parse_decimal_u128(value, "range")?;
                        set_once(&mut parsed.range, "--range", range)?;
                    }
                    "--words" => {
                        index += 1;
                        let value = args
                            .get(index)
                            .ok_or_else(|| CliError::usage("--words requires a value"))?;
                        let words_u128 = parse_decimal_u128(value, "words")?;
                        let words = usize::try_from(words_u128)
                            .map_err(|_| CliError::usage("words value is too large"))?;
                        set_once(&mut parsed.words, "--words", words)?;
                    }
                    "--dict" => {
                        index += 1;
                        let value = args
                            .get(index)
                            .ok_or_else(|| CliError::usage("--dict requires a value"))?;
                        set_once(&mut parsed.dictionary, "--dict", value.clone())?;
                    }
                    _ => return Err(CliError::usage(format!("unknown flag `{arg}`"))),
                }
            } else {
                parsed.positionals.push(arg.clone());
            }
            index += 1;
        }

        Ok(parsed)
    }

    fn has_shape_options(&self) -> bool {
        self.preset.is_some()
            || self.range.is_some()
            || self.words.is_some()
            || self.dictionary.is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CliError {
    Usage(String),
    Runtime(String),
}

impl CliError {
    fn usage(message: impl Into<String>) -> Self {
        Self::Usage(message.into())
    }

    fn runtime(message: impl Into<String>) -> Self {
        Self::Runtime(message.into())
    }

    const fn exit_code(&self) -> i32 {
        match self {
            Self::Usage(_) => 2,
            Self::Runtime(_) => 1,
        }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(message) | Self::Runtime(message) => f.write_str(message),
        }
    }
}

fn runtime_error(error: nwords::core::Error) -> CliError {
    CliError::runtime(format!("conversion failed: {error}"))
}

fn reject_shape_options(parsed: &ParsedArgs, help: &'static str) -> Result<(), CliError> {
    if parsed.has_shape_options() || parsed.explain {
        Err(CliError::usage(help))
    } else {
        Ok(())
    }
}

fn set_once<T>(slot: &mut Option<T>, name: &str, value: T) -> Result<(), CliError> {
    if slot.is_some() {
        return Err(CliError::usage(format!("duplicate {name}")));
    }
    *slot = Some(value);
    Ok(())
}

fn parse_decimal_u128(value: &str, name: &str) -> Result<u128, CliError> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(CliError::usage(format!(
            "invalid {name}; expected decimal digits"
        )));
    }
    value
        .parse::<u128>()
        .map_err(|_| CliError::usage(format!("{name} value is too large")))
}

fn split_words(positionals: &[String]) -> Vec<String> {
    positionals
        .iter()
        .flat_map(|arg| arg.split_whitespace())
        .map(str::to_owned)
        .collect()
}

fn join_positionals(positionals: &[String]) -> String {
    positionals.join(" ")
}

fn word_bytes_codec() -> WordBytes<English> {
    WordBytes::new(English)
}

fn parse_hex(value: &str) -> Result<Vec<u8>, CliError> {
    if value.len() % 2 != 0 {
        return Err(CliError::usage("hex input must have an even length"));
    }

    value
        .as_bytes()
        .chunks_exact(2)
        .map(|chunk| {
            let high = hex_value(chunk[0])?;
            let low = hex_value(chunk[1])?;
            Ok((high << 4) | low)
        })
        .collect()
}

fn hex_value(byte: u8) -> Result<u8, CliError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(CliError::usage("hex input contains a non-hex digit")),
    }
}

fn format_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        output.push(char::from(HEX[(byte >> 4) as usize]));
        output.push(char::from(HEX[(byte & 0x0f) as usize]));
    }
    output
}

fn find_preset(name: &str) -> Option<&'static Preset> {
    PRESETS.iter().find(|preset| preset.name == name)
}

fn find_preset_by_shape(
    dictionary: Dictionary,
    permutation: PermutationKind,
    words: usize,
    range: u128,
) -> Option<&'static Preset> {
    PRESETS.iter().find(|preset| {
        preset.dictionary == dictionary
            && preset.permutation == permutation
            && preset.words == words
            && preset.range == range
    })
}

const HELP: &str = "\
nwords: positional ID phrase converter

USAGE:
    nwords encode <id> (--preset <name> | --range <R> --words <N> [--dict <name>]) [--explain]
    nwords decode <words...> (--preset <name> | --range <R> --words <N> [--dict <name>]) [--explain]
    nwords presets
    nwords plan (--preset <name> | --range <R> [--words <N>] [--dict <name>])
    nwords bytes encode (--text <text> | --hex <hex>)
    nwords bytes decode (--text | --hex) <words...>
    nwords text encode <text>
    nwords text decode <words...>

BIP-39 wordlist used as a positional dictionary, not a BIP-39 mnemonic.
";

const ENCODE_HELP: &str = "\
USAGE:
    nwords encode <id> (--preset <name> | --range <R> --words <N> [--dict <name>]) [--explain]
";

const DECODE_HELP: &str = "\
USAGE:
    nwords decode <words...> (--preset <name> | --range <R> --words <N> [--dict <name>]) [--explain]
";

const PRESETS_HELP: &str = "\
USAGE:
    nwords presets
";

const PLAN_HELP: &str = "\
USAGE:
    nwords plan (--preset <name> | --range <R> [--words <N>] [--dict <name>])
";

const BYTES_HELP: &str = "\
USAGE:
    nwords bytes encode (--text <text> | --hex <hex>)
    nwords bytes decode (--text | --hex) <words...>

BIP-39 wordlist used as a positional dictionary, not a BIP-39 mnemonic.
";

const BYTES_ENCODE_HELP: &str = "\
USAGE:
    nwords bytes encode (--text <text> | --hex <hex>)
";

const BYTES_DECODE_HELP: &str = "\
USAGE:
    nwords bytes decode (--text | --hex) <words...>
";

const TEXT_HELP: &str = "\
USAGE:
    nwords text encode <text>
    nwords text decode <words...>

BIP-39 wordlist used as a positional dictionary, not a BIP-39 mnemonic.
";

const TEXT_ENCODE_HELP: &str = "\
USAGE:
    nwords text encode <text>
";

const TEXT_DECODE_HELP: &str = "\
USAGE:
    nwords text decode <words...>
";

#[cfg(test)]
mod tests {
    use super::{run, POSITIONAL_CAVEAT};

    #[test]
    fn encode_and_decode_round_trip_u32() {
        let encoded = run(["encode", "42", "--preset", "u32"]);
        assert_eq!(encoded.exit_code, 0);

        let words = encoded.stdout.trim().split(' ').collect::<Vec<_>>();
        assert_eq!(words.len(), 3);

        let decoded = run(["decode", encoded.stdout.trim(), "--preset", "u32"]);
        assert_eq!(decoded.exit_code, 0);
        assert_eq!(decoded.stdout, "42\n");
    }

    #[test]
    fn flags_may_appear_before_or_after_decode_words() {
        let encoded = run(["encode", "42", "--preset", "u32"]);
        let phrase = encoded.stdout.trim();

        assert_eq!(run(["decode", "--preset", "u32", phrase]).stdout, "42\n");
        assert_eq!(run(["decode", phrase, "--preset", "u32"]).stdout, "42\n");
    }

    #[test]
    fn unknown_words_are_not_echoed() {
        let output = run(["decode", "abandon", "zzzzz", "abandon", "--preset", "u32"]);

        assert_eq!(output.exit_code, 1);
        assert!(output.stderr.contains("unknown word at position 1"));
        assert!(!output.stderr.contains("zzzzz"));
    }

    #[test]
    fn explain_is_key_value_output() {
        let output = run(["encode", "42", "--preset", "u32", "--explain"]);

        assert_eq!(output.exit_code, 0);
        assert!(output.stdout.contains(POSITIONAL_CAVEAT));
        assert!(output.stdout.contains("mode: encode\n"));
        assert!(output.stdout.contains("preset: u32\n"));
        assert!(output.stdout.contains("id: 42\n"));
        assert!(output.stdout.contains("phrase: "));
    }
}
