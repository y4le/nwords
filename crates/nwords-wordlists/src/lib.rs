#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Built-in wordlists for `nwords`.

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "named")]
#[path = "adjective_animal_adjectives.rs"]
pub(crate) mod adjective_animal_adjectives;

#[cfg(feature = "named")]
#[path = "adjective_animal_animals.rs"]
pub(crate) mod adjective_animal_animals;

#[cfg(feature = "named")]
#[path = "unique_names_generator_colors.rs"]
pub(crate) mod unique_names_generator_colors;

#[cfg(feature = "named")]
#[path = "friendly_words_objects.rs"]
pub(crate) mod friendly_words_objects;

#[cfg(feature = "named")]
#[path = "friendly_words_descriptors.rs"]
pub(crate) mod friendly_words_descriptors;

/// Curated English adjective-animal wordlists.
#[cfg(feature = "adjective-animal")]
pub mod adjective_animal;

/// Named single-position word lists and ordered phrase-shape adapters.
#[cfg(feature = "named")]
pub mod named;

/// BIP-39 wordlists.
pub mod bip39;
