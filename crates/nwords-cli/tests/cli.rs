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
fn presets_lists_stable_rows_and_caveat() {
    let output = nwords(&["presets"]);

    assert!(output.status.success());
    let text = stdout(&output);
    assert!(text.contains("BIP-39 wordlist used as a positional dictionary"));
    assert!(text.contains(
        "name\tdictionary\tpermutation\twords\trange\tcapacity\tslack\tacceptance_ratio"
    ));
    assert!(text.contains("dec6\tbip39-en-positional\tidentity\t2\t1000000\t4194304\t3194304"));
    assert!(text.contains(
        "dec6-spread\tbip39-en-positional\tspread-affine-v1\t2\t1000000\t4194304\t3194304"
    ));
    assert!(
        text.contains("u32\tbip39-en-positional\tidentity\t3\t4294967296\t8589934592\t4294967296")
    );
    assert!(text.contains(
        "u64\tbip39-en-positional\tidentity\t6\t18446744073709551616\t73786976294838206464"
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
fn explain_output_is_key_value_only() {
    let output = nwords(&["encode", "42", "--preset", "u32", "--explain"]);

    assert!(output.status.success());
    let text = stdout(&output);
    assert!(text.contains("mode: encode\n"));
    assert!(text.contains("preset: u32\n"));
    assert!(text.contains("dictionary: bip39-en-positional\n"));
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
