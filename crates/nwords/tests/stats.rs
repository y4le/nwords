#[cfg(all(feature = "stats", feature = "bip39"))]
#[test]
fn umbrella_reexports_bip39_stats_adapter() {
    assert_eq!(
        nwords::stats::bip39::for_word_count(24),
        Ok(nwords::stats::bip39::Bip39Report {
            dictionary_size: 2048,
            word_count: 24,
            entropy_bits: 256,
            checksum_bits: 8,
            total_bits: 264,
        })
    );
}
