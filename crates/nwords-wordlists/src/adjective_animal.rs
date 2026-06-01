//! Curated English adjective-animal wordlist.
//!
//! The source lists come from `andreasonny83/unique-names-generator` commit
//! `10ff70b131c8a080e88c315a55e45a0f5caadd24`, with local filtering recorded
//! under `tests/vectors/adjective-animal/`. The source project is MIT
//! licensed; see
//! `tests/vectors/licenses/unique-names-generator-MIT.LICENSE`.

use nwords_core::WordMap;

use crate::{adjective_animal_adjectives, adjective_animal_animals};

/// Position for adjectives in [`AdjectiveAnimal`].
pub const ADJECTIVE_POSITION: usize = 0;

/// Position for animals in [`AdjectiveAnimal`].
pub const ANIMAL_POSITION: usize = 1;

/// Number of positions in [`AdjectiveAnimal`].
pub const WORD_COUNT: usize = 2;

/// Number of adjective words.
pub const ADJECTIVE_COUNT: usize = adjective_animal_adjectives::ADJECTIVES.len();

/// Number of animal words.
pub const ANIMAL_COUNT: usize = adjective_animal_animals::ANIMALS.len();

/// Exact representational capacity of the adjective-animal shape.
pub const CAPACITY: u128 = (ADJECTIVE_COUNT as u128) * (ANIMAL_COUNT as u128);

/// English adjective-animal word map.
///
/// Position 0 is an adjective and position 1 is an animal. The order is part
/// of the encoding contract.
#[derive(Debug, Clone, Copy, Default)]
pub struct AdjectiveAnimal;

impl WordMap for AdjectiveAnimal {
    fn len(&self, position: usize) -> usize {
        match position {
            ADJECTIVE_POSITION => ADJECTIVE_COUNT,
            ANIMAL_POSITION => ANIMAL_COUNT,
            _ => 0,
        }
    }

    fn word(&self, index: usize, position: usize) -> Option<&str> {
        match position {
            ADJECTIVE_POSITION => adjective_animal_adjectives::ADJECTIVES.get(index).copied(),
            ANIMAL_POSITION => adjective_animal_animals::ANIMALS.get(index).copied(),
            _ => None,
        }
    }

    fn index_of(&self, word: &str, position: usize) -> Option<usize> {
        match position {
            ADJECTIVE_POSITION => adjective_animal_adjectives::ADJECTIVES
                .iter()
                .position(|candidate| *candidate == word),
            ANIMAL_POSITION => adjective_animal_animals::ANIMALS
                .iter()
                .position(|candidate| *candidate == word),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AdjectiveAnimal, ADJECTIVE_COUNT, ANIMAL_COUNT, CAPACITY, WORD_COUNT};
    use nwords_core::WordMap;

    #[test]
    fn adjective_animal_has_expected_shape() {
        let words = AdjectiveAnimal;

        assert_eq!(WORD_COUNT, 2);
        assert_eq!(ADJECTIVE_COUNT, 749);
        assert_eq!(ANIMAL_COUNT, 333);
        assert_eq!(CAPACITY, 249_417);
        assert_eq!(words.len(0), ADJECTIVE_COUNT);
        assert_eq!(words.len(1), ANIMAL_COUNT);
        assert_eq!(words.len(2), 0);
    }

    #[test]
    fn adjective_position_has_expected_boundary_words() {
        let words = AdjectiveAnimal;

        assert_eq!(words.word(0, 0), Some("able"));
        assert_eq!(words.word(748, 0), Some("zippy"));
        assert_eq!(words.index_of("brave", 0), Some(78));
        assert_eq!(words.index_of("brave", 1), None);
    }

    #[test]
    fn animal_position_has_expected_boundary_words() {
        let words = AdjectiveAnimal;

        assert_eq!(words.word(0, 1), Some("aardvark"));
        assert_eq!(words.word(332, 1), Some("zebra"));
        assert_eq!(words.index_of("zebra", 1), Some(332));
        assert_eq!(words.index_of("zebra", 0), None);
    }
}
