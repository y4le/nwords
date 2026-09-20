//! Named word lists and ordered phrase-shape adapters.
//!
//! Names and word order are encoding contracts. Changing a list's contents or
//! order requires a new list name.

#[cfg(feature = "alloc")]
use alloc::{collections::BTreeMap, string::String, vec::Vec};

use nwords_core::WordMap;

use crate::{
    adjective_animal_adjectives, adjective_animal_animals, friendly_words_descriptors,
    friendly_words_objects, semantic_foods, semantic_materials, semantic_moods, semantic_plants,
    semantic_shapes, semantic_weather, unique_names_generator_colors,
};

/// Advisory grammar role for a word list in phrase-shape planning.
///
/// Roles are metadata for documentation, preset selection, and optional CLI
/// warnings. They are not codec constraints; position still determines decode
/// behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordListRole {
    /// List entries naturally modify a following head noun.
    Modifier,
    /// List entries naturally act as phrase head nouns.
    Head,
    /// List entries can reasonably be used as modifiers or heads.
    Either,
}

/// Built-in single-position word list names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NamedWordList {
    /// Curated adjective list from `unique-names-generator`.
    Adjective,
    /// Curated animal list from `unique-names-generator`.
    Animal,
    /// Color list from `unique-names-generator`.
    Color,
    /// Broad friendly head-noun list from Glitch `friendly-words`.
    Object,
    /// Friendly modifier list from Glitch `friendly-words` predicates.
    Descriptor,
    /// Authored friendly mood modifier list.
    Mood,
    /// Authored material list for visual phrase shapes.
    Material,
    /// Authored shape/form list for visual phrase shapes.
    Shape,
    /// Authored weather/outdoor-condition modifier list.
    Weather,
    /// Authored ornamental plant and garden head-noun list.
    Plant,
    /// Authored food/ingredient/dish head-noun list.
    Food,
    /// English BIP-39 wordlist used as a positional dictionary.
    #[cfg(feature = "bip39-english")]
    Bip39English,
    /// EFF long dictionary in original dice-roll order (7,776 words).
    #[cfg(feature = "eff-long")]
    EffLong,
}

impl NamedWordList {
    /// Parses a built-in word list name.
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "adjective" | "adjectives" => Some(Self::Adjective),
            "animal" | "animals" => Some(Self::Animal),
            "color" | "colors" => Some(Self::Color),
            "object" | "objects" => Some(Self::Object),
            "descriptor" | "descriptors" => Some(Self::Descriptor),
            "mood" | "moods" => Some(Self::Mood),
            "material" | "materials" => Some(Self::Material),
            "shape" | "shapes" => Some(Self::Shape),
            // Weather is intentionally singular because it is used here as an
            // uncountable condition category.
            "weather" => Some(Self::Weather),
            "plant" | "plants" => Some(Self::Plant),
            "food" | "foods" => Some(Self::Food),
            #[cfg(feature = "eff-long")]
            "eff-long" => Some(Self::EffLong),
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
            Self::Object => "object",
            Self::Descriptor => "descriptor",
            Self::Mood => "mood",
            Self::Material => "material",
            Self::Shape => "shape",
            Self::Weather => "weather",
            Self::Plant => "plant",
            Self::Food => "food",
            #[cfg(feature = "eff-long")]
            Self::EffLong => "eff-long",
            #[cfg(feature = "bip39-english")]
            Self::Bip39English => "bip39-en",
        }
    }

    /// Returns the advisory grammar role for this list.
    pub const fn role(self) -> WordListRole {
        match self {
            Self::Adjective | Self::Color | Self::Descriptor | Self::Mood | Self::Weather => {
                WordListRole::Modifier
            }
            Self::Animal | Self::Object | Self::Plant | Self::Food => WordListRole::Head,
            Self::Material | Self::Shape => WordListRole::Either,
            #[cfg(feature = "eff-long")]
            Self::EffLong => WordListRole::Either,
            #[cfg(feature = "bip39-english")]
            Self::Bip39English => WordListRole::Either,
        }
    }

    /// Returns the number of words in this list.
    pub fn len(self) -> usize {
        match self {
            Self::Adjective => adjective_animal_adjectives::ADJECTIVES.len(),
            Self::Animal => adjective_animal_animals::ANIMALS.len(),
            Self::Color => unique_names_generator_colors::COLORS.len(),
            Self::Object => friendly_words_objects::OBJECTS.len(),
            Self::Descriptor => friendly_words_descriptors::DESCRIPTORS.len(),
            Self::Mood => semantic_moods::MOODS.len(),
            Self::Material => semantic_materials::MATERIALS.len(),
            Self::Shape => semantic_shapes::SHAPES.len(),
            Self::Weather => semantic_weather::WEATHER.len(),
            Self::Plant => semantic_plants::PLANTS.len(),
            Self::Food => semantic_foods::FOODS.len(),
            #[cfg(feature = "eff-long")]
            Self::EffLong => crate::eff_long::WORDS.len(),
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
            Self::Object => friendly_words_objects::OBJECTS.get(index).copied(),
            Self::Descriptor => friendly_words_descriptors::DESCRIPTORS.get(index).copied(),
            Self::Mood => semantic_moods::MOODS.get(index).copied(),
            Self::Material => semantic_materials::MATERIALS.get(index).copied(),
            Self::Shape => semantic_shapes::SHAPES.get(index).copied(),
            Self::Weather => semantic_weather::WEATHER.get(index).copied(),
            Self::Plant => semantic_plants::PLANTS.get(index).copied(),
            Self::Food => semantic_foods::FOODS.get(index).copied(),
            #[cfg(feature = "eff-long")]
            Self::EffLong => crate::eff_long::WORDS.get(index).copied(),
            #[cfg(feature = "bip39-english")]
            Self::Bip39English => crate::bip39::English.word(index, 0),
        }
    }

    /// Returns the index for `word`.
    pub fn index_of(self, word: &str) -> Option<usize> {
        match self {
            #[cfg(feature = "eff-long")]
            Self::EffLong => crate::eff_long::WORDS.binary_search(&word).ok(),
            Self::Adjective => adjective_animal_adjectives::ADJECTIVES
                .binary_search(&word)
                .ok(),
            Self::Animal => adjective_animal_animals::ANIMALS.binary_search(&word).ok(),
            Self::Color => unique_names_generator_colors::COLORS
                .binary_search(&word)
                .ok(),
            Self::Object => friendly_words_objects::OBJECTS.binary_search(&word).ok(),
            Self::Descriptor => friendly_words_descriptors::DESCRIPTORS
                .binary_search(&word)
                .ok(),
            Self::Mood => semantic_moods::MOODS.binary_search(&word).ok(),
            Self::Material => semantic_materials::MATERIALS.binary_search(&word).ok(),
            Self::Shape => semantic_shapes::SHAPES.binary_search(&word).ok(),
            Self::Weather => semantic_weather::WEATHER.binary_search(&word).ok(),
            Self::Plant => semantic_plants::PLANTS.binary_search(&word).ok(),
            Self::Food => semantic_foods::FOODS.binary_search(&word).ok(),
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

/// Validation error for owned or dynamic word list construction.
#[cfg(feature = "alloc")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordListError {
    /// A user-defined list name was empty or contained an invalid byte.
    InvalidListName,
    /// A user-defined list name collided with a built-in list name or alias.
    BuiltinNameCollision,
    /// A user-defined list had fewer than two words.
    TooFewWords {
        /// Number of accepted words.
        len: usize,
    },
    /// A word was empty or contained a byte outside the accepted token policy.
    InvalidWord {
        /// Index of the invalid word in the provided word vector.
        index: usize,
    },
    /// A word appeared more than once in a list.
    DuplicateWord {
        /// Index where the word first appeared.
        first: usize,
        /// Index where the duplicate appeared.
        duplicate: usize,
    },
    /// The dynamic sequence contained no positions.
    EmptyShape,
    /// An owned list name appeared more than once in a dynamic sequence pool.
    DuplicateListName {
        /// Index where the list name first appeared.
        first: usize,
        /// Index where the duplicate appeared.
        duplicate: usize,
    },
    /// A dynamic sequence position referenced an owned-list index outside the
    /// owned-list pool.
    OwnedListIndexOutOfRange {
        /// Referenced owned-list index.
        index: usize,
        /// Number of owned lists available.
        len: usize,
    },
}

/// Owned single-position word list for runtime user-defined shapes.
#[cfg(feature = "alloc")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnedWordList {
    name: String,
    role: WordListRole,
    words: Vec<String>,
    lookup_order: Vec<usize>,
}

#[cfg(feature = "alloc")]
impl OwnedWordList {
    /// Creates an owned word list after validating its name and words.
    ///
    /// Words are accepted exactly as provided. The constructor rejects invalid
    /// tokens and duplicates; it does not lowercase, sort, dedupe, or otherwise
    /// repair input.
    pub fn new(
        name: impl Into<String>,
        role: WordListRole,
        words: Vec<String>,
    ) -> Result<Self, WordListError> {
        let name = name.into();
        validate_owned_list_name(&name)?;
        if NamedWordList::parse(&name).is_some() {
            return Err(WordListError::BuiltinNameCollision);
        }
        validate_words(&words)?;
        Ok(Self::validated(name, role, words))
    }

    /// Creates a list of exact Unicode tokens for whitespace-separated phrases.
    ///
    /// Tokens must be nonempty and contain no whitespace, control characters,
    /// or BOM. Case, punctuation, accents, emoji, and order are preserved.
    /// This is separate from `new`, which retains the CLI lowercase-ASCII policy.
    pub fn from_tokens(
        name: impl Into<String>,
        role: WordListRole,
        words: Vec<String>,
    ) -> Result<Self, WordListError> {
        let name = name.into();
        validate_owned_list_name(&name)?;
        if NamedWordList::parse(&name).is_some() {
            return Err(WordListError::BuiltinNameCollision);
        }
        if words.len() < 2 {
            return Err(WordListError::TooFewWords { len: words.len() });
        }
        let mut seen = BTreeMap::new();
        for (index, word) in words.iter().enumerate() {
            if word.is_empty()
                || word
                    .chars()
                    .any(|ch| ch.is_whitespace() || ch.is_control() || ch == '\u{feff}')
            {
                return Err(WordListError::InvalidWord { index });
            }
            if let Some(first) = seen.insert(word, index) {
                return Err(WordListError::DuplicateWord {
                    first,
                    duplicate: index,
                });
            }
        }
        Ok(Self::validated(name, role, words))
    }

    fn validated(name: String, role: WordListRole, words: Vec<String>) -> Self {
        let mut lookup_order = (0..words.len()).collect::<Vec<_>>();
        lookup_order.sort_unstable_by(|a, b| words[*a].cmp(&words[*b]));
        Self {
            name,
            role,
            words,
            lookup_order,
        }
    }

    /// Returns the user-defined list name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the advisory grammar role for this list.
    pub const fn role(&self) -> WordListRole {
        self.role
    }

    /// Returns the number of words in this list.
    pub fn len(&self) -> usize {
        self.words.len()
    }

    /// Returns true when this list has no words.
    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    /// Returns the word at `index`.
    pub fn word(&self, index: usize) -> Option<&str> {
        self.words.get(index).map(String::as_str)
    }

    /// Returns the index for `word`.
    pub fn index_of(&self, word: &str) -> Option<usize> {
        self.lookup_order
            .binary_search_by(|index| self.words[*index].as_str().cmp(word))
            .ok()
            .map(|slot| self.lookup_order[slot])
    }

    /// Returns the backing words in index order.
    pub fn words(&self) -> &[String] {
        &self.words
    }
}

/// Position slot in a dynamic word-list sequence.
#[cfg(feature = "alloc")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DynamicWordListSlot {
    /// Use a built-in named word list at this position.
    Builtin(NamedWordList),
    /// Use an owned word list from the sequence's owned-list pool.
    Owned(usize),
}

/// Owned sequence of built-in and user-defined word lists.
#[cfg(feature = "alloc")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicWordListSequence {
    owned: Vec<OwnedWordList>,
    positions: Vec<DynamicWordListSlot>,
}

#[cfg(feature = "alloc")]
impl DynamicWordListSequence {
    /// Creates a dynamic sequence from an owned-list pool and position slots.
    pub fn new(
        owned: Vec<OwnedWordList>,
        positions: Vec<DynamicWordListSlot>,
    ) -> Result<Self, WordListError> {
        validate_owned_list_names(&owned)?;
        if positions.is_empty() {
            return Err(WordListError::EmptyShape);
        }
        for slot in &positions {
            if let DynamicWordListSlot::Owned(index) = *slot {
                if index >= owned.len() {
                    return Err(WordListError::OwnedListIndexOutOfRange {
                        index,
                        len: owned.len(),
                    });
                }
            }
        }
        Ok(Self { owned, positions })
    }

    /// Creates a dynamic sequence containing only built-in lists.
    pub fn from_builtin(lists: &[NamedWordList]) -> Result<Self, WordListError> {
        let positions = lists
            .iter()
            .copied()
            .map(DynamicWordListSlot::Builtin)
            .collect();
        Self::new(Vec::new(), positions)
    }

    /// Returns the owned-list pool.
    pub fn owned_lists(&self) -> &[OwnedWordList] {
        &self.owned
    }

    /// Returns the position slots.
    pub fn positions(&self) -> &[DynamicWordListSlot] {
        &self.positions
    }

    /// Returns the phrase word count.
    pub fn word_count(&self) -> usize {
        self.positions.len()
    }

    /// Returns the display name for the list at `position`.
    pub fn list_name(&self, position: usize) -> Option<&str> {
        match *self.positions.get(position)? {
            DynamicWordListSlot::Builtin(list) => Some(list.name()),
            DynamicWordListSlot::Owned(index) => self.owned.get(index).map(OwnedWordList::name),
        }
    }

    /// Returns the advisory grammar role for the list at `position`.
    pub fn role(&self, position: usize) -> Option<WordListRole> {
        match *self.positions.get(position)? {
            DynamicWordListSlot::Builtin(list) => Some(list.role()),
            DynamicWordListSlot::Owned(index) => self.owned.get(index).map(OwnedWordList::role),
        }
    }

    fn list_len(&self, position: usize) -> usize {
        match self.positions.get(position).copied() {
            Some(DynamicWordListSlot::Builtin(list)) => list.len(),
            Some(DynamicWordListSlot::Owned(index)) => {
                self.owned.get(index).map_or(0, OwnedWordList::len)
            }
            None => 0,
        }
    }

    fn list_word(&self, index: usize, position: usize) -> Option<&str> {
        match *self.positions.get(position)? {
            DynamicWordListSlot::Builtin(list) => list.word(index),
            DynamicWordListSlot::Owned(owned_index) => self.owned.get(owned_index)?.word(index),
        }
    }

    fn list_index_of(&self, word: &str, position: usize) -> Option<usize> {
        match *self.positions.get(position)? {
            DynamicWordListSlot::Builtin(list) => list.index_of(word),
            DynamicWordListSlot::Owned(index) => self.owned.get(index)?.index_of(word),
        }
    }
}

#[cfg(feature = "alloc")]
impl WordMap for DynamicWordListSequence {
    fn len(&self, position: usize) -> usize {
        self.list_len(position)
    }

    fn word(&self, index: usize, position: usize) -> Option<&str> {
        self.list_word(index, position)
    }

    fn index_of(&self, word: &str, position: usize) -> Option<usize> {
        self.list_index_of(word, position)
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

#[cfg(feature = "alloc")]
fn validate_owned_list_name(name: &str) -> Result<(), WordListError> {
    let mut bytes = name.bytes();
    match bytes.next() {
        Some(first) if first.is_ascii_lowercase() => {}
        _ => return Err(WordListError::InvalidListName),
    }
    for byte in bytes {
        if !(byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' || byte == b'-') {
            return Err(WordListError::InvalidListName);
        }
    }
    Ok(())
}

#[cfg(feature = "alloc")]
fn validate_words(words: &[String]) -> Result<(), WordListError> {
    if words.len() < 2 {
        return Err(WordListError::TooFewWords { len: words.len() });
    }

    let mut seen = BTreeMap::<&str, usize>::new();
    for (index, word) in words.iter().enumerate() {
        if word.is_empty() || !word.bytes().all(|byte| byte.is_ascii_lowercase()) {
            return Err(WordListError::InvalidWord { index });
        }
        if let Some(first) = seen.insert(word.as_str(), index) {
            return Err(WordListError::DuplicateWord {
                first,
                duplicate: index,
            });
        }
    }
    Ok(())
}

#[cfg(feature = "alloc")]
fn validate_owned_list_names(owned: &[OwnedWordList]) -> Result<(), WordListError> {
    let mut seen = BTreeMap::<&str, usize>::new();
    for (index, list) in owned.iter().enumerate() {
        if let Some(first) = seen.insert(list.name(), index) {
            return Err(WordListError::DuplicateListName {
                first,
                duplicate: index,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{NamedWordList, WordListRole, WordListSequence};
    use nwords_core::WordMap;

    #[test]
    fn named_lists_match_checked_in_wordset_snapshots() {
        let snapshots = [
            (
                NamedWordList::Adjective,
                include_str!("../../../tests/vectors/adjective-animal/nwords-adjectives.txt"),
            ),
            (
                NamedWordList::Animal,
                include_str!("../../../tests/vectors/adjective-animal/nwords-animals.txt"),
            ),
            (
                NamedWordList::Color,
                include_str!(
                    "../../../tests/vectors/adjective-animal/unique-names-generator-colors.txt"
                ),
            ),
            (
                NamedWordList::Object,
                include_str!("../../../tests/vectors/friendly-words/nwords-objects.txt"),
            ),
            (
                NamedWordList::Descriptor,
                include_str!("../../../tests/vectors/friendly-words/nwords-descriptors.txt"),
            ),
            (
                NamedWordList::Mood,
                include_str!("../../../tests/vectors/semantic-wordlists/mood/nwords-moods.txt"),
            ),
            (
                NamedWordList::Material,
                include_str!(
                    "../../../tests/vectors/semantic-wordlists/material/nwords-materials.txt"
                ),
            ),
            (
                NamedWordList::Shape,
                include_str!("../../../tests/vectors/semantic-wordlists/shape/nwords-shapes.txt"),
            ),
            (
                NamedWordList::Weather,
                include_str!(
                    "../../../tests/vectors/semantic-wordlists/weather/nwords-weather.txt"
                ),
            ),
            (
                NamedWordList::Plant,
                include_str!("../../../tests/vectors/semantic-wordlists/plant/nwords-plants.txt"),
            ),
            (
                NamedWordList::Food,
                include_str!("../../../tests/vectors/semantic-wordlists/food/nwords-foods.txt"),
            ),
            #[cfg(feature = "eff-long")]
            (
                NamedWordList::EffLong,
                include_str!("../../../tests/vectors/eff-long/words.txt"),
            ),
        ];
        for (list, source) in snapshots {
            assert_eq!(
                list.len(),
                source.lines().count(),
                "{} word count",
                list.name()
            );
            for (index, word) in source.lines().enumerate() {
                assert_eq!(list.word(index), Some(word), "{} word {index}", list.name());
            }
        }
    }

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

        assert_eq!(NamedWordList::Object.len(), 3051);
        assert_eq!(NamedWordList::Object.word(0), Some("aardvark"));
        assert_eq!(NamedWordList::Object.word(3050), Some("zydeco"));

        assert_eq!(NamedWordList::Descriptor.len(), 1437);
        assert_eq!(NamedWordList::Descriptor.word(0), Some("abalone"));
        assert_eq!(NamedWordList::Descriptor.word(1436), Some("zircon"));

        assert_eq!(NamedWordList::Mood.len(), 64);
        assert_eq!(NamedWordList::Mood.word(0), Some("alert"));
        assert_eq!(NamedWordList::Mood.word(63), Some("zestful"));

        assert_eq!(NamedWordList::Material.len(), 64);
        assert_eq!(NamedWordList::Material.word(0), Some("acrylic"));
        assert_eq!(NamedWordList::Material.word(63), Some("zinc"));

        assert_eq!(NamedWordList::Shape.len(), 40);
        assert_eq!(NamedWordList::Shape.word(0), Some("angular"));
        assert_eq!(NamedWordList::Shape.word(39), Some("zigzag"));

        assert_eq!(NamedWordList::Weather.len(), 40);
        assert_eq!(NamedWordList::Weather.word(0), Some("balmy"));
        assert_eq!(NamedWordList::Weather.word(39), Some("wintry"));

        assert_eq!(NamedWordList::Plant.len(), 128);
        assert_eq!(NamedWordList::Plant.word(0), Some("abelia"));
        assert_eq!(NamedWordList::Plant.word(127), Some("zinnia"));

        assert_eq!(NamedWordList::Food.len(), 128);
        assert_eq!(NamedWordList::Food.word(0), Some("almond"));
        assert_eq!(NamedWordList::Food.word(127), Some("zucchini"));
    }

    #[cfg(feature = "eff-long")]
    #[test]
    fn eff_long_matches_frozen_source_and_order() {
        let source = include_str!("../fixtures/eff_large_wordlist.txt");
        let list = NamedWordList::EffLong;
        assert_eq!(list.len(), 7776);
        for (index, row) in source.lines().enumerate() {
            let word = row.split_whitespace().nth(1).expect("source word");
            assert_eq!(list.word(index), Some(word));
            assert_eq!(list.index_of(word), Some(index));
            if index > 0 {
                assert!(list.word(index - 1) < list.word(index));
            }
        }
        assert_eq!(list.word(0), Some("abacus"));
        assert_eq!(list.word(7775), Some("zoom"));
        assert_eq!(list.word(7776), None);
        assert_eq!(list.index_of("Abacus"), None);
        assert_eq!(NamedWordList::parse("eff-long"), Some(list));
    }

    #[test]
    fn named_lists_have_roles() {
        assert_eq!(NamedWordList::Adjective.role(), WordListRole::Modifier);
        assert_eq!(NamedWordList::Animal.role(), WordListRole::Head);
        assert_eq!(NamedWordList::Color.role(), WordListRole::Modifier);
        assert_eq!(NamedWordList::Object.role(), WordListRole::Head);
        assert_eq!(NamedWordList::Descriptor.role(), WordListRole::Modifier);
        assert_eq!(NamedWordList::Mood.role(), WordListRole::Modifier);
        assert_eq!(NamedWordList::Material.role(), WordListRole::Either);
        assert_eq!(NamedWordList::Shape.role(), WordListRole::Either);
        assert_eq!(NamedWordList::Weather.role(), WordListRole::Modifier);
        assert_eq!(NamedWordList::Plant.role(), WordListRole::Head);
        assert_eq!(NamedWordList::Food.role(), WordListRole::Head);

        #[cfg(feature = "bip39-english")]
        assert_eq!(NamedWordList::Bip39English.role(), WordListRole::Either);
    }

    #[test]
    fn named_list_parser_accepts_semantic_aliases() {
        assert_eq!(NamedWordList::parse("mood"), Some(NamedWordList::Mood));
        assert_eq!(NamedWordList::parse("moods"), Some(NamedWordList::Mood));
        assert_eq!(
            NamedWordList::parse("material"),
            Some(NamedWordList::Material)
        );
        assert_eq!(
            NamedWordList::parse("materials"),
            Some(NamedWordList::Material)
        );
        assert_eq!(NamedWordList::parse("shape"), Some(NamedWordList::Shape));
        assert_eq!(NamedWordList::parse("shapes"), Some(NamedWordList::Shape));
        assert_eq!(
            NamedWordList::parse("weather"),
            Some(NamedWordList::Weather)
        );
        assert_eq!(NamedWordList::parse("weathers"), None);
        assert_eq!(NamedWordList::parse("plant"), Some(NamedWordList::Plant));
        assert_eq!(NamedWordList::parse("plants"), Some(NamedWordList::Plant));
        assert_eq!(NamedWordList::parse("food"), Some(NamedWordList::Food));
        assert_eq!(NamedWordList::parse("foods"), Some(NamedWordList::Food));
    }

    #[test]
    fn binary_searched_lists_are_strictly_sorted_in_encoding_order() {
        for list in [
            NamedWordList::Adjective,
            NamedWordList::Animal,
            NamedWordList::Color,
            NamedWordList::Object,
            NamedWordList::Descriptor,
            NamedWordList::Mood,
            NamedWordList::Material,
            NamedWordList::Shape,
            NamedWordList::Weather,
            NamedWordList::Plant,
            NamedWordList::Food,
        ] {
            for index in 1..list.len() {
                assert!(
                    list.word(index - 1) < list.word(index),
                    "{}[{index}]",
                    list.name()
                );
            }
            for missing in ["", "Aardvark", "aardvark!", "zzzzzz"] {
                assert_eq!(list.index_of(missing), None, "{}: {missing}", list.name());
            }
        }
    }

    #[test]
    fn named_lists_round_trip_indexes_without_duplicates() {
        for list in [
            NamedWordList::Adjective,
            NamedWordList::Animal,
            NamedWordList::Color,
            NamedWordList::Object,
            NamedWordList::Descriptor,
            NamedWordList::Mood,
            NamedWordList::Material,
            NamedWordList::Shape,
            NamedWordList::Weather,
            NamedWordList::Plant,
            NamedWordList::Food,
        ] {
            for index in 0..list.len() {
                let word = list.word(index).expect("word exists");
                assert_eq!(list.index_of(word), Some(index), "{}[{index}]", list.name());
            }
        }
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

    #[test]
    fn descriptor_object_sequence_has_stable_capacity() {
        let sequence = WordListSequence::new(&[NamedWordList::Descriptor, NamedWordList::Object]);

        assert_eq!(sequence.word_count(), 2);
        assert_eq!(sequence.capacity(), Some(4_384_287));
        assert_eq!(sequence.len(0), 1437);
        assert_eq!(sequence.len(1), 3051);
        assert_eq!(sequence.word(0, 0), Some("abalone"));
        assert_eq!(sequence.word(0, 1), Some("aardvark"));
        assert_eq!(sequence.index_of("zircon", 0), Some(1436));
        assert_eq!(sequence.index_of("zydeco", 1), Some(3050));
        assert_eq!(sequence.index_of("zydeco", 0), None);
    }

    #[test]
    fn authored_semantic_sequences_have_stable_capacities() {
        let mood_descriptor_object = WordListSequence::new(&[
            NamedWordList::Mood,
            NamedWordList::Descriptor,
            NamedWordList::Object,
        ]);
        assert_eq!(mood_descriptor_object.capacity(), Some(280_594_368));
        assert_eq!(mood_descriptor_object.word(0, 0), Some("alert"));
        assert_eq!(mood_descriptor_object.word(0, 1), Some("abalone"));
        assert_eq!(mood_descriptor_object.word(0, 2), Some("aardvark"));

        let material_shape_object = WordListSequence::new(&[
            NamedWordList::Material,
            NamedWordList::Shape,
            NamedWordList::Object,
        ]);
        assert_eq!(material_shape_object.capacity(), Some(7_810_560));
        assert_eq!(material_shape_object.word(0, 0), Some("acrylic"));
        assert_eq!(material_shape_object.word(0, 1), Some("angular"));
        assert_eq!(material_shape_object.word(0, 2), Some("aardvark"));

        let weather_descriptor_plant = WordListSequence::new(&[
            NamedWordList::Weather,
            NamedWordList::Descriptor,
            NamedWordList::Plant,
        ]);
        assert_eq!(weather_descriptor_plant.capacity(), Some(7_357_440));
        assert_eq!(weather_descriptor_plant.word(0, 0), Some("balmy"));
        assert_eq!(weather_descriptor_plant.word(0, 1), Some("abalone"));
        assert_eq!(weather_descriptor_plant.word(0, 2), Some("abelia"));

        let mood_descriptor_food = WordListSequence::new(&[
            NamedWordList::Mood,
            NamedWordList::Descriptor,
            NamedWordList::Food,
        ]);
        assert_eq!(mood_descriptor_food.capacity(), Some(11_771_904));
        assert_eq!(mood_descriptor_food.word(0, 0), Some("alert"));
        assert_eq!(mood_descriptor_food.word(0, 1), Some("abalone"));
        assert_eq!(mood_descriptor_food.word(0, 2), Some("almond"));

        let material_shape_food = WordListSequence::new(&[
            NamedWordList::Material,
            NamedWordList::Shape,
            NamedWordList::Food,
        ]);
        assert_eq!(material_shape_food.capacity(), Some(327_680));
        assert_eq!(material_shape_food.word(0, 0), Some("acrylic"));
        assert_eq!(material_shape_food.word(0, 1), Some("angular"));
        assert_eq!(material_shape_food.word(0, 2), Some("almond"));
    }

    #[cfg(feature = "alloc")]
    mod dynamic_tests {
        use super::super::{
            DynamicWordListSequence, DynamicWordListSlot, NamedWordList, OwnedWordList,
            WordListError, WordListRole,
        };
        use alloc::{string::ToString, vec};
        use nwords_core::WordMap;

        fn owned(name: &str, words: &[&str]) -> OwnedWordList {
            OwnedWordList::new(
                name.to_string(),
                WordListRole::Either,
                words.iter().map(|word| (*word).to_string()).collect(),
            )
            .expect("owned list")
        }

        #[test]
        fn owned_word_list_validates_inputs() {
            assert_eq!(
                OwnedWordList::new(
                    "animal".to_string(),
                    WordListRole::Head,
                    vec!["cat".to_string(), "dog".to_string()]
                ),
                Err(WordListError::BuiltinNameCollision)
            );
            assert_eq!(
                OwnedWordList::new(
                    "my-list".to_string(),
                    WordListRole::Head,
                    vec!["cat".to_string()]
                ),
                Err(WordListError::TooFewWords { len: 1 })
            );
            assert_eq!(
                OwnedWordList::new(
                    "my-list".to_string(),
                    WordListRole::Head,
                    vec!["cat".to_string(), "cat".to_string()]
                ),
                Err(WordListError::DuplicateWord {
                    first: 0,
                    duplicate: 1
                })
            );
            assert_eq!(
                OwnedWordList::new(
                    "my-list".to_string(),
                    WordListRole::Head,
                    vec!["cat".to_string(), "tabby1".to_string()]
                ),
                Err(WordListError::InvalidWord { index: 1 })
            );
        }

        #[test]
        fn unicode_tokens_keep_order_and_index_exactly() {
            let words = ["🦊", "Cat", "cat", "é", "e\u{301}", "yo-yo", "\"quote\""];
            let list = OwnedWordList::from_tokens(
                "tokens",
                WordListRole::Either,
                words.iter().map(|w| (*w).into()).collect(),
            )
            .unwrap();
            for (index, word) in words.iter().enumerate() {
                assert_eq!(list.word(index), Some(*word));
                assert_eq!(list.index_of(word), Some(index));
            }
            assert_eq!(list.index_of("CAT"), None);
            for invalid in ["a b", "a\t", "\0", "\u{feff}", ""] {
                assert!(OwnedWordList::from_tokens(
                    "tokens",
                    WordListRole::Either,
                    vec![invalid.into(), "valid".into()]
                )
                .is_err());
            }
            assert!(OwnedWordList::from_tokens(
                "tokens",
                WordListRole::Either,
                vec!["same".into(), "same".into()]
            )
            .is_err());
            assert!(OwnedWordList::new(
                "tokens",
                WordListRole::Either,
                vec!["Cat".into(), "dog".into()]
            )
            .is_err());
        }

        #[test]
        fn dynamic_sequence_dispatches_builtin_and_owned_positions() {
            let sequence = DynamicWordListSequence::new(
                vec![owned("project", &["alpha", "bravo"])],
                vec![
                    DynamicWordListSlot::Builtin(NamedWordList::Color),
                    DynamicWordListSlot::Owned(0),
                ],
            )
            .expect("sequence");

            assert_eq!(sequence.word_count(), 2);
            assert_eq!(sequence.len(0), 52);
            assert_eq!(sequence.len(1), 2);
            assert_eq!(sequence.word(0, 0), Some("amaranth"));
            assert_eq!(sequence.word(1, 1), Some("bravo"));
            assert_eq!(sequence.index_of("yellow", 0), Some(51));
            assert_eq!(sequence.index_of("bravo", 1), Some(1));
            assert_eq!(sequence.index_of("yellow", 1), None);
            assert_eq!(sequence.list_name(0), Some("color"));
            assert_eq!(sequence.list_name(1), Some("project"));
            assert_eq!(sequence.role(0), Some(WordListRole::Modifier));
            assert_eq!(sequence.role(1), Some(WordListRole::Either));
        }

        #[test]
        fn dynamic_sequence_validates_positions_and_names() {
            assert_eq!(
                DynamicWordListSequence::from_builtin(&[]),
                Err(WordListError::EmptyShape)
            );
            assert_eq!(
                DynamicWordListSequence::new(
                    vec![owned("one", &["alpha", "bravo"])],
                    vec![DynamicWordListSlot::Owned(1)]
                ),
                Err(WordListError::OwnedListIndexOutOfRange { index: 1, len: 1 })
            );
            assert_eq!(
                DynamicWordListSequence::new(
                    vec![
                        owned("one", &["alpha", "bravo"]),
                        owned("one", &["charlie", "delta"])
                    ],
                    vec![DynamicWordListSlot::Owned(0)]
                ),
                Err(WordListError::DuplicateListName {
                    first: 0,
                    duplicate: 1
                })
            );
        }
    }
}
