//! Named word lists and ordered phrase-shape adapters.
//!
//! Names and word order are encoding contracts. Changing a list's contents or
//! order requires a new list name.

use nwords_core::WordMap;

use crate::{adjective_animal_adjectives, adjective_animal_animals, unique_names_generator_colors};

/// Built-in single-position word list names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NamedWordList {
    /// Curated adjective list from `unique-names-generator`.
    Adjective,
    /// Curated animal list from `unique-names-generator`.
    Animal,
    /// Color list from `unique-names-generator`.
    Color,
    /// English BIP-39 wordlist used as a positional dictionary.
    #[cfg(feature = "bip39-english")]
    Bip39English,
}

impl NamedWordList {
    /// Parses a built-in word list name.
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "adjective" | "adjectives" => Some(Self::Adjective),
            "animal" | "animals" => Some(Self::Animal),
            "color" | "colors" => Some(Self::Color),
            #[cfg(feature = "bip39-english")]
            "bip39-en" | "bip39-english" | "bip39-en-positional" => Some(Self::Bip39English),
            _ => None,
        }
    }

    /// Returns the canonical list name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Adjective => "adjective",
            Self::Animal => "animal",
            Self::Color => "color",
            #[cfg(feature = "bip39-english")]
            Self::Bip39English => "bip39-en",
        }
    }

    /// Returns the number of words in this list.
    pub fn len(self) -> usize {
        match self {
            Self::Adjective => adjective_animal_adjectives::ADJECTIVES.len(),
            Self::Animal => adjective_animal_animals::ANIMALS.len(),
            Self::Color => unique_names_generator_colors::COLORS.len(),
            #[cfg(feature = "bip39-english")]
            Self::Bip39English => crate::bip39::English.len(0),
        }
    }

    /// Returns true when this list has no words.
    pub fn is_empty(self) -> bool {
        self.len() == 0
    }

    /// Returns the word at `index`.
    pub fn word(self, index: usize) -> Option<&'static str> {
        match self {
            Self::Adjective => adjective_animal_adjectives::ADJECTIVES.get(index).copied(),
            Self::Animal => adjective_animal_animals::ANIMALS.get(index).copied(),
            Self::Color => unique_names_generator_colors::COLORS.get(index).copied(),
            #[cfg(feature = "bip39-english")]
            Self::Bip39English => crate::bip39::English.word(index, 0),
        }
    }

    /// Returns the index for `word`.
    pub fn index_of(self, word: &str) -> Option<usize> {
        match self {
            Self::Adjective => adjective_animal_adjectives::ADJECTIVES
                .iter()
                .position(|candidate| *candidate == word),
            Self::Animal => adjective_animal_animals::ANIMALS
                .iter()
                .position(|candidate| *candidate == word),
            Self::Color => unique_names_generator_colors::COLORS
                .iter()
                .position(|candidate| *candidate == word),
            #[cfg(feature = "bip39-english")]
            Self::Bip39English => crate::bip39::English.index_of(word, 0),
        }
    }

    /// Returns true when this list uses BIP-39 words as positional words.
    pub const fn is_bip39_english(self) -> bool {
        match self {
            #[cfg(feature = "bip39-english")]
            Self::Bip39English => true,
            _ => false,
        }
    }
}

/// Ordered sequence of named word lists.
#[derive(Debug, Clone, Copy)]
pub struct WordListSequence<'a> {
    lists: &'a [NamedWordList],
}

impl<'a> WordListSequence<'a> {
    /// Creates a phrase shape from ordered word lists.
    pub const fn new(lists: &'a [NamedWordList]) -> Self {
        Self { lists }
    }

    /// Returns the ordered word lists.
    pub const fn lists(&self) -> &'a [NamedWordList] {
        self.lists
    }

    /// Returns the phrase word count.
    pub const fn word_count(&self) -> usize {
        self.lists.len()
    }

    /// Returns exact capacity if it fits in `u128`.
    pub fn capacity(&self) -> Option<u128> {
        let mut capacity = 1u128;
        for list in self.lists {
            capacity = capacity.checked_mul(list.len() as u128)?;
        }
        Some(capacity)
    }
}

impl WordMap for WordListSequence<'_> {
    fn len(&self, position: usize) -> usize {
        self.lists.get(position).map_or(0, |list| list.len())
    }

    fn word(&self, index: usize, position: usize) -> Option<&str> {
        self.lists.get(position).and_then(|list| list.word(index))
    }

    fn index_of(&self, word: &str, position: usize) -> Option<usize> {
        self.lists
            .get(position)
            .and_then(|list| list.index_of(word))
    }
}

#[cfg(test)]
mod tests {
    use super::{NamedWordList, WordListSequence};
    use nwords_core::WordMap;

    #[test]
    fn named_lists_have_stable_boundaries() {
        assert_eq!(NamedWordList::Adjective.len(), 749);
        assert_eq!(NamedWordList::Adjective.word(0), Some("able"));
        assert_eq!(NamedWordList::Adjective.word(748), Some("zippy"));

        assert_eq!(NamedWordList::Animal.len(), 333);
        assert_eq!(NamedWordList::Animal.word(0), Some("aardvark"));
        assert_eq!(NamedWordList::Animal.word(332), Some("zebra"));

        assert_eq!(NamedWordList::Color.len(), 52);
        assert_eq!(NamedWordList::Color.word(0), Some("amaranth"));
        assert_eq!(NamedWordList::Color.word(51), Some("yellow"));
    }

    #[cfg(feature = "bip39-english")]
    #[test]
    fn bip39_english_has_explicit_name() {
        assert_eq!(
            NamedWordList::parse("bip39-en"),
            Some(NamedWordList::Bip39English)
        );
        assert_eq!(NamedWordList::parse("word"), None);
        assert_eq!(NamedWordList::Bip39English.len(), 2048);
        assert_eq!(NamedWordList::Bip39English.word(0), Some("abandon"));
    }

    #[test]
    fn sequence_dispatches_by_position() {
        let sequence = WordListSequence::new(&[
            NamedWordList::Color,
            NamedWordList::Adjective,
            NamedWordList::Animal,
        ]);

        assert_eq!(sequence.word_count(), 3);
        assert_eq!(sequence.capacity(), Some(12_969_684));
        assert_eq!(sequence.len(0), 52);
        assert_eq!(sequence.len(1), 749);
        assert_eq!(sequence.len(2), 333);
        assert_eq!(sequence.len(3), 0);
        assert_eq!(sequence.word(0, 0), Some("amaranth"));
        assert_eq!(sequence.word(0, 1), Some("able"));
        assert_eq!(sequence.word(0, 2), Some("aardvark"));
        assert_eq!(sequence.index_of("yellow", 0), Some(51));
        assert_eq!(sequence.index_of("yellow", 1), None);
    }
}
