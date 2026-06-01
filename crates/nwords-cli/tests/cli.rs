use std::process::{Command, Output};

use nwords::core::WordMap;

fn nwords(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_nwords"))
        .args(args)
        .output()
        .expect("run nwords")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout utf8")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr utf8")
}

#[test]
fn encode_decode_round_trip_u32() {
    let encoded = nwords(&["encode", "42", "--preset", "u32"]);
    assert!(encoded.status.success());
    assert_eq!(stderr(&encoded), "");

    let phrase = stdout(&encoded);
    assert_eq!(phrase.split_whitespace().count(), 3);

    let decoded = nwords(&["decode", phrase.trim(), "--preset", "u32"]);
    assert!(decoded.status.success());
    assert_eq!(stdout(&decoded), "42\n");
    assert_eq!(stderr(&decoded), "");
}

#[test]
fn boundary_ids_for_u32() {
    let zero = nwords(&["encode", "0", "--preset", "u32"]);
    assert!(zero.status.success());
    assert_eq!(zero.stdout, b"abandon abandon abandon\n");

    let max = nwords(&["encode", "4294967295", "--preset", "u32"]);
    assert!(max.status.success());
    assert_eq!(stdout(&max).split_whitespace().count(), 3);

    let out_of_range = nwords(&["encode", "4294967296", "--preset", "u32"]);
    assert_eq!(out_of_range.status.code(), Some(1));
    assert_eq!(stdout(&out_of_range), "");
    assert!(stderr(&out_of_range).contains("outside range"));
}

#[test]
fn decoding_first_u32_slack_state_fails() {
    let words = nwords::wordlists::bip39::English;
    let slack_phrase = format!(
        "{} {} {}",
        words.word(1024, 0).expect("word 1024"),
        words.word(0, 0).expect("word 0"),
        words.word(0, 0).expect("word 0")
    );

    let output = nwords(&["decode", &slack_phrase, "--preset", "u32"]);

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(stdout(&output), "");
    assert!(stderr(&output).contains("outside range"));
}

#[test]
fn decode_sanitizes_unknown_words() {
    let output = nwords(&["decode", "abandon zzzzz abandon", "--preset", "u32"]);

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(stdout(&output), "");
    assert!(stderr(&output).contains("unknown word at position 1"));
    assert!(!stderr(&output).contains("zzzzz"));
}

#[test]
fn usage_errors_exit_two() {
    let missing_command = nwords(&[]);
    assert_eq!(missing_command.status.code(), Some(2));
    assert_eq!(stdout(&missing_command), "");
    assert!(stderr(&missing_command).contains("USAGE"));

    let unknown_preset = nwords(&["encode", "1", "--preset", "missing"]);
    assert_eq!(unknown_preset.status.code(), Some(2));
    assert_eq!(stdout(&unknown_preset), "");
    assert!(stderr(&unknown_preset).contains("unknown preset"));

    let conflicting = nwords(&["encode", "1", "--preset", "dec6", "--range", "1000000"]);
    assert_eq!(conflicting.status.code(), Some(2));
    assert!(stderr(&conflicting).contains("do not combine --preset"));
}

#[test]
fn help_supports_command_topics_and_examples() {
    let encode = nwords(&["help", "encode"]);
    assert!(encode.status.success());
    let text = stdout(&encode);
    assert!(text.contains("nwords encode:"));
    assert!(text.contains("EXAMPLES:"));
    assert!(text.contains("nwords encode 1337 --range 1e6 --shape color,adjective,animal"));
    assert_eq!(stderr(&encode), "");

    let bytes_encode = nwords(&["help", "bytes", "encode"]);
    assert!(bytes_encode.status.success());
    let text = stdout(&bytes_encode);
    assert!(text.contains("nwords bytes encode:"));
    assert!(text.contains("nwords bytes encode --hex deadbeef"));

    let unknown = nwords(&["help", "missing"]);
    assert_eq!(unknown.status.code(), Some(2));
    assert!(stderr(&unknown).contains("unknown help topic `missing`"));
}

#[test]
fn help_flags_use_custom_detailed_help() {
    let top = nwords(&["--help"]);
    assert!(top.status.success());
    assert!(stdout(&top).contains("Run `nwords help <command>`"));
    assert_eq!(stderr(&top), "");

    let encode = nwords(&["encode", "--help"]);
    assert!(encode.status.success());
    assert!(stdout(&encode).contains("nwords encode:"));
    assert_eq!(stderr(&encode), "");

    let bytes_encode = nwords(&["bytes", "encode", "--help"]);
    assert!(bytes_encode.status.success());
    assert!(stdout(&bytes_encode).contains("nwords bytes encode:"));
    assert_eq!(stderr(&bytes_encode), "");
}

#[test]
fn help_flags_after_separator_are_literal_text() {
    let encoded = nwords(&["text", "encode", "--", "--help"]);
    assert!(encoded.status.success());
    assert!(!stdout(&encoded).contains("nwords text encode:"));

    let decoded = nwords(&["text", "decode", stdout(&encoded).trim()]);
    assert!(decoded.status.success());
    assert_eq!(stdout(&decoded), "--help\n");

    let bytes = nwords(&["bytes", "encode", "--text", "--", "-h"]);
    assert!(bytes.status.success());
    let decoded_bytes = nwords(&["bytes", "decode", "--text", stdout(&bytes).trim()]);
    assert!(decoded_bytes.status.success());
    assert_eq!(stdout(&decoded_bytes), "-h\n");
}

#[test]
fn presets_lists_stable_rows_and_caveat() {
    let output = nwords(&["presets"]);

    assert!(output.status.success());
    let text = stdout(&output);
    assert!(text.contains("Presets encode deterministic positional ID phrases"));
    assert!(
        text.contains("name\tshape\tpermutation\twords\trange\tcapacity\tslack\tacceptance_ratio")
    );
    assert!(text.contains("dec6\tbip39-en,bip39-en\tidentity\t2\t1000000\t4194304\t3194304"));
    assert!(text.contains(
        "dec6-spread\tbip39-en,bip39-en\tspread-affine-v1\t2\t1000000\t4194304\t3194304"
    ));
    assert!(text.contains(
        "u32\tbip39-en,bip39-en,bip39-en\tidentity\t3\t4294967296\t8589934592\t4294967296"
    ));
    assert!(text.contains(
        "u64\tbip39-en,bip39-en,bip39-en,bip39-en,bip39-en,bip39-en\tidentity\t6\t18446744073709551616\t73786976294838206464"
    ));
    assert!(text.contains("aa\tadjective,animal\tidentity\t2\t249417\t249417\t0\t"));
    assert!(text.contains("dec5-aa\tadjective,animal\tidentity\t2\t100000\t249417\t149417\t"));
    assert!(text.contains("color-aa\tcolor,adjective,animal\tidentity\t3\t12969684\t12969684\t0\t"));
    assert!(
        text.contains("descriptor-object\tdescriptor,object\tidentity\t2\t4384287\t4384287\t0\t")
    );
    assert!(text.contains(
        "color-descriptor-object\tcolor,descriptor,object\tidentity\t3\t227982924\t227982924\t0\t"
    ));
}

#[test]
fn spread_presets_round_trip_and_report_permutation() {
    let identity = nwords(&["encode", "42", "--preset", "dec6"]);
    assert!(identity.status.success());

    let spread = nwords(&["encode", "42", "--preset", "dec6-spread"]);
    assert!(spread.status.success());
    assert_eq!(stdout(&spread).split_whitespace().count(), 2);
    assert_ne!(stdout(&spread), stdout(&identity));

    let decoded = nwords(&["decode", stdout(&spread).trim(), "--preset", "dec6-spread"]);
    assert!(decoded.status.success());
    assert_eq!(stdout(&decoded), "42\n");

    let explained = nwords(&["encode", "42", "--preset", "dec6-spread", "--explain"]);
    assert!(explained.status.success());
    let explained_text = stdout(&explained);
    assert!(explained_text.contains("preset: dec6-spread\n"));
    assert!(explained_text.contains("permutation: spread-affine-v1\n"));

    let plan = nwords(&["plan", "--preset", "dec6-spread"]);
    assert!(plan.status.success());
    assert!(stdout(&plan).contains("permutation: spread-affine-v1\n"));
}

#[test]
fn plan_reports_capacity_and_unrepresentable_shapes() {
    let planned = nwords(&["plan", "--range", "1000000"]);
    assert!(planned.status.success());
    let planned_text = stdout(&planned);
    assert_eq!(
        planned_text
            .matches("BIP-39 wordlist used as a positional dictionary")
            .count(),
        1
    );
    assert!(planned_text.contains("words: 2\n"));
    assert!(planned_text.contains("capacity: 4194304\n"));
    assert!(planned_text.contains("slack: 3194304\n"));

    let preset = nwords(&["plan", "--preset", "u32"]);
    assert!(preset.status.success());
    let preset_text = stdout(&preset);
    assert!(preset_text.contains("mode: plan\n"));
    assert!(preset_text.contains("preset: u32\n"));
    assert!(preset_text.contains("permutation: identity\n"));
    assert!(preset_text.contains("range: 4294967296\n"));
    assert!(preset_text.contains("acceptance_ratio: 4294967296/8589934592\n"));

    let unrepresentable = nwords(&["plan", "--range", "1000000", "--words", "1"]);
    assert!(unrepresentable.status.success());
    let text = stdout(&unrepresentable);
    assert!(text.contains("words: 1\n"));
    assert!(text.contains("representable: no\n"));
    assert!(text.contains("slack: n/a\n"));

    let adjective_animal = nwords(&["plan", "--range", "100000", "--dict", "adjective-animal"]);
    assert!(adjective_animal.status.success());
    let text = stdout(&adjective_animal);
    assert!(text.contains("Named word-list phrases are deterministic positional IDs"));
    assert!(text.contains("shape: adjective,animal\n"));
    assert!(text.contains("words: 2\n"));
    assert!(text.contains("capacity: 249417\n"));
    assert!(text.contains("slack: 149417\n"));
}

#[test]
fn plan_shape_reports_position_counts_without_range() {
    let output = nwords(&["plan", "--shape", "color,adjective,animal"]);

    assert!(output.status.success());
    let text = stdout(&output);
    assert!(text.contains("Named word-list phrases are deterministic positional IDs"));
    assert!(text.contains("mode: plan\n"));
    assert!(text.contains("preset: custom\n"));
    assert!(text.contains("shape: color,adjective,animal\n"));
    assert!(text.contains("words: 3\n"));
    assert!(text.contains("position_0_list: color\n"));
    assert!(text.contains("position_0_words: 52\n"));
    assert!(text.contains("position_1_list: adjective\n"));
    assert!(text.contains("position_1_words: 749\n"));
    assert!(text.contains("position_2_list: animal\n"));
    assert!(text.contains("position_2_words: 333\n"));
    assert!(text.contains("capacity: 12969684\n"));
    assert!(!text.contains("range:"));
    assert!(!text.contains("slack:"));

    let descriptor_object = nwords(&["plan", "--shape", "descriptor,object"]);
    assert!(descriptor_object.status.success());
    let text = stdout(&descriptor_object);
    assert!(text.contains("shape: descriptor,object\n"));
    assert!(text.contains("position_0_list: descriptor\n"));
    assert!(text.contains("position_0_words: 1437\n"));
    assert!(text.contains("position_1_list: object\n"));
    assert!(text.contains("position_1_words: 3051\n"));
    assert!(text.contains("capacity: 4384287\n"));
}

#[test]
fn custom_range_and_words_round_trip() {
    let encoded = nwords(&["encode", "123", "--range", "1000000", "--words", "2"]);
    assert!(encoded.status.success());
    assert_eq!(stdout(&encoded).split_whitespace().count(), 2);

    let decoded = nwords(&[
        "decode",
        stdout(&encoded).trim(),
        "--range",
        "1000000",
        "--words",
        "2",
    ]);
    assert!(decoded.status.success());
    assert_eq!(stdout(&decoded), "123\n");
}

#[test]
fn custom_named_shape_round_trips_and_rejects_word_alias() {
    let encoded = nwords(&[
        "encode",
        "42",
        "--range",
        "100000",
        "--shape",
        "color,adjective,animal",
    ]);
    assert!(encoded.status.success());
    assert_eq!(stdout(&encoded).split_whitespace().count(), 3);

    let decoded = nwords(&[
        "decode",
        stdout(&encoded).trim(),
        "--range",
        "100000",
        "--shape",
        "color,adjective,animal",
    ]);
    assert!(decoded.status.success());
    assert_eq!(stdout(&decoded), "42\n");

    let word_alias = nwords(&["plan", "--range", "100", "--shape", "color,word"]);
    assert_eq!(word_alias.status.code(), Some(2));
    assert!(stderr(&word_alias).contains("use `bip39-en`"));
}

#[test]
fn range_accepts_exact_scientific_shorthand() {
    let encoded = nwords(&[
        "encode",
        "1337",
        "--range",
        "1e6",
        "--shape",
        "color,adjective,animal",
    ]);
    assert!(encoded.status.success());
    assert_eq!(stdout(&encoded).split_whitespace().count(), 3);

    let decoded = nwords(&[
        "decode",
        stdout(&encoded).trim(),
        "--range",
        "1E6",
        "--shape",
        "color,adjective,animal",
    ]);
    assert!(decoded.status.success());
    assert_eq!(stdout(&decoded), "1337\n");

    let planned = nwords(&[
        "plan",
        "--range",
        "1.5e6",
        "--shape",
        "color,adjective,animal",
    ]);
    assert!(planned.status.success());
    assert!(stdout(&planned).contains("range: 1500000\n"));

    let fractional = nwords(&["plan", "--range", "1.5e0", "--shape", "adjective,animal"]);
    assert_eq!(fractional.status.code(), Some(2));
    assert!(stderr(&fractional).contains("must expand to an integer"));
}

#[test]
fn adjective_animal_presets_and_custom_dictionary_round_trip() {
    let zero = nwords(&["encode", "0", "--preset", "aa"]);
    assert!(zero.status.success());
    assert_eq!(stdout(&zero), "able aardvark\n");

    let max = nwords(&["encode", "249416", "--preset", "aa"]);
    assert!(max.status.success());
    assert_eq!(stdout(&max), "zippy zebra\n");

    let decoded = nwords(&["decode", "zippy zebra", "--preset", "aa"]);
    assert!(decoded.status.success());
    assert_eq!(stdout(&decoded), "249416\n");

    let out_of_range = nwords(&["encode", "249417", "--preset", "aa"]);
    assert_eq!(out_of_range.status.code(), Some(1));
    assert!(stderr(&out_of_range).contains("outside range"));

    let custom = nwords(&[
        "encode",
        "42",
        "--range",
        "100000",
        "--dict",
        "adjective-animal",
    ]);
    assert!(custom.status.success());
    assert_eq!(stdout(&custom).split_whitespace().count(), 2);

    let custom_decoded = nwords(&[
        "decode",
        stdout(&custom).trim(),
        "--range",
        "100000",
        "--dict",
        "adjective-animal",
    ]);
    assert!(custom_decoded.status.success());
    assert_eq!(stdout(&custom_decoded), "42\n");

    let conflicting_words = nwords(&[
        "encode",
        "42",
        "--range",
        "100000",
        "--dict",
        "adjective-animal",
        "--words",
        "2",
    ]);
    assert_eq!(conflicting_words.status.code(), Some(2));
    assert!(stderr(&conflicting_words).contains("intrinsic word count"));
}

#[test]
fn descriptor_object_presets_round_trip() {
    let zero = nwords(&["encode", "0", "--preset", "descriptor-object"]);
    assert!(zero.status.success());
    assert_eq!(stdout(&zero), "abalone aardvark\n");

    let max = nwords(&["encode", "4384286", "--preset", "descriptor-object"]);
    assert!(max.status.success());
    assert_eq!(stdout(&max), "zircon zydeco\n");

    let decoded = nwords(&["decode", "zircon zydeco", "--preset", "descriptor-object"]);
    assert!(decoded.status.success());
    assert_eq!(stdout(&decoded), "4384286\n");

    let color = nwords(&["encode", "0", "--preset", "color-descriptor-object"]);
    assert!(color.status.success());
    assert_eq!(stdout(&color), "amaranth abalone aardvark\n");

    let custom = nwords(&[
        "encode",
        "42",
        "--range",
        "1000000",
        "--shape",
        "descriptor,object",
    ]);
    assert!(custom.status.success());
    assert_eq!(stdout(&custom).split_whitespace().count(), 2);

    let custom_decoded = nwords(&[
        "decode",
        stdout(&custom).trim(),
        "--range",
        "1000000",
        "--shape",
        "descriptor,object",
    ]);
    assert!(custom_decoded.status.success());
    assert_eq!(stdout(&custom_decoded), "42\n");
}

#[test]
fn explain_output_is_key_value_only() {
    let output = nwords(&["encode", "42", "--preset", "u32", "--explain"]);

    assert!(output.status.success());
    let text = stdout(&output);
    assert!(text.contains("mode: encode\n"));
    assert!(text.contains("preset: u32\n"));
    assert!(text.contains("shape: bip39-en,bip39-en,bip39-en\n"));
    assert!(text.contains("range: 4294967296\n"));
    assert!(text.contains("capacity: 8589934592\n"));
    assert!(text.contains("id: 42\n"));
    assert!(text.contains("phrase: "));
}

#[test]
fn whitespace_is_tolerated_but_case_is_exact() {
    let decoded = nwords(&[
        "decode",
        "  abandon    abandon   abandon  ",
        "--preset",
        "u32",
    ]);
    assert!(decoded.status.success());
    assert_eq!(stdout(&decoded), "0\n");

    let uppercase = nwords(&["decode", "ABANDON abandon abandon", "--preset", "u32"]);
    assert_eq!(uppercase.status.code(), Some(1));
    assert!(stderr(&uppercase).contains("unknown word at position 0"));
}

#[test]
fn text_commands_round_trip_utf8_without_normalization() {
    let encoded = nwords(&["text", "encode", "hello, 世界"]);
    assert!(encoded.status.success());
    assert_eq!(stdout(&encoded).split_whitespace().count(), 13);

    let decoded = nwords(&["text", "decode", stdout(&encoded).trim()]);
    assert!(decoded.status.success());
    assert_eq!(stdout(&decoded), "hello, 世界\n");

    let precomposed = nwords(&["text", "encode", "é"]);
    let decomposed = nwords(&["text", "encode", "é"]);
    assert!(precomposed.status.success());
    assert!(decomposed.status.success());
    assert_ne!(stdout(&precomposed), stdout(&decomposed));
}

#[test]
fn bytes_commands_round_trip_hex_and_text() {
    let hex = nwords(&["bytes", "encode", "--hex", "deadbeef"]);
    assert!(hex.status.success());
    assert_eq!(stdout(&hex), "abandon abandon abuse run swim jealous\n");

    let decoded_hex = nwords(&["bytes", "decode", "--hex", stdout(&hex).trim()]);
    assert!(decoded_hex.status.success());
    assert_eq!(stdout(&decoded_hex), "deadbeef\n");

    let text = nwords(&["bytes", "encode", "--text", "hello"]);
    assert!(text.status.success());
    assert_eq!(
        stdout(&text),
        "abandon abandon access speak fine curtain rose\n"
    );

    let decoded_text = nwords(&["bytes", "decode", "--text", stdout(&text).trim()]);
    assert!(decoded_text.status.success());
    assert_eq!(stdout(&decoded_text), "hello\n");
}

#[test]
fn bytes_usage_and_decode_errors_are_sanitized() {
    let invalid_hex = nwords(&["bytes", "encode", "--hex", "abc"]);
    assert_eq!(invalid_hex.status.code(), Some(2));
    assert!(stderr(&invalid_hex).contains("even length"));

    let invalid_word = nwords(&["bytes", "decode", "--hex", "abandon zzzzz abandon"]);
    assert_eq!(invalid_word.status.code(), Some(1));
    assert!(stderr(&invalid_word).contains("unknown word at position 1"));
    assert!(!stderr(&invalid_word).contains("zzzzz"));
}
