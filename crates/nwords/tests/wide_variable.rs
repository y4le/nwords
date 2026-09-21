#![cfg(all(feature = "positional", feature = "adjective-animal"))]

use nwords::{
    variable::VariablePositional,
    wide_variable::{WideId, WideVariableError, WideVariablePositional},
    wordlists::named::{NamedWordList, WordListSequence},
};

fn names() -> WideVariablePositional<WordListSequence<'static>> {
    static SHAPE: &[NamedWordList] = &[NamedWordList::Adjective, NamedWordList::Animal];
    WideVariablePositional::new(WordListSequence::new(SHAPE), 1, 1, None, 32).unwrap()
}

#[test]
fn matches_published_wide_js_phrases() {
    let codec = names();
    let legacy = VariablePositional::new(
        WordListSequence::new(&[NamedWordList::Adjective, NamedWordList::Animal]),
        1,
        1,
        Some(u128::MAX),
        Some(32),
    )
    .unwrap();
    for row in include_str!("../../../tests/vectors/variable/variable-v1-wide.tsv")
        .lines()
        .filter(|row| !row.starts_with('#'))
    {
        let (decimal, phrase) = row.split_once('\t').unwrap();
        let id = WideId::parse_decimal(decimal).unwrap();
        assert_eq!(codec.encode(&id).unwrap(), phrase, "ID {decimal}");
        assert_eq!(codec.decode_phrase(phrase).unwrap(), id, "ID {decimal}");
        if let Ok(narrow) = decimal.parse::<u128>() {
            if narrow < u128::MAX {
                assert_eq!(legacy.encode(narrow).unwrap(), phrase, "ID {decimal}");
            }
        }
    }
}

#[test]
fn exact_views_and_bounds_roundtrip() {
    let codec = names();
    for bits in [
        "",
        "0",
        "01011",
        "00000000",
        &"1".repeat(127),
        &"0".repeat(128),
        &"1".repeat(129),
        &"1".repeat(255),
    ] {
        assert_eq!(
            codec
                .decode_bits(&codec.encode_bits(bits).unwrap())
                .unwrap(),
            bits
        );
    }
    for bytes in [&[][..], &[0, 0, 42], &[255, 0, 128]] {
        assert_eq!(
            codec
                .decode_bytes(&codec.encode_bytes(bytes).unwrap())
                .unwrap(),
            bytes
        );
    }
    for text in ["", "hello", "café 🦊", "\u{feff}A"] {
        assert_eq!(
            codec
                .decode_text(&codec.encode_text(text).unwrap())
                .unwrap(),
            text
        );
    }
    assert_eq!(
        codec.decode_bytes(&codec.encode(&WideId::from_u128(42)).unwrap()),
        Err(WideVariableError::NotByteAligned)
    );
    assert_eq!(
        codec.decode_text(&codec.encode_bits("11111111").unwrap()),
        Err(WideVariableError::InvalidUtf8)
    );
    assert_eq!(
        codec.encode_bits(&"0".repeat(4097)),
        Err(WideVariableError::InvalidBits)
    );
    assert_eq!(
        codec.encode_bytes(&[0; 513]),
        Err(WideVariableError::InvalidBytes)
    );
    let bounded = WideVariablePositional::new(
        WordListSequence::new(&[NamedWordList::Adjective, NamedWordList::Animal]),
        1,
        1,
        Some(WideId::from_u128(1000)),
        32,
    )
    .unwrap();
    assert_eq!(
        bounded.encode(&WideId::from_u128(1000)),
        Err(WideVariableError::OutOfRange)
    );
    assert_eq!(
        bounded.decode_phrase(&codec.encode(&WideId::from_u128(1000)).unwrap()),
        Err(WideVariableError::OutOfRange)
    );
    let two_words = WideVariablePositional::new(
        WordListSequence::new(&[NamedWordList::Adjective, NamedWordList::Animal]),
        1,
        1,
        None,
        2,
    )
    .unwrap();
    assert_eq!(
        two_words.decode_phrase("aardvark"),
        Err(WideVariableError::InvalidWordCount { got: 1 })
    );
    assert_eq!(
        two_words.decode_phrase("able able aardvark"),
        Err(WideVariableError::InvalidWordCount { got: 3 })
    );
}
