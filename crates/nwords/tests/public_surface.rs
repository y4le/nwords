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
