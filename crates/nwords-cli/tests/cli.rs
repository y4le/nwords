use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

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

fn temp_wordlist(name: &str, contents: &[u8]) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "nwords-cli-test-{}-{nanos}-{name}.txt",
        std::process::id()
    ));
    fs::write(&path, contents).expect("write temp wordlist");
    path
}

fn lowercase_word(mut index: usize) -> String {
    let mut bytes = [b'a'; 6];
    for byte in bytes.iter_mut().rev() {
        *byte = b'a' + (index % 26) as u8;
        index /= 26;
    }
    String::from_utf8(bytes.to_vec()).expect("lowercase ascii word")
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
fn duplicate_shape_flags_are_rejected() {
    for (args, flag) in [
        (
            vec!["encode", "42", "--preset", "u32", "--preset", "dec6"],
            "--preset",
        ),
        (
            vec!["encode", "42", "--words", "3", "--words", "4"],
            "--words",
        ),
        (
            vec![
                "encode", "42", "--words", "3", "--range", "100", "--range", "200",
            ],
            "--range",
        ),
        (
            vec!["plan", "--shape", "animal", "--shape", "adjective"],
            "--shape",
        ),
        (
            vec!["plan", "--dict", "bip39-en", "--dict", "adjective-animal"],
            "--dict",
        ),
    ] {
        let output = nwords(&args);
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert!(
            stderr(&output).contains(&format!("duplicate {flag}")),
            "{args:?}"
        );
    }
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

    let lists = nwords(&["help", "lists"]);
    assert!(lists.status.success());
    let text = stdout(&lists);
    assert!(text.contains("nwords lists:"));
    assert!(text.contains("canonical name, aliases, advisory role, word count"));

    let bytes_encode = nwords(&["help", "bytes", "encode"]);
    assert!(bytes_encode.status.success());
    let text = stdout(&bytes_encode);
    assert!(text.contains("nwords bytes encode:"));
    assert!(text.contains("nwords bytes encode --hex deadbeef"));

    let bytes_group = nwords(&["bytes", "help"]);
    assert!(bytes_group.status.success());
    assert!(stdout(&bytes_group).contains("nwords bytes:"));

    let text_group = nwords(&["text", "help"]);
    assert!(text_group.status.success());
    assert!(stdout(&text_group).contains("nwords text:"));

    let bytes_topic = nwords(&["bytes", "help", "encode"]);
    assert!(bytes_topic.status.success());
    assert!(stdout(&bytes_topic).contains("nwords bytes encode:"));

    let text_topic = nwords(&["text", "help", "decode"]);
    assert!(text_topic.status.success());
    assert!(stdout(&text_topic).contains("nwords text decode:"));

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
        text.contains("BIP-39 wordlist used as a positional dictionary, not a BIP-39 mnemonic.")
    );
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
    assert!(text.contains(
        "mood-descriptor-object\tmood,descriptor,object\tidentity\t3\t280594368\t280594368\t0\t"
    ));
    assert!(text.contains(
        "material-shape-object\tmaterial,shape,object\tidentity\t3\t7810560\t7810560\t0\t"
    ));
    assert!(text.contains("mood-aa\tmood,adjective,animal\tidentity\t3\t15962688\t15962688\t0\t"));
    assert!(text.contains("descriptor-plant\tdescriptor,plant\tidentity\t2\t183936\t183936\t0\t"));
    assert!(text.contains(
        "weather-descriptor-plant\tweather,descriptor,plant\tidentity\t3\t7357440\t7357440\t0\t"
    ));
    assert!(text.contains(
        "mood-descriptor-food\tmood,descriptor,food\tidentity\t3\t11771904\t11771904\t0\t"
    ));
    assert!(
        text.contains("material-shape-food\tmaterial,shape,food\tidentity\t3\t327680\t327680\t0\t")
    );
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

    let full_dictionary = nwords(&["plan", "--dict", "adjective-animal"]);
    assert!(full_dictionary.status.success());
    assert!(stdout(&full_dictionary).contains("capacity: 249417\n"));

    let full_words = nwords(&["plan", "--words", "2"]);
    assert!(full_words.status.success());
    assert!(stdout(&full_words).contains("capacity: 4194304\n"));

    let missing_words = nwords(&["plan", "--dict", "bip39-english"]);
    assert_eq!(missing_words.status.code(), Some(2));
    assert!(stderr(&missing_words).contains("dictionary `bip39-english` requires --words"));

    let inferred_words = nwords(&[
        "plan",
        "--range",
        "1000000",
        "--dict",
        "bip39-en-positional",
    ]);
    assert!(inferred_words.status.success());
    assert!(stdout(&inferred_words).contains("words: 2\n"));
    assert!(stdout(&inferred_words).contains("range: 1000000\n"));
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

    let material_shape_object = nwords(&["plan", "--shape", "material,shape,object"]);
    assert!(material_shape_object.status.success());
    let text = stdout(&material_shape_object);
    assert!(text.contains("shape: material,shape,object\n"));
    assert!(text.contains("position_0_list: material\n"));
    assert!(text.contains("position_0_words: 64\n"));
    assert!(text.contains("position_1_list: shape\n"));
    assert!(text.contains("position_1_words: 40\n"));
    assert!(text.contains("position_2_list: object\n"));
    assert!(text.contains("position_2_words: 3051\n"));
    assert!(text.contains("capacity: 7810560\n"));

    let weather_descriptor_plant = nwords(&["plan", "--shape", "weather,descriptor,plant"]);
    assert!(weather_descriptor_plant.status.success());
    let text = stdout(&weather_descriptor_plant);
    assert!(text.contains("shape: weather,descriptor,plant\n"));
    assert!(text.contains("position_0_list: weather\n"));
    assert!(text.contains("position_0_words: 40\n"));
    assert!(text.contains("position_1_list: descriptor\n"));
    assert!(text.contains("position_1_words: 1437\n"));
    assert!(text.contains("position_2_list: plant\n"));
    assert!(text.contains("position_2_words: 128\n"));
    assert!(text.contains("capacity: 7357440\n"));
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

    let full_capacity_max = nwords(&["encode", "4194303", "--words", "2"]);
    assert!(full_capacity_max.status.success());
    assert_eq!(stdout(&full_capacity_max), "zoo zoo\n");

    let full_capacity_decoded = nwords(&["decode", "zoo zoo", "--words", "2"]);
    assert!(full_capacity_decoded.status.success());
    assert_eq!(stdout(&full_capacity_decoded), "4194303\n");

    let full_capacity_out_of_range = nwords(&["encode", "4194304", "--words", "2"]);
    assert_eq!(full_capacity_out_of_range.status.code(), Some(1));
    assert!(stderr(&full_capacity_out_of_range).contains("outside range"));

    let explain = nwords(&["encode", "4194303", "--words", "2", "--explain"]);
    assert!(explain.status.success());
    let text = stdout(&explain);
    assert!(text.contains("range: 4194304\n"));
    assert!(text.contains("capacity: 4194304\n"));
    assert!(text.contains("slack: 0\n"));
    assert!(text.contains("acceptance_ratio: 4194304/4194304\n"));

    let words = nwords::wordlists::bip39::English;
    let slack_word = words.word(100, 1).expect("word 100");
    let slack = nwords(&[
        "decode", "abandon", slack_word, "--range", "100", "--words", "2",
    ]);
    assert_eq!(slack.status.code(), Some(1));
    assert!(stderr(&slack).contains("outside range"));

    let beyond_u128 = nwords(&["encode", "0", "--words", "12"]);
    assert_eq!(beyond_u128.status.code(), Some(2));
    assert!(stderr(&beyond_u128).contains("shape capacity exceeds u128"));
    assert!(stderr(&beyond_u128).contains("provide --range"));

    let phrase = ["abandon"; 12].join(" ");
    let beyond_u128_decode = nwords(&["decode", &phrase, "--words", "12"]);
    assert_eq!(beyond_u128_decode.status.code(), Some(2));
    assert!(stderr(&beyond_u128_decode).contains("shape capacity exceeds u128"));

    let explicit_range = nwords(&["encode", "0", "--words", "12", "--range", "1"]);
    assert!(explicit_range.status.success());
    assert_eq!(stdout(&explicit_range).trim(), phrase);
    let explicit_decode = nwords(&["decode", &phrase, "--words", "12", "--range", "1"]);
    assert!(explicit_decode.status.success());
    assert_eq!(stdout(&explicit_decode), "0\n");
}

#[test]
fn legacy_dictionary_defaults_to_full_capacity() {
    let encoded = nwords(&["encode", "249416", "--dict", "adjective-animal"]);
    assert!(encoded.status.success());
    assert_eq!(stdout(&encoded), "zippy zebra\n");
    let decoded = nwords(&["decode", "zippy zebra", "--dict", "adjective-animal"]);
    assert!(decoded.status.success());
    assert_eq!(stdout(&decoded), "249416\n");
    let invalid = nwords(&["encode", "249417", "--dict", "adjective-animal"]);
    assert_eq!(invalid.status.code(), Some(1));
    assert!(stderr(&invalid).contains("outside range"));
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
fn user_defined_shape_round_trips_and_reports_fingerprint() {
    let list = temp_wordlist("project", b"alpha\n# ignored\n\nbravo\ncharlie\n");
    let spec = format!("project={}", list.display());

    let full = nwords(&[
        "encode",
        "998",
        "--shape",
        "project,animal",
        "--list",
        &spec,
    ]);
    assert!(full.status.success());
    assert_eq!(stdout(&full), "charlie zebra\n");
    let full_decoded = nwords(&[
        "decode",
        "charlie zebra",
        "--shape",
        "project,animal",
        "--list",
        &spec,
    ]);
    assert!(full_decoded.status.success());
    assert_eq!(stdout(&full_decoded), "998\n");
    let invalid = nwords(&[
        "encode",
        "999",
        "--shape",
        "project,animal",
        "--list",
        &spec,
    ]);
    assert_eq!(invalid.status.code(), Some(1));
    assert!(stderr(&invalid).contains("outside range"));

    let encoded = nwords(&[
        "encode",
        "5",
        "--range",
        "12",
        "--shape",
        "project,animal",
        "--list",
        &spec,
        "--explain",
    ]);
    assert!(encoded.status.success());
    let text = stdout(&encoded);
    assert!(text.contains("preset: custom\n"));
    assert!(text.contains("shape: project,animal\n"));
    assert!(text.contains("capacity: 999\n"));
    assert!(text.contains("phrase: alpha amphibian\n"));
    assert!(text.contains("user_list_0_name: project\n"));
    assert!(text.contains("user_list_0_words: 3\n"));
    assert!(text.contains("user_list_0_fingerprint: fnv1a64:6fc51280479725c3\n"));

    let decoded = nwords(&[
        "decode",
        "alpha amphibian",
        "--range",
        "12",
        "--shape",
        "project,animal",
        "--list",
        &spec,
    ]);
    assert!(decoded.status.success());
    assert_eq!(stdout(&decoded), "5\n");

    let plan = nwords(&["plan", "--shape", "project,animal", "--list", &spec]);
    assert!(plan.status.success());
    let text = stdout(&plan);
    assert!(text.contains("position_0_list: project\n"));
    assert!(text.contains("position_0_words: 3\n"));
    assert!(text.contains("position_0_fingerprint: fnv1a64:6fc51280479725c3\n"));
    assert!(text.contains("user_list_0_fingerprint: fnv1a64:6fc51280479725c3\n"));
}

#[test]
fn user_defined_list_validation_errors_are_actionable() {
    let duplicate = temp_wordlist("duplicate", b"alpha\nbravo\nalpha\n");
    let duplicate_spec = format!("project={}", duplicate.display());
    let output = nwords(&["plan", "--shape", "project", "--list", &duplicate_spec]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("duplicate word in list `project` at line 3"));
    assert!(stderr(&output).contains("first seen at line 1"));

    let invalid = temp_wordlist("invalid", b"alpha\nBravo\n");
    let invalid_spec = format!("project={}", invalid.display());
    let output = nwords(&["plan", "--shape", "project", "--list", &invalid_spec]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("invalid word in list `project` at line 2"));

    let too_few = temp_wordlist("too-few", b"# ignored\n\nalpha\n");
    let too_few_spec = format!("project={}", too_few.display());
    let output = nwords(&["plan", "--shape", "project", "--list", &too_few_spec]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("must contain at least two words"));

    let collision = temp_wordlist("collision", b"alpha\nbravo\n");
    let collision_spec = format!("animal={}", collision.display());
    let output = nwords(&["plan", "--shape", "animal", "--list", &collision_spec]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("collides with a built-in word list"));

    let invalid_utf8 = temp_wordlist("invalid-utf8", &[0xff, b'\n']);
    let invalid_utf8_spec = format!("project={}", invalid_utf8.display());
    let output = nwords(&["plan", "--shape", "project", "--list", &invalid_utf8_spec]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("not valid UTF-8"));

    let output = nwords(&["plan", "--range", "10", "--list", &invalid_spec]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("--list requires --shape"));

    let valid = temp_wordlist("valid", b"alpha\nbravo\n");
    let valid_spec = format!("project={}", valid.display());
    let output = nwords(&[
        "plan",
        "--shape",
        "project",
        "--list",
        &valid_spec,
        "--list",
        &valid_spec,
    ]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("duplicate user list `project`"));

    let output = nwords(&["plan", "--preset", "aa", "--list", &valid_spec]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("do not combine --preset"));

    let output = nwords(&[
        "plan",
        "--range",
        "10",
        "--dict",
        "adjective-animal",
        "--list",
        &valid_spec,
    ]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("cannot be combined with --dict or --words"));

    let output = nwords(&[
        "plan",
        "--range",
        "10",
        "--words",
        "2",
        "--list",
        &valid_spec,
    ]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("cannot be combined with --dict or --words"));

    let unused = temp_wordlist("unused", b"alpha\nbravo\n");
    let unused_spec = format!("project={}", unused.display());
    let output = nwords(&["plan", "--shape", "animal", "--list", &unused_spec]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("user list `project` is not referenced by --shape"));

    let invalid_name = temp_wordlist("invalid-name", b"alpha\nbravo\n");
    let invalid_name_spec = format!("Project={}", invalid_name.display());
    let output = nwords(&["plan", "--shape", "Project", "--list", &invalid_name_spec]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("invalid user list name `Project`"));

    let long_line = temp_wordlist(
        "long-line",
        format!("{}\nbravo\n", "a".repeat(129)).as_bytes(),
    );
    let long_line_spec = format!("project={}", long_line.display());
    let output = nwords(&["plan", "--shape", "project", "--list", &long_line_spec]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("line 1 exceeds"));

    let long_blank = temp_wordlist(
        "long-blank",
        format!("{}\nalpha\nbravo\n", " ".repeat(129)).as_bytes(),
    );
    let long_blank_spec = format!("project={}", long_blank.display());
    let output = nwords(&["plan", "--shape", "project", "--list", &long_blank_spec]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("line 1 exceeds"));

    let unicode_space = temp_wordlist("unicode-space", "alpha\n\u{2003}bravo\n".as_bytes());
    let unicode_space_spec = format!("project={}", unicode_space.display());
    let output = nwords(&["plan", "--shape", "project", "--list", &unicode_space_spec]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("invalid word in list `project` at line 2"));

    let oversized = temp_wordlist("oversized", &vec![b'a'; 1_048_577]);
    let oversized_spec = format!("project={}", oversized.display());
    let output = nwords(&["plan", "--shape", "project", "--list", &oversized_spec]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("exceeds 1048576 bytes"));

    let mut too_many_words = String::new();
    for index in 0..4097 {
        too_many_words.push_str(&lowercase_word(index));
        too_many_words.push('\n');
    }
    let too_many_words = temp_wordlist("too-many-words", too_many_words.as_bytes());
    let too_many_words_spec = format!("project={}", too_many_words.display());
    let output = nwords(&["plan", "--shape", "project", "--list", &too_many_words_spec]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("exceeds 4096 accepted words"));

    let mut too_many_list_args = vec!["plan", "--shape", "list0"];
    let list_specs = (0..33)
        .map(|index| format!("list{index}={}", valid.display()))
        .collect::<Vec<_>>();
    for spec in &list_specs {
        too_many_list_args.push("--list");
        too_many_list_args.push(spec);
    }
    let output = nwords(&too_many_list_args);
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("at most 32 user lists"));
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

    let zero = nwords(&[
        "plan",
        "--range",
        "0e999999999999999999",
        "--shape",
        "animal",
    ]);
    assert_eq!(zero.status.code(), Some(2));
    assert!(stderr(&zero).contains("range must be greater than zero"));
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

    let full_capacity = nwords(&["encode", "12969683", "--shape", "color,adjective,animal"]);
    assert!(full_capacity.status.success());
    assert_eq!(stdout(&full_capacity), "yellow zippy zebra\n");

    let full_capacity_decoded = nwords(&[
        "decode",
        "yellow zippy zebra",
        "--shape",
        "color,adjective,animal",
    ]);
    assert!(full_capacity_decoded.status.success());
    assert_eq!(stdout(&full_capacity_decoded), "12969683\n");

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
fn lists_reports_shape_options_with_examples() {
    let output = nwords(&["lists"]);

    assert!(output.status.success());
    assert_eq!(stderr(&output), "");
    let text = stdout(&output);
    assert!(
        text.contains("Built-in word-list names and order are decoding compatibility surfaces.")
    );
    assert!(text.contains("name\taliases\trole\twords\texamples\n"));
    assert_eq!(text.lines().count(), 14);
    assert!(text.contains("adjective\tadjectives\tmodifier\t749\table,"));
    assert!(text.contains(",zippy\n"));
    assert!(text.contains("animal\tanimals\thead\t333\taardvark,"));
    assert!(text.contains(",zebra\n"));
    assert!(text.contains("color\tcolors\tmodifier\t52\tamaranth,"));
    assert!(text.contains("descriptor\tdescriptors\tmodifier\t1437\tabalone,"));
    assert!(text.contains("object\tobjects\thead\t3051\taardvark,"));
    assert!(text.contains("mood\tmoods\tmodifier\t64\talert,"));
    assert!(text.contains("material\tmaterials\teither\t64\tacrylic,"));
    assert!(text.contains("shape\tshapes\teither\t40\tangular,"));
    assert!(text.contains("weather\t-\tmodifier\t40\tbalmy,"));
    assert!(text.contains("plant\tplants\thead\t128\tabelia,"));
    assert!(text.contains("food\tfoods\thead\t128\talmond,"));
    assert!(text.contains("bip39-en\tbip39-english,bip39-en-positional\teither\t2048\tabandon,"));
    assert!(text.contains(",zoo\n"));

    for row in text.lines().skip(2) {
        let fields = row.split('\t').collect::<Vec<_>>();
        assert_eq!(fields.len(), 5);
        for name in
            std::iter::once(fields[0]).chain(fields[1].split(',').filter(|alias| *alias != "-"))
        {
            let plan = nwords(&["plan", "--shape", name]);
            assert!(plan.status.success(), "catalog name {name} must resolve");
            let plan = stdout(&plan);
            assert!(plan.contains(&format!("position_0_list: {}\n", fields[0])));
            assert!(plan.contains(&format!("position_0_words: {}\n", fields[3])));
        }
    }
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
fn authored_semantic_presets_round_trip() {
    let mood = nwords(&["encode", "0", "--preset", "mood-descriptor-object"]);
    assert!(mood.status.success());
    assert_eq!(stdout(&mood), "alert abalone aardvark\n");

    let mood_max = nwords(&["encode", "280594367", "--preset", "mood-descriptor-object"]);
    assert!(mood_max.status.success());
    assert_eq!(stdout(&mood_max), "zestful zircon zydeco\n");

    let mood_decoded = nwords(&[
        "decode",
        "zestful zircon zydeco",
        "--preset",
        "mood-descriptor-object",
    ]);
    assert!(mood_decoded.status.success());
    assert_eq!(stdout(&mood_decoded), "280594367\n");

    let material = nwords(&["encode", "0", "--preset", "material-shape-object"]);
    assert!(material.status.success());
    assert_eq!(stdout(&material), "acrylic angular aardvark\n");

    let material_max = nwords(&["encode", "7810559", "--preset", "material-shape-object"]);
    assert!(material_max.status.success());
    assert_eq!(stdout(&material_max), "zinc zigzag zydeco\n");

    let mood_aa = nwords(&["encode", "0", "--preset", "mood-aa"]);
    assert!(mood_aa.status.success());
    assert_eq!(stdout(&mood_aa), "alert able aardvark\n");

    let mood_aa_max = nwords(&["encode", "15962687", "--preset", "mood-aa"]);
    assert!(mood_aa_max.status.success());
    assert_eq!(stdout(&mood_aa_max), "zestful zippy zebra\n");
}

#[test]
fn plant_food_presets_round_trip() {
    let plant = nwords(&["encode", "0", "--preset", "descriptor-plant"]);
    assert!(plant.status.success());
    assert_eq!(stdout(&plant), "abalone abelia\n");

    let plant_max = nwords(&["encode", "183935", "--preset", "descriptor-plant"]);
    assert!(plant_max.status.success());
    assert_eq!(stdout(&plant_max), "zircon zinnia\n");

    let weather_plant = nwords(&["encode", "0", "--preset", "weather-descriptor-plant"]);
    assert!(weather_plant.status.success());
    assert_eq!(stdout(&weather_plant), "balmy abalone abelia\n");

    let weather_plant_max = nwords(&["encode", "7357439", "--preset", "weather-descriptor-plant"]);
    assert!(weather_plant_max.status.success());
    assert_eq!(stdout(&weather_plant_max), "wintry zircon zinnia\n");

    for (preset, phrase, id) in [
        ("descriptor-plant", "abalone abelia", "0"),
        ("descriptor-plant", "zircon zinnia", "183935"),
        ("weather-descriptor-plant", "balmy abalone abelia", "0"),
        (
            "weather-descriptor-plant",
            "wintry zircon zinnia",
            "7357439",
        ),
    ] {
        let decoded = nwords(&["decode", phrase, "--preset", preset]);
        assert!(decoded.status.success());
        assert_eq!(stdout(&decoded).trim(), id);
    }
    let help = nwords(&["help", "decode"]);
    assert!(help.status.success());
    assert!(stdout(&help)
        .contains("nwords decode \"balmy abalone abelia\" --preset weather-descriptor-plant"));

    let food = nwords(&["encode", "0", "--preset", "mood-descriptor-food"]);
    assert!(food.status.success());
    assert_eq!(stdout(&food), "alert abalone almond\n");

    let food_max = nwords(&["encode", "11771903", "--preset", "mood-descriptor-food"]);
    assert!(food_max.status.success());
    assert_eq!(stdout(&food_max), "zestful zircon zucchini\n");

    let visual_food = nwords(&["encode", "0", "--preset", "material-shape-food"]);
    assert!(visual_food.status.success());
    assert_eq!(stdout(&visual_food), "acrylic angular almond\n");

    let visual_food_max = nwords(&["encode", "327679", "--preset", "material-shape-food"]);
    assert!(visual_food_max.status.success());
    assert_eq!(stdout(&visual_food_max), "zinc zigzag zucchini\n");
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
    let missing_text = nwords(&["text", "encode"]);
    assert_eq!(missing_text.status.code(), Some(2));
    assert!(stderr(&missing_text).contains("nwords text encode:"));

    let missing_bytes = nwords(&["bytes", "encode", "--text"]);
    assert_eq!(missing_bytes.status.code(), Some(2));
    assert!(stderr(&missing_bytes).contains("nwords bytes encode:"));

    let invalid_hex = nwords(&["bytes", "encode", "--hex", "abc"]);
    assert_eq!(invalid_hex.status.code(), Some(2));
    assert!(stderr(&invalid_hex).contains("even length"));

    let invalid_word = nwords(&["bytes", "decode", "--hex", "abandon zzzzz abandon"]);
    assert_eq!(invalid_word.status.code(), Some(1));
    assert!(stderr(&invalid_word).contains("unknown word at position 1"));
    assert!(!stderr(&invalid_word).contains("zzzzz"));
}
