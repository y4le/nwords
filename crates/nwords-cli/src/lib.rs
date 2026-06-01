#![forbid(unsafe_code)]

use std::fmt;

use clap::{Args, Parser, Subcommand};
use nwords::{
    core::{AffinePermutation, WordMap},
    positional::MixedPositional,
    schemes::AsciiSpace,
    stats::{self, CapacityClass, PlanSolution, PlanTarget},
    word_bytes::WordBytes,
    wordlists::{
        bip39::English,
        named::{NamedWordList, WordListSequence},
    },
};

const DEFAULT_DICTIONARY: &str = "bip39-en-positional";
const ADJECTIVE_ANIMAL_DICTIONARY: &str = "adjective-animal";
const DEFAULT_LIST_NAME: &str = "bip39-en";
const BIP39_POSITIONAL_CAVEAT: &str =
    "BIP-39 wordlist used as a positional dictionary, not a BIP-39 mnemonic.";
const NAMED_SHAPE_CAVEAT: &str =
    "Named word-list phrases are deterministic positional IDs, not random names.";
const PRESETS_CAVEAT: &str =
    "Presets encode deterministic positional ID phrases; shape order is part of the decoding contract.";
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

const DEC6_SHAPE: &[NamedWordList] = &[NamedWordList::Bip39English, NamedWordList::Bip39English];
const DEC9_SHAPE: &[NamedWordList] = &[
    NamedWordList::Bip39English,
    NamedWordList::Bip39English,
    NamedWordList::Bip39English,
];
const DEC18_SHAPE: &[NamedWordList] = &[
    NamedWordList::Bip39English,
    NamedWordList::Bip39English,
    NamedWordList::Bip39English,
    NamedWordList::Bip39English,
    NamedWordList::Bip39English,
    NamedWordList::Bip39English,
];
const ADJECTIVE_ANIMAL_SHAPE: &[NamedWordList] = &[NamedWordList::Adjective, NamedWordList::Animal];
const COLOR_ADJECTIVE_ANIMAL_SHAPE: &[NamedWordList] = &[
    NamedWordList::Color,
    NamedWordList::Adjective,
    NamedWordList::Animal,
];
const DESCRIPTOR_OBJECT_SHAPE: &[NamedWordList] =
    &[NamedWordList::Descriptor, NamedWordList::Object];
const COLOR_DESCRIPTOR_OBJECT_SHAPE: &[NamedWordList] = &[
    NamedWordList::Color,
    NamedWordList::Descriptor,
    NamedWordList::Object,
];

const PRESETS: &[Preset] = &[
    Preset {
        name: "dec6",
        range: 1_000_000,
        shape: DEC6_SHAPE,
        permutation: IDENTITY_PERMUTATION,
    },
    Preset {
        name: "dec6-spread",
        range: 1_000_000,
        shape: DEC6_SHAPE,
        permutation: DECIMAL_SPREAD,
    },
    Preset {
        name: "dec9",
        range: 1_000_000_000,
        shape: DEC9_SHAPE,
        permutation: IDENTITY_PERMUTATION,
    },
    Preset {
        name: "dec9-spread",
        range: 1_000_000_000,
        shape: DEC9_SHAPE,
        permutation: DECIMAL_SPREAD,
    },
    Preset {
        name: "u32",
        range: 1u128 << 32,
        shape: DEC9_SHAPE,
        permutation: IDENTITY_PERMUTATION,
    },
    Preset {
        name: "u32-spread",
        range: 1u128 << 32,
        shape: DEC9_SHAPE,
        permutation: U32_SPREAD,
    },
    Preset {
        name: "u64",
        range: 1u128 << 64,
        shape: DEC18_SHAPE,
        permutation: IDENTITY_PERMUTATION,
    },
    Preset {
        name: "u64-spread",
        range: 1u128 << 64,
        shape: DEC18_SHAPE,
        permutation: U64_SPREAD,
    },
    Preset {
        name: "dec18",
        range: 1_000_000_000_000_000_000,
        shape: DEC18_SHAPE,
        permutation: IDENTITY_PERMUTATION,
    },
    Preset {
        name: "dec18-spread",
        range: 1_000_000_000_000_000_000,
        shape: DEC18_SHAPE,
        permutation: DEC18_SPREAD,
    },
    Preset {
        name: "aa",
        range: 249_417,
        shape: ADJECTIVE_ANIMAL_SHAPE,
        permutation: IDENTITY_PERMUTATION,
    },
    Preset {
        name: "dec5-aa",
        range: 100_000,
        shape: ADJECTIVE_ANIMAL_SHAPE,
        permutation: IDENTITY_PERMUTATION,
    },
    Preset {
        name: "color-aa",
        range: 12_969_684,
        shape: COLOR_ADJECTIVE_ANIMAL_SHAPE,
        permutation: IDENTITY_PERMUTATION,
    },
    Preset {
        name: "descriptor-object",
        range: 4_384_287,
        shape: DESCRIPTOR_OBJECT_SHAPE,
        permutation: IDENTITY_PERMUTATION,
    },
    Preset {
        name: "color-descriptor-object",
        range: 227_982_924,
        shape: COLOR_DESCRIPTOR_OBJECT_SHAPE,
        permutation: IDENTITY_PERMUTATION,
    },
];

#[derive(Debug, Parser)]
#[command(
    name = "nwords",
    disable_help_flag = true,
    disable_help_subcommand = true
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
#[command(disable_help_flag = true, disable_help_subcommand = true)]
enum Command {
    /// Encode an integer ID into a deterministic word phrase.
    Encode(EncodeArgs),
    /// Decode a deterministic word phrase back into an integer ID.
    Decode(DecodeArgs),
    /// List built-in positional ID presets.
    Presets,
    /// Show capacity and range statistics.
    Plan(PlanArgs),
    /// Encode or decode arbitrary bytes with word-bytes-v1.
    Bytes(NestedBytesArgs),
    /// Encode or decode UTF-8 text with word-bytes-v1.
    Text(NestedTextArgs),
}

#[derive(Debug, Args)]
#[command(disable_help_flag = true)]
struct EncodeArgs {
    id: String,
    #[command(flatten)]
    shape: ShapeOptions,
    #[arg(long)]
    explain: bool,
}

#[derive(Debug, Args)]
#[command(disable_help_flag = true)]
struct DecodeArgs {
    #[arg(id = "phrase_words", num_args = 1..)]
    words: Vec<String>,
    #[command(flatten)]
    shape: ShapeOptions,
    #[arg(long)]
    explain: bool,
}

#[derive(Debug, Args)]
#[command(disable_help_flag = true)]
struct PlanArgs {
    #[command(flatten)]
    shape: ShapeOptions,
}

#[derive(Debug, Args)]
#[command(disable_help_flag = true, disable_help_subcommand = true)]
struct NestedBytesArgs {
    #[command(subcommand)]
    command: Option<BytesCommand>,
}

#[derive(Debug, Subcommand)]
#[command(disable_help_flag = true)]
enum BytesCommand {
    /// Encode text or hex bytes with word-bytes-v1.
    Encode(BytesEncodeArgs),
    /// Decode a word-bytes-v1 phrase to text or hex.
    Decode(BytesDecodeArgs),
}

#[derive(Debug, Args)]
#[command(disable_help_flag = true)]
struct BytesEncodeArgs {
    #[arg(long)]
    text: bool,
    #[arg(long)]
    hex: bool,
    #[arg(num_args = 1..)]
    data: Vec<String>,
}

#[derive(Debug, Args)]
#[command(disable_help_flag = true)]
struct BytesDecodeArgs {
    #[arg(long)]
    text: bool,
    #[arg(long)]
    hex: bool,
    #[arg(id = "phrase_words", num_args = 1..)]
    words: Vec<String>,
}

#[derive(Debug, Args)]
#[command(disable_help_flag = true, disable_help_subcommand = true)]
struct NestedTextArgs {
    #[command(subcommand)]
    command: Option<TextCommand>,
}

#[derive(Debug, Subcommand)]
#[command(disable_help_flag = true)]
enum TextCommand {
    /// Encode UTF-8 text with word-bytes-v1.
    Encode(TextEncodeArgs),
    /// Decode a word-bytes-v1 phrase into UTF-8 text.
    Decode(TextDecodeArgs),
}

#[derive(Debug, Args)]
#[command(disable_help_flag = true)]
struct TextEncodeArgs {
    #[arg(num_args = 1..)]
    text: Vec<String>,
}

#[derive(Debug, Args)]
#[command(disable_help_flag = true)]
struct TextDecodeArgs {
    #[arg(id = "phrase_words", num_args = 1..)]
    words: Vec<String>,
}

#[derive(Debug, Args, Default, Clone, PartialEq, Eq)]
#[command(disable_help_flag = true)]
struct ShapeOptions {
    #[arg(long)]
    preset: Option<String>,
    #[arg(long, value_name = "R")]
    range: Option<String>,
    #[arg(long, value_name = "N")]
    words: Option<String>,
    #[arg(long)]
    shape: Option<String>,
    #[arg(long = "dict")]
    dictionary: Option<String>,
}

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
    if let Some(help) = custom_help(args)? {
        return Ok(help);
    }

    let cli =
        Cli::try_parse_from(core::iter::once("nwords".to_owned()).chain(args.iter().cloned()))
            .map_err(|error| CliError::usage(error.to_string()))?;

    dispatch(cli)
}

fn dispatch(cli: Cli) -> Result<String, CliError> {
    match cli.command {
        Some(Command::Encode(args)) => encode(args),
        Some(Command::Decode(args)) => decode(args),
        Some(Command::Presets) => presets(),
        Some(Command::Plan(args)) => plan(args),
        Some(Command::Bytes(args)) => bytes_command(args),
        Some(Command::Text(args)) => text_command(args),
        None => Err(CliError::usage(HELP)),
    }
}

fn custom_help(args: &[String]) -> Result<Option<String>, CliError> {
    let Some(command) = args.first().map(String::as_str) else {
        return Ok(None);
    };

    if command == "-h" || command == "--help" {
        return Ok(Some(HELP.to_owned()));
    }
    if command == "help" {
        return help(&args[1..]).map(Some);
    }
    let args_before_separator = args
        .iter()
        .skip(1)
        .take_while(|arg| arg.as_str() != "--")
        .collect::<Vec<_>>();
    if args_before_separator
        .iter()
        .any(|arg| arg.as_str() == "-h" || arg.as_str() == "--help")
    {
        let help_args = if (command == "bytes" || command == "text") && args.len() > 1 {
            let subcommand = args_before_separator
                .iter()
                .find(|arg| !arg.starts_with('-'))
                .map(|arg| arg.as_str());
            if let Some(subcommand) = subcommand {
                vec![command.to_owned(), subcommand.to_owned()]
            } else {
                vec![command.to_owned()]
            }
        } else {
            vec![command.to_owned()]
        };
        return help(&help_args).map(Some);
    }

    Ok(None)
}

fn help(args: &[String]) -> Result<String, CliError> {
    match args {
        [] => Ok(HELP.to_owned()),
        [command] => match command.as_str() {
            "encode" => Ok(ENCODE_HELP.to_owned()),
            "decode" => Ok(DECODE_HELP.to_owned()),
            "presets" => Ok(PRESETS_HELP.to_owned()),
            "plan" => Ok(PLAN_HELP.to_owned()),
            "bytes" => Ok(BYTES_HELP.to_owned()),
            "text" => Ok(TEXT_HELP.to_owned()),
            _ => Err(CliError::usage(format!(
                "unknown help topic `{command}`\n\n{HELP}"
            ))),
        },
        [group, command] if group == "bytes" => match command.as_str() {
            "encode" => Ok(BYTES_ENCODE_HELP.to_owned()),
            "decode" => Ok(BYTES_DECODE_HELP.to_owned()),
            _ => Err(CliError::usage(format!(
                "unknown help topic `bytes {command}`\n\n{BYTES_HELP}"
            ))),
        },
        [group, command] if group == "text" => match command.as_str() {
            "encode" => Ok(TEXT_ENCODE_HELP.to_owned()),
            "decode" => Ok(TEXT_DECODE_HELP.to_owned()),
            _ => Err(CliError::usage(format!(
                "unknown help topic `text {command}`\n\n{TEXT_HELP}"
            ))),
        },
        _ => Err(CliError::usage(format!(
            "unknown help topic `{}`\n\n{HELP}",
            args.join(" ")
        ))),
    }
}

fn encode(args: EncodeArgs) -> Result<String, CliError> {
    let parsed = ParsedArgs::from_shape_options(vec![args.id], args.shape, args.explain)?;
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

fn decode(args: DecodeArgs) -> Result<String, CliError> {
    let parsed = ParsedArgs::from_shape_options(args.words, args.shape, args.explain)?;
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

fn presets() -> Result<String, CliError> {
    let mut output = String::new();
    output.push_str(PRESETS_CAVEAT);
    output.push('\n');
    output.push_str("name\tshape\tpermutation\twords\trange\tcapacity\tslack\tacceptance_ratio\n");
    for preset in PRESETS {
        let report = Report::new(preset.shape, preset.permutation, preset.range)?;
        output.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            preset.name,
            format_shape(preset.shape),
            preset.permutation.name(),
            preset.shape.len(),
            preset.range,
            report.capacity,
            report.slack(),
            report.acceptance_ratio()
        ));
    }
    Ok(output)
}

fn plan(args: PlanArgs) -> Result<String, CliError> {
    let parsed = ParsedArgs::from_shape_options(Vec::new(), args.shape, false)?;
    if let Some(name) = parsed.preset.as_deref() {
        if parsed.range.is_some()
            || parsed.words.is_some()
            || parsed.dictionary.is_some()
            || parsed.shape.is_some()
        {
            return Err(CliError::usage(
                "do not combine --preset with --range, --words, --shape, or --dict",
            ));
        }
        let preset =
            find_preset(name).ok_or_else(|| CliError::usage(format!("unknown preset `{name}`")))?;
        let report = Report::new(preset.shape, preset.permutation, preset.range)?;
        return Ok(format_report_fields("plan", &report, None, None));
    }

    let Some(range) = parsed.range else {
        if let Some(shape) = parsed.shape.as_deref() {
            if parsed.words.is_some() || parsed.dictionary.is_some() {
                return Err(CliError::usage(PLAN_HELP));
            }
            let lists = parse_shape(shape)?;
            return format_shape_report(&lists);
        }
        return Err(CliError::usage(
            "plan requires --range <R> or --shape <lists>",
        ));
    };
    if range == 0 {
        return Err(CliError::usage("range must be greater than zero"));
    }

    let lists = resolve_custom_shape(&parsed, true, range)?;
    let report = Report::new(&lists, IDENTITY_PERMUTATION, range)?;
    Ok(format_report_fields("plan", &report, None, None))
}

fn bytes_command(args: NestedBytesArgs) -> Result<String, CliError> {
    match args.command {
        Some(BytesCommand::Encode(args)) => bytes_encode(args),
        Some(BytesCommand::Decode(args)) => bytes_decode(args),
        None => Err(CliError::usage(BYTES_HELP)),
    }
}

fn bytes_encode(args: BytesEncodeArgs) -> Result<String, CliError> {
    let mode = byte_mode_from_flags(args.text, args.hex, "bytes encode")?;
    let bytes = match mode {
        ByteMode::Text => join_positionals(&args.data).into_bytes(),
        ByteMode::Hex => {
            if args.data.len() != 1 {
                return Err(CliError::usage("bytes encode --hex requires one hex value"));
            }
            parse_hex(&args.data[0])?
        }
    };
    let phrase = word_bytes_codec()
        .encode_bytes(&bytes)
        .map_err(runtime_error)?;
    Ok(format!("{phrase}\n"))
}

fn bytes_decode(args: BytesDecodeArgs) -> Result<String, CliError> {
    let words = split_words(&args.words);
    if words.is_empty() {
        return Err(CliError::usage(BYTES_DECODE_HELP));
    }

    let mode = byte_mode_from_flags(args.text, args.hex, "bytes decode")?;
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

fn text_command(args: NestedTextArgs) -> Result<String, CliError> {
    match args.command {
        Some(TextCommand::Encode(args)) => text_encode(args),
        Some(TextCommand::Decode(args)) => text_decode(args),
        None => Err(CliError::usage(TEXT_HELP)),
    }
}

fn text_encode(args: TextEncodeArgs) -> Result<String, CliError> {
    let text = join_positionals(&args.text);
    let phrase = word_bytes_codec()
        .encode_text(&text)
        .map_err(runtime_error)?;
    Ok(format!("{phrase}\n"))
}

fn text_decode(args: TextDecodeArgs) -> Result<String, CliError> {
    let words = split_words(&args.words);
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
    let report = Report::new(&shape.lists, shape.permutation, shape.range)?;
    Ok(format_report_fields(
        "encode",
        &report,
        Some(id),
        Some(phrase),
    ))
}

fn explain_decode(shape: &Shape, id: u128, phrase: &str) -> Result<String, CliError> {
    let report = Report::new(&shape.lists, shape.permutation, shape.range)?;
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
    output.push_str(report.caveat());
    output.push('\n');
    output.push_str(&format!("mode: {mode}\n"));
    if let Some(preset) = report.preset_name {
        output.push_str(&format!("preset: {preset}\n"));
    } else {
        output.push_str("preset: custom\n");
    }
    output.push_str(&format!("shape: {}\n", format_shape(&report.lists)));
    output.push_str(&format!("permutation: {}\n", report.permutation.name()));
    output.push_str(&format!("words: {}\n", report.lists.len()));
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

fn format_shape_report(lists: &[NamedWordList]) -> Result<String, CliError> {
    let capacity = CapacityDisplay(shape_capacity(lists).map_err(runtime_error)?);
    let mut output = String::new();
    if lists.iter().any(|list| list.is_bip39_english()) {
        output.push_str(BIP39_POSITIONAL_CAVEAT);
    } else {
        output.push_str(NAMED_SHAPE_CAVEAT);
    }
    output.push('\n');
    output.push_str("mode: plan\n");
    output.push_str("preset: custom\n");
    output.push_str(&format!("shape: {}\n", format_shape(lists)));
    output.push_str(&format!("words: {}\n", lists.len()));
    for (position, list) in lists.iter().enumerate() {
        output.push_str(&format!("position_{position}_list: {}\n", list.name()));
        output.push_str(&format!("position_{position}_words: {}\n", list.len()));
    }
    output.push_str(&format!("capacity: {capacity}\n"));
    Ok(output)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Preset {
    name: &'static str,
    range: u128,
    shape: &'static [NamedWordList],
    permutation: PermutationKind,
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct Shape {
    range: u128,
    lists: Vec<NamedWordList>,
    permutation: PermutationKind,
    preset_name: Option<&'static str>,
}

impl Shape {
    fn resolve(parsed: &ParsedArgs) -> Result<Self, CliError> {
        if let Some(name) = parsed.preset.as_deref() {
            if parsed.range.is_some()
                || parsed.words.is_some()
                || parsed.dictionary.is_some()
                || parsed.shape.is_some()
            {
                return Err(CliError::usage(
                    "do not combine --preset with --range, --words, --shape, or --dict",
                ));
            }
            let preset = find_preset(name)
                .ok_or_else(|| CliError::usage(format!("unknown preset `{name}`")))?;
            return Ok(Self {
                range: preset.range,
                lists: preset.shape.to_vec(),
                permutation: preset.permutation,
                preset_name: Some(preset.name),
            });
        }

        let range = parsed.range.ok_or_else(|| {
            CliError::usage(
                "use --preset <name> or provide --range <R> (--shape <lists> | --words <N>)",
            )
        })?;
        if range == 0 {
            return Err(CliError::usage("range must be greater than zero"));
        }
        let lists = resolve_custom_shape(parsed, false, range)?;

        Ok(Self {
            range,
            lists,
            permutation: IDENTITY_PERMUTATION,
            preset_name: None,
        })
    }

    fn codec(
        &self,
    ) -> Result<
        MixedPositional<WordListSequence<'_>, AsciiSpace, AffinePermutation>,
        nwords::core::Error,
    > {
        let permutation = self.permutation.permutation(self.range)?;
        let sequence = WordListSequence::new(&self.lists);
        MixedPositional::with_formatter_and_permutation(
            sequence,
            AsciiSpace,
            permutation,
            self.lists.len(),
            self.range,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Report {
    lists: Vec<NamedWordList>,
    permutation: PermutationKind,
    range: u128,
    capacity: CapacityDisplay,
    preset_name: Option<&'static str>,
}

impl Report {
    fn new(
        lists: &[NamedWordList],
        permutation: PermutationKind,
        range: u128,
    ) -> Result<Self, CliError> {
        if lists.is_empty() {
            return Err(CliError::usage("words must be greater than zero"));
        }
        if range == 0 {
            return Err(CliError::usage("range must be greater than zero"));
        }
        let capacity = shape_capacity(lists).map_err(runtime_error)?;
        Ok(Self {
            lists: lists.to_vec(),
            permutation,
            range,
            capacity: CapacityDisplay(capacity),
            preset_name: find_preset_by_shape(lists, permutation, range).map(|preset| preset.name),
        })
    }

    fn caveat(&self) -> &'static str {
        if self.lists.iter().any(|list| list.is_bip39_english()) {
            BIP39_POSITIONAL_CAVEAT
        } else {
            NAMED_SHAPE_CAVEAT
        }
    }

    fn representable(&self) -> &'static str {
        match self.capacity.0 {
            CapacityClass::Exact(capacity) if self.range <= capacity => "yes",
            CapacityClass::Exact(_) => "no",
            CapacityClass::BeyondU128 { .. } => "yes",
        }
    }

    fn slack(&self) -> String {
        stats::slack(self.capacity.0, self.range)
            .map(|slack| slack.to_string())
            .unwrap_or_else(|| "n/a".to_owned())
    }

    fn acceptance_ratio(&self) -> String {
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
    shape: Option<String>,
    dictionary: Option<String>,
    explain: bool,
}

impl ParsedArgs {
    fn from_shape_options(
        positionals: Vec<String>,
        shape: ShapeOptions,
        explain: bool,
    ) -> Result<Self, CliError> {
        let range = shape.range.as_deref().map(parse_range_u128).transpose()?;
        let words = shape
            .words
            .as_deref()
            .map(|value| {
                let words_u128 = parse_decimal_u128(value, "words")?;
                usize::try_from(words_u128).map_err(|_| CliError::usage("words value is too large"))
            })
            .transpose()?;

        Ok(Self {
            positionals,
            preset: shape.preset,
            range,
            words,
            shape: shape.shape,
            dictionary: shape.dictionary,
            explain,
        })
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

fn byte_mode_from_flags(text: bool, hex: bool, command: &str) -> Result<ByteMode, CliError> {
    match (text, hex) {
        (true, false) => Ok(ByteMode::Text),
        (false, true) => Ok(ByteMode::Hex),
        (false, false) => Err(CliError::usage(format!(
            "{command} requires --text or --hex"
        ))),
        (true, true) => Err(CliError::usage("duplicate --text/--hex")),
    }
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

fn parse_range_u128(value: &str) -> Result<u128, CliError> {
    if value.contains('e') || value.contains('E') {
        parse_scientific_range_u128(value)
    } else {
        parse_decimal_u128(value, "range")
    }
}

fn parse_scientific_range_u128(value: &str) -> Result<u128, CliError> {
    let mut parts = value.split(['e', 'E']);
    let mantissa = parts.next().unwrap_or_default();
    let exponent = parts.next().ok_or_else(invalid_range_shorthand)?;
    if parts.next().is_some() {
        return Err(invalid_range_shorthand());
    }

    let (significand, fractional_digits) = parse_range_mantissa(mantissa)?;
    let exponent = parse_range_exponent(exponent)?;

    if exponent >= fractional_digits {
        checked_mul_pow10(significand, exponent - fractional_digits)
    } else {
        let divisor = checked_pow10(fractional_digits - exponent)?;
        if significand % divisor != 0 {
            return Err(CliError::usage(
                "invalid range; scientific shorthand must expand to an integer",
            ));
        }
        Ok(significand / divisor)
    }
}

fn parse_range_mantissa(value: &str) -> Result<(u128, usize), CliError> {
    if value.is_empty() {
        return Err(invalid_range_shorthand());
    }

    let mut significand = 0u128;
    let mut fractional_digits = 0usize;
    let mut saw_digit = false;
    let mut saw_decimal_point = false;

    for byte in value.bytes() {
        match byte {
            b'0'..=b'9' => {
                let digit = u128::from(byte - b'0');
                significand = significand
                    .checked_mul(10)
                    .and_then(|value| value.checked_add(digit))
                    .ok_or_else(|| CliError::usage("range value is too large"))?;
                if saw_decimal_point {
                    fractional_digits = fractional_digits
                        .checked_add(1)
                        .ok_or_else(|| CliError::usage("range value is too large"))?;
                }
                saw_digit = true;
            }
            b'.' if !saw_decimal_point => saw_decimal_point = true,
            _ => return Err(invalid_range_shorthand()),
        }
    }

    if saw_digit {
        Ok((significand, fractional_digits))
    } else {
        Err(invalid_range_shorthand())
    }
}

fn parse_range_exponent(value: &str) -> Result<usize, CliError> {
    let value = value.strip_prefix('+').unwrap_or(value);
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(invalid_range_shorthand());
    }

    let mut exponent = 0usize;
    for byte in value.bytes() {
        exponent = exponent
            .checked_mul(10)
            .and_then(|value| value.checked_add(usize::from(byte - b'0')))
            .ok_or_else(|| CliError::usage("range value is too large"))?;
    }
    Ok(exponent)
}

fn checked_mul_pow10(mut value: u128, exponent: usize) -> Result<u128, CliError> {
    for _ in 0..exponent {
        value = value
            .checked_mul(10)
            .ok_or_else(|| CliError::usage("range value is too large"))?;
    }
    Ok(value)
}

fn checked_pow10(exponent: usize) -> Result<u128, CliError> {
    checked_mul_pow10(1, exponent)
}

fn invalid_range_shorthand() -> CliError {
    CliError::usage("invalid range; expected decimal digits or scientific notation like 1e6")
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

fn resolve_custom_shape(
    parsed: &ParsedArgs,
    allow_planning_default: bool,
    range: u128,
) -> Result<Vec<NamedWordList>, CliError> {
    if parsed.shape.is_some() && parsed.dictionary.is_some() {
        return Err(CliError::usage("do not combine --shape with --dict"));
    }
    if let Some(shape) = parsed.shape.as_deref() {
        if parsed.words.is_some() {
            return Err(CliError::usage(
                "shape has an intrinsic word count; omit --words",
            ));
        }
        return parse_shape(shape);
    }
    if let Some(dictionary) = parsed.dictionary.as_deref() {
        return resolve_legacy_dictionary_shape(dictionary, parsed.words);
    }
    if let Some(words) = parsed.words {
        return repeat_list(NamedWordList::Bip39English, words);
    }
    if allow_planning_default {
        match stats::required_words(PlanTarget::Range(range), English.len(0)) {
            Ok(PlanSolution::RequiredWords { word_count, .. }) => {
                repeat_list(NamedWordList::Bip39English, word_count)
            }
            Ok(_) => Err(CliError::runtime("unexpected stats planner result")),
            Err(error) => Err(runtime_error(error)),
        }
    } else {
        Err(CliError::usage(
            "use --preset <name> or provide --range <R> (--shape <lists> | --words <N>)",
        ))
    }
}

fn resolve_legacy_dictionary_shape(
    dictionary: &str,
    words: Option<usize>,
) -> Result<Vec<NamedWordList>, CliError> {
    match dictionary {
        DEFAULT_DICTIONARY | DEFAULT_LIST_NAME | "bip39-english" => {
            let words = words
                .ok_or_else(|| CliError::usage("dictionary `bip39-en` requires --words <N>"))?;
            repeat_list(NamedWordList::Bip39English, words)
        }
        ADJECTIVE_ANIMAL_DICTIONARY => {
            if words.is_some() {
                return Err(CliError::usage(
                    "dictionary `adjective-animal` has an intrinsic word count; omit --words",
                ));
            }
            Ok(ADJECTIVE_ANIMAL_SHAPE.to_vec())
        }
        _ => Err(CliError::usage(format!(
            "unknown dictionary `{dictionary}`"
        ))),
    }
}

fn repeat_list(list: NamedWordList, words: usize) -> Result<Vec<NamedWordList>, CliError> {
    if words == 0 {
        return Err(CliError::usage("words must be greater than zero"));
    }
    Ok(vec![list; words])
}

fn parse_shape(shape: &str) -> Result<Vec<NamedWordList>, CliError> {
    let mut lists = Vec::new();
    for raw_name in shape.split(',') {
        let name = raw_name.trim();
        if name.is_empty() {
            return Err(CliError::usage("shape contains an empty word-list name"));
        }
        if name == "word" {
            return Err(CliError::usage(
                "unknown word list `word`; use `bip39-en` for the BIP-39 English positional wordlist",
            ));
        }
        let list = NamedWordList::parse(name)
            .ok_or_else(|| CliError::usage(format!("unknown word list `{name}`")))?;
        lists.push(list);
    }
    if lists.is_empty() {
        return Err(CliError::usage("shape must contain at least one word list"));
    }
    Ok(lists)
}

fn shape_capacity(lists: &[NamedWordList]) -> Result<CapacityClass, nwords::core::Error> {
    let sizes = lists.iter().map(|list| list.len()).collect::<Vec<_>>();
    stats::capacity_mixed(&sizes)
}

fn format_shape(lists: &[NamedWordList]) -> String {
    let mut output = String::new();
    for (index, list) in lists.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(list.name());
    }
    output
}

fn find_preset(name: &str) -> Option<&'static Preset> {
    PRESETS.iter().find(|preset| preset.name == name)
}

fn find_preset_by_shape(
    lists: &[NamedWordList],
    permutation: PermutationKind,
    range: u128,
) -> Option<&'static Preset> {
    PRESETS.iter().find(|preset| {
        preset.shape == lists && preset.permutation == permutation && preset.range == range
    })
}

const HELP: &str = "\
nwords: positional ID phrase converter

USAGE:
    nwords encode <id> (--preset <name> | --range <R> (--shape <lists> | --words <N>)) [--explain]
    nwords decode <words...> (--preset <name> | --range <R> (--shape <lists> | --words <N>)) [--explain]
    nwords presets
    nwords plan (--preset <name> | --shape <lists> | --range <R> [--shape <lists> | --words <N>])
    nwords bytes encode (--text <text> | --hex <hex>)
    nwords bytes decode (--text | --hex) <words...>
    nwords text encode <text>
    nwords text decode <words...>

Shapes are comma-separated ordered word-list names such as adjective,animal, descriptor,object, or color,descriptor,object.

COMMANDS:
    encode      Encode an integer ID into a positional word phrase.
    decode      Decode a positional word phrase back into an integer ID.
    presets     List built-in preset ranges and shapes.
    plan        Show capacity and range statistics for a preset or shape.
    bytes       Encode or decode arbitrary bytes with word-bytes-v1.
    text        Encode or decode UTF-8 text with word-bytes-v1.

EXAMPLES:
    nwords encode 42 --preset u32
        Encode ID 42 with the built-in u32 preset.

    nwords encode 1337 --range 1e6 --shape color,adjective,animal
        Encode ID 1337 into a custom ordered named-list shape.

    nwords encode 42 --preset descriptor-object
        Encode ID 42 with the descriptor-object preset.

    nwords plan --shape color,adjective,animal
        Show per-position list sizes and total shape capacity.

Run `nwords help <command>` for detailed help.
";

const ENCODE_HELP: &str = "\
nwords encode: encode an integer ID into a deterministic word phrase

USAGE:
    nwords encode <id> (--preset <name> | --range <R> (--shape <lists> | --words <N>)) [--explain]

DESCRIPTION:
    Encodes an integer ID from the accepted range into a fixed ordered phrase.
    The mapping is deterministic and bidirectional. Shape order is part of the
    decoding contract.

OPTIONS:
    --preset <name>     Use a built-in preset such as u32, dec6, aa, color-aa,
                        or descriptor-object.
    --range <R>         Accepted exclusive range [0, R). Accepts decimal digits
                        or exact scientific shorthand such as 1e6.
    --shape <lists>     Comma-separated named word lists, for example
                        adjective,animal or color,adjective,animal.
    --words <N>         Repeat the BIP-39 English positional list N times.
    --dict <name>       Legacy alias; adjective-animal is accepted.
    --explain           Print phrase plus capacity/range metadata.

EXAMPLES:
    nwords encode 42 --preset u32
        Encode ID 42 with the built-in u32 preset.

    nwords encode 1337 --range 1e6 --shape color,adjective,animal
        Encode ID 1337 with a custom color-adjective-animal shape.

    nwords encode 42 --preset descriptor-object
        Encode ID 42 with the descriptor-object preset.

    nwords encode 42 --range 100000 --shape adjective,animal --explain
        Encode and include capacity, slack, and acceptance ratio metadata.
";

const DECODE_HELP: &str = "\
nwords decode: decode a deterministic word phrase back into an integer ID

USAGE:
    nwords decode <words...> (--preset <name> | --range <R> (--shape <lists> | --words <N>)) [--explain]

DESCRIPTION:
    Decodes a phrase created with the same preset or custom shape back into its
    original integer ID. Unknown words are reported by position and are not
    echoed in errors.

OPTIONS:
    --preset <name>     Use the same built-in preset used for encoding.
    --range <R>         Accepted exclusive range [0, R). Must match encoding.
    --shape <lists>     Ordered named lists. Must match encoding.
    --words <N>         Repeat the BIP-39 English positional list N times.
    --dict <name>       Legacy alias; adjective-animal is accepted.
    --explain           Print ID plus capacity/range metadata.

EXAMPLES:
    nwords decode \"abandon abandon abandon\" --preset u32
        Decode a phrase with the built-in u32 preset.

    nwords decode \"amaranth abundant amphibian\" --range 1e6 --shape color,adjective,animal
        Decode a phrase with a custom ordered named-list shape.

    nwords decode \"abalone aardvark\" --preset descriptor-object
        Decode a phrase with the descriptor-object preset.
";

const PRESETS_HELP: &str = "\
nwords presets: list built-in positional ID presets

USAGE:
    nwords presets

DESCRIPTION:
    Prints a tab-separated table of built-in presets, including shape,
    permutation, word count, accepted range, capacity, slack, and acceptance
    ratio.

EXAMPLES:
    nwords presets
        List every built-in preset.

    nwords plan --preset color-aa
        Show detailed stats for one preset.
";

const PLAN_HELP: &str = "\
nwords plan: show capacity and range statistics

USAGE:
    nwords plan (--preset <name> | --shape <lists> | --range <R> [--shape <lists> | --words <N>])

DESCRIPTION:
    Reports phrase-shape capacity and, when a range is provided, whether the
    range is representable along with slack and acceptance ratio.

OPTIONS:
    --preset <name>     Report stats for a built-in preset.
    --shape <lists>     Report shape-only stats, or combine with --range for
                        range/slack/acceptance stats.
    --range <R>         Accepted exclusive range [0, R). Accepts decimal digits
                        or exact scientific shorthand such as 1e6.
    --words <N>         Repeat the BIP-39 English positional list N times.
    --dict <name>       Legacy alias; adjective-animal is accepted.

EXAMPLES:
    nwords plan --shape color,adjective,animal
        Show list sizes and total capacity for the ordered shape.

    nwords plan --shape descriptor,object
        Show list sizes and total capacity for the descriptor-object shape.

    nwords plan --range 1e6 --shape color,adjective,animal
        Show whether one million IDs fit in the custom shape.

    nwords plan --preset u32
        Show capacity, slack, and acceptance ratio for the u32 preset.
";

const BYTES_HELP: &str = "\
nwords bytes: encode or decode arbitrary bytes with word-bytes-v1

USAGE:
    nwords bytes encode (--text <text> | --hex <hex>)
    nwords bytes decode (--text | --hex) <words...>

BIP-39 wordlist used as a positional dictionary, not a BIP-39 mnemonic.

DESCRIPTION:
    Encodes bytes as word phrases using word-bytes-v1: a 32-bit big-endian byte
    length, payload bytes, and zero padding to an 11-bit word boundary.

EXAMPLES:
    nwords bytes encode --hex deadbeef
        Encode raw bytes from hex into a word phrase.

    nwords bytes decode --hex \"abandon abandon abuse run swim jealous\"
        Decode a byte phrase and print lowercase hex.

Run `nwords help bytes encode` or `nwords help bytes decode` for subcommand help.
";

const BYTES_ENCODE_HELP: &str = "\
nwords bytes encode: encode text or hex bytes with word-bytes-v1

USAGE:
    nwords bytes encode (--text <text> | --hex <hex>)

DESCRIPTION:
    Encodes either UTF-8 text bytes or raw hex bytes into a word phrase. This is
    byte encoding, not BIP-39 mnemonic generation.

EXAMPLES:
    nwords bytes encode --text \"hello\"
        Encode UTF-8 text bytes.

    nwords bytes encode --hex deadbeef
        Encode raw bytes from hexadecimal input.
";

const BYTES_DECODE_HELP: &str = "\
nwords bytes decode: decode a word-bytes-v1 phrase to text or hex

USAGE:
    nwords bytes decode (--text | --hex) <words...>

DESCRIPTION:
    Decodes a word-bytes-v1 phrase and prints either UTF-8 text or lowercase
    hexadecimal bytes.

EXAMPLES:
    nwords bytes decode --text \"abandon abandon access speak fine curtain rose\"
        Decode a phrase and validate it as UTF-8 text.

    nwords bytes decode --hex \"abandon abandon abuse run swim jealous\"
        Decode a phrase and print raw bytes as hex.
";

const TEXT_HELP: &str = "\
nwords text: encode or decode UTF-8 text with word-bytes-v1

USAGE:
    nwords text encode <text>
    nwords text decode <words...>

BIP-39 wordlist used as a positional dictionary, not a BIP-39 mnemonic.

DESCRIPTION:
    Convenience wrapper over `nwords bytes --text`. Text is encoded as
    byte-exact UTF-8 with no default Unicode normalization.

EXAMPLES:
    nwords text encode \"hello\"
        Encode text into a word phrase.

    nwords text decode \"abandon abandon access speak fine curtain rose\"
        Decode a word phrase back into UTF-8 text.

Run `nwords help text encode` or `nwords help text decode` for subcommand help.
";

const TEXT_ENCODE_HELP: &str = "\
nwords text encode: encode UTF-8 text with word-bytes-v1

USAGE:
    nwords text encode <text>

DESCRIPTION:
    Encodes text as byte-exact UTF-8 with no normalization.

EXAMPLES:
    nwords text encode \"hello, world\"
        Encode text into a word phrase.
";

const TEXT_DECODE_HELP: &str = "\
nwords text decode: decode a word-bytes-v1 phrase into UTF-8 text

USAGE:
    nwords text decode <words...>

DESCRIPTION:
    Decodes a word-bytes-v1 phrase and validates the payload as UTF-8 text.

EXAMPLES:
    nwords text decode \"abandon abandon access speak fine curtain rose\"
        Decode a phrase back into text.
";

#[cfg(test)]
mod tests {
    use super::{run, BIP39_POSITIONAL_CAVEAT};

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
        assert!(output.stdout.contains(BIP39_POSITIONAL_CAVEAT));
        assert!(output.stdout.contains("mode: encode\n"));
        assert!(output.stdout.contains("preset: u32\n"));
        assert!(output.stdout.contains("id: 42\n"));
        assert!(output.stdout.contains("phrase: "));
    }
}
