#[cfg(feature = "bip39")]
#[test]
fn top_level_bip39_reexport_works() {
    let codec = nwords::bip39::English::default();

    assert_eq!(
        codec.encode_entropy(&[0u8; 16]).as_deref(),
        Ok("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about")
    );
}

#[cfg(all(feature = "bip39", feature = "bip39-seed"))]
#[test]
fn nested_bip39_seed_reexport_works() {
    let seed = nwords::bip39::seed::derive_seed(
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        "TREZOR",
    );

    assert_eq!(seed.len(), nwords::bip39::seed::SEED_BYTES);
}

#[cfg(feature = "positional")]
#[test]
fn top_level_positional_reexport_works() {
    let codec = nwords::positional::Positional::new(
        nwords::core::Linear::new(&["zero", "one", "two"]),
        2,
        9,
    )
    .expect("codec");

    assert_eq!(codec.encode(8).as_deref(), Ok("two two"));
}

#[cfg(all(feature = "adjective-animal", feature = "positional"))]
#[test]
fn adjective_animal_wordlist_reexport_works() {
    let codec = nwords::positional::MixedPositional::new(
        nwords::wordlists::adjective_animal::AdjectiveAnimal,
        nwords::wordlists::adjective_animal::WORD_COUNT,
        nwords::wordlists::adjective_animal::CAPACITY,
    )
    .expect("codec");

    assert_eq!(codec.encode(0).as_deref(), Ok("able aardvark"));
}

#[cfg(all(feature = "named", feature = "positional", feature = "bip39"))]
#[test]
fn named_wordlist_sequence_reexport_works() {
    let lists = [
        nwords::wordlists::named::NamedWordList::Color,
        nwords::wordlists::named::NamedWordList::Adjective,
        nwords::wordlists::named::NamedWordList::Animal,
    ];
    let shape = nwords::wordlists::named::WordListSequence::new(&lists);
    let codec = nwords::positional::MixedPositional::new(shape, shape.word_count(), 12_969_684)
        .expect("codec");

    assert_eq!(codec.encode(0).as_deref(), Ok("amaranth able aardvark"));
    assert_eq!(
        codec.decode_words(&["yellow", "zippy", "zebra"]),
        Ok(12_969_683)
    );
}

#[cfg(all(feature = "named", feature = "positional", feature = "alloc"))]
#[test]
fn dynamic_wordlist_sequence_reexport_works() {
    let owned = nwords::wordlists::named::OwnedWordList::new(
        "project",
        nwords::wordlists::named::WordListRole::Either,
        vec!["alpha".to_owned(), "bravo".to_owned()],
    )
    .expect("owned list");
    let sequence = nwords::wordlists::named::DynamicWordListSequence::new(
        vec![owned],
        vec![
            nwords::wordlists::named::DynamicWordListSlot::Builtin(
                nwords::wordlists::named::NamedWordList::Color,
            ),
            nwords::wordlists::named::DynamicWordListSlot::Owned(0),
        ],
    )
    .expect("sequence");
    let codec = nwords::positional::MixedPositional::new(sequence, 2, 104).expect("codec");

    assert_eq!(codec.encode(0).as_deref(), Ok("amaranth alpha"));
    assert_eq!(codec.decode_words(&["amber", "bravo"]), Ok(3));
}

#[cfg(all(feature = "word-bytes", feature = "bip39"))]
#[test]
fn top_level_word_bytes_reexport_works() {
    let codec = nwords::word_bytes::WordBytes::new(nwords::wordlists::bip39::English);
    let phrase = codec.encode_text("hello").expect("encode");

    assert_eq!(phrase, "abandon abandon access speak fine curtain rose");
    assert_eq!(
        codec
            .decode_text(&phrase.split_whitespace().collect::<Vec<_>>())
            .as_deref(),
        Ok("hello")
    );
}
