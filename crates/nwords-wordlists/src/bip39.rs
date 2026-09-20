//! BIP-39 wordlists.
//!
//! Generated word arrays come from Trezor `python-mnemonic` commit `b57a5ad77a981e743f4167ab2f7927a55c1e82a8`.
//! The source project is MIT licensed; see
//! `tests/vectors/licenses/python-mnemonic-MIT.LICENSE`.

#[path = "bip39_english.rs"]
mod bip39_english;
#[cfg(feature = "bip39-japanese")]
#[path = "bip39_japanese.rs"]
mod bip39_japanese;

use nwords_core::WordMap;

/// English BIP-39 word map.
#[derive(Debug, Clone, Copy, Default)]
pub struct English;

impl WordMap for English {
    fn len(&self, _position: usize) -> usize {
        bip39_english::ENGLISH_WORDS.len()
    }

    fn word(&self, index: usize, _position: usize) -> Option<&str> {
        bip39_english::ENGLISH_WORDS.get(index).copied()
    }

    fn index_of(&self, word: &str, _position: usize) -> Option<usize> {
        bip39_english::ENGLISH_WORDS.binary_search(&word).ok()
    }
}

/// Japanese BIP-39 word map.
#[cfg(feature = "bip39-japanese")]
#[derive(Debug, Clone, Copy, Default)]
pub struct Japanese;

#[cfg(feature = "bip39-japanese")]
impl WordMap for Japanese {
    fn len(&self, _position: usize) -> usize {
        bip39_japanese::JAPANESE_WORDS.len()
    }

    fn word(&self, index: usize, _position: usize) -> Option<&str> {
        bip39_japanese::JAPANESE_WORDS.get(index).copied()
    }

    fn index_of(&self, word: &str, _position: usize) -> Option<usize> {
        bip39_japanese::JAPANESE_WORDS
            .iter()
            .position(|candidate| *candidate == word)
    }
}

#[cfg(test)]
mod tests {
    use super::English;
    use nwords_core::WordMap;

    #[test]
    fn english_binary_search_preserves_every_frozen_index() {
        let words = English;
        let frozen = &super::bip39_english::ENGLISH_WORDS;
        assert!(frozen.windows(2).all(|pair| pair[0] < pair[1]));
        for (index, word) in frozen.iter().enumerate() {
            assert_eq!(words.index_of(word, 0), Some(index));
        }
        for missing in ["", "Abandon", "abandon!", "zzzzzz"] {
            assert_eq!(words.index_of(missing, 0), None);
        }
    }

    #[test]
    fn english_has_expected_boundary_words() {
        let words = English;

        assert_eq!(words.len(0), 2048);
        assert_eq!(words.word(0, 0), Some("abandon"));
        assert_eq!(words.word(2047, 0), Some("zoo"));
        assert_eq!(words.index_of("about", 0), Some(3));
    }

    #[cfg(feature = "bip39-japanese")]
    #[test]
    fn japanese_has_expected_boundary_words() {
        let words = super::Japanese;

        assert_eq!(words.len(0), 2048);
        assert_eq!(words.word(0, 0), Some("あいこくしん"));
        assert_eq!(words.index_of("あおぞら", 0), Some(3));
    }
}
