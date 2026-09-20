#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WordPolicy {
    pub min_len: usize,
    pub max_len: Option<usize>,
}

impl WordPolicy {
    pub const fn any_len() -> Self {
        Self {
            min_len: 1,
            max_len: None,
        }
    }

    pub const fn bounded(min_len: usize, max_len: usize) -> Self {
        Self {
            min_len,
            max_len: Some(max_len),
        }
    }

    fn accepts_len(self, word: &str) -> bool {
        word.len() >= self.min_len && self.max_len.is_none_or(|max_len| word.len() <= max_len)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolError {
    Io {
        path: PathBuf,
        message: String,
    },
    InvalidWord {
        index: usize,
    },
    DuplicateWord {
        first: usize,
        duplicate: usize,
    },
    UnknownBlockedWord {
        index: usize,
    },
    Overlap {
        artifact: &'static str,
        index: usize,
    },
    Mismatch {
        artifact: &'static str,
    },
    EmptyList,
    Usage(String),
}

impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, message } => write!(f, "{}: {}", path.display(), message),
            Self::InvalidWord { index } => write!(f, "invalid word at index {index}"),
            Self::DuplicateWord { first, duplicate } => {
                write!(
                    f,
                    "duplicate word at index {duplicate}; first seen at {first}"
                )
            }
            Self::UnknownBlockedWord { index } => {
                write!(f, "blocklist word at index {index} is absent from upstream")
            }
            Self::Overlap { artifact, index } => {
                write!(
                    f,
                    "{artifact} word lists overlap at right-list index {index}"
                )
            }
            Self::Mismatch { artifact } => write!(f, "{artifact} does not match expected output"),
            Self::EmptyList => f.write_str("word list is empty"),
            Self::Usage(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for ToolError {}

pub fn read_word_file(path: &Path) -> Result<Vec<String>, ToolError> {
    let text = fs::read_to_string(path).map_err(|error| ToolError::Io {
        path: path.to_path_buf(),
        message: error.to_string(),
    })?;
    Ok(parse_word_lines(&text))
}

pub fn read_rust_array_words(path: &Path) -> Result<Vec<String>, ToolError> {
    let text = fs::read_to_string(path).map_err(|error| ToolError::Io {
        path: path.to_path_buf(),
        message: error.to_string(),
    })?;
    let mut words = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(word) = trimmed
            .strip_prefix('"')
            .and_then(|rest| rest.strip_suffix("\","))
        {
            words.push(word.to_owned());
        }
    }
    Ok(words)
}

pub fn parse_word_lines(text: &str) -> Vec<String> {
    text.lines().map(str::to_owned).collect()
}

pub fn validate_words(words: &[String], policy: WordPolicy) -> Result<(), ToolError> {
    if words.is_empty() {
        return Err(ToolError::EmptyList);
    }
    validate_word_entries(words, policy)
}

pub fn validate_blocklist(words: &[String]) -> Result<(), ToolError> {
    validate_word_entries(words, WordPolicy::any_len())
}

fn validate_word_entries(words: &[String], policy: WordPolicy) -> Result<(), ToolError> {
    let mut seen = BTreeSet::<&str>::new();
    for (index, word) in words.iter().enumerate() {
        if !is_lowercase_ascii_token(word) || !policy.accepts_len(word) {
            return Err(ToolError::InvalidWord { index });
        }
        if !seen.insert(word) {
            let first = words
                .iter()
                .position(|candidate| candidate == word)
                .unwrap_or(index);
            return Err(ToolError::DuplicateWord {
                first,
                duplicate: index,
            });
        }
    }
    Ok(())
}

pub fn is_lowercase_ascii_token(word: &str) -> bool {
    !word.is_empty() && word.bytes().all(|byte| byte.is_ascii_lowercase())
}

pub fn derive_curated_list(
    upstream: &[String],
    blocklist: &[String],
    policy: WordPolicy,
) -> Result<Vec<String>, ToolError> {
    validate_words(upstream, WordPolicy::any_len())?;
    validate_blocklist(blocklist)?;
    let upstream_words = upstream.iter().map(String::as_str).collect::<BTreeSet<_>>();
    for (index, word) in blocklist.iter().enumerate() {
        if !upstream_words.contains(word.as_str()) {
            return Err(ToolError::UnknownBlockedWord { index });
        }
    }

    let blocked = blocklist
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let curated = upstream
        .iter()
        .filter(|word| policy.accepts_len(word))
        .filter(|word| !blocked.contains(word.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    validate_words(&curated, WordPolicy::any_len())?;
    Ok(curated)
}

pub fn require_exact(
    artifact: &'static str,
    actual: &[String],
    expected: &[String],
) -> Result<(), ToolError> {
    if actual == expected {
        Ok(())
    } else {
        Err(ToolError::Mismatch { artifact })
    }
}

pub fn emit_rust_array(const_name: &str, words: &[String]) -> Result<String, ToolError> {
    validate_const_name(const_name)?;
    validate_words(words, WordPolicy::any_len())?;

    let mut output = String::new();
    output.push_str("pub(crate) const ");
    output.push_str(const_name);
    output.push_str(": &[&str] = &[\n");
    for word in words {
        output.push_str("    \"");
        output.push_str(word);
        output.push_str("\",\n");
    }
    output.push_str("];\n");
    Ok(output)
}

pub fn sample_shape(lists: &[Vec<String>], max_samples: usize) -> Result<Vec<String>, ToolError> {
    if lists.is_empty() || max_samples == 0 {
        return Ok(Vec::new());
    }
    for list in lists {
        validate_words(list, WordPolicy::any_len())?;
    }

    let index_sets = lists
        .iter()
        .map(|list| sample_indices(list.len()))
        .collect::<Vec<_>>();
    let mut samples = Vec::new();
    let mut current = Vec::new();
    collect_samples(
        lists,
        &index_sets,
        0,
        &mut current,
        &mut samples,
        max_samples,
    );
    Ok(samples)
}

pub fn check_existing_vectors(root: &Path) -> Result<(), ToolError> {
    check_unique_names_generator_vectors(root)?;
    check_friendly_words_vectors(root)?;
    check_authored_semantic_wordlists(root)?;
    Ok(())
}

fn check_unique_names_generator_vectors(root: &Path) -> Result<(), ToolError> {
    let base = root.join("tests/vectors/adjective-animal");

    let upstream_adjectives = read_word_file(&base.join("unique-names-generator-adjectives.txt"))?;
    let upstream_animals = read_word_file(&base.join("unique-names-generator-animals.txt"))?;
    let upstream_colors = read_word_file(&base.join("unique-names-generator-colors.txt"))?;
    let adjective_blocklist = read_word_file(&base.join("adjective-blocklist.txt"))?;
    let animal_blocklist = read_word_file(&base.join("animal-blocklist.txt"))?;
    let expected_adjectives = read_word_file(&base.join("nwords-adjectives.txt"))?;
    let expected_animals = read_word_file(&base.join("nwords-animals.txt"))?;
    let rust_adjectives = read_rust_array_words(
        &root.join("crates/nwords-wordlists/src/adjective_animal_adjectives.rs"),
    )?;
    let rust_animals = read_rust_array_words(
        &root.join("crates/nwords-wordlists/src/adjective_animal_animals.rs"),
    )?;
    let rust_colors = read_rust_array_words(
        &root.join("crates/nwords-wordlists/src/unique_names_generator_colors.rs"),
    )?;

    let adjectives = derive_curated_list(
        &upstream_adjectives,
        &adjective_blocklist,
        WordPolicy::bounded(3, 11),
    )?;
    let animals = derive_curated_list(&upstream_animals, &animal_blocklist, WordPolicy::any_len())?;

    validate_words(&upstream_colors, WordPolicy::any_len())?;
    require_exact("nwords-adjectives", &adjectives, &expected_adjectives)?;
    require_exact("nwords-animals", &animals, &expected_animals)?;
    require_exact("rust adjectives", &expected_adjectives, &rust_adjectives)?;
    require_exact("rust animals", &expected_animals, &rust_animals)?;
    require_exact("rust colors", &upstream_colors, &rust_colors)?;
    Ok(())
}

fn check_friendly_words_vectors(root: &Path) -> Result<(), ToolError> {
    let base = root.join("tests/vectors/friendly-words");

    let upstream_objects = read_word_file(&base.join("glitch-friendly-words-objects.txt"))?;
    let upstream_descriptors = read_word_file(&base.join("glitch-friendly-words-predicates.txt"))?;
    let object_blocklist = read_word_file(&base.join("object-blocklist.txt"))?;
    let descriptor_blocklist = read_word_file(&base.join("descriptor-blocklist.txt"))?;
    let expected_objects = read_word_file(&base.join("nwords-objects.txt"))?;
    let expected_descriptors = read_word_file(&base.join("nwords-descriptors.txt"))?;
    let rust_objects =
        read_rust_array_words(&root.join("crates/nwords-wordlists/src/friendly_words_objects.rs"))?;
    let rust_descriptors = read_rust_array_words(
        &root.join("crates/nwords-wordlists/src/friendly_words_descriptors.rs"),
    )?;

    let objects = derive_curated_list(&upstream_objects, &object_blocklist, WordPolicy::any_len())?;
    let descriptors = derive_curated_list(
        &upstream_descriptors,
        &descriptor_blocklist,
        WordPolicy::any_len(),
    )?;

    require_exact("nwords-objects", &objects, &expected_objects)?;
    require_exact("nwords-descriptors", &descriptors, &expected_descriptors)?;
    require_exact("rust objects", &expected_objects, &rust_objects)?;
    require_exact("rust descriptors", &expected_descriptors, &rust_descriptors)?;
    Ok(())
}

fn check_authored_semantic_wordlists(root: &Path) -> Result<(), ToolError> {
    for (artifact, vector_path, rust_path, expected_len) in [
        (
            "nwords-moods",
            "tests/vectors/semantic-wordlists/mood/nwords-moods.txt",
            "crates/nwords-wordlists/src/semantic_moods.rs",
            64,
        ),
        (
            "nwords-materials",
            "tests/vectors/semantic-wordlists/material/nwords-materials.txt",
            "crates/nwords-wordlists/src/semantic_materials.rs",
            64,
        ),
        (
            "nwords-shapes",
            "tests/vectors/semantic-wordlists/shape/nwords-shapes.txt",
            "crates/nwords-wordlists/src/semantic_shapes.rs",
            40,
        ),
        (
            "nwords-weather",
            "tests/vectors/semantic-wordlists/weather/nwords-weather.txt",
            "crates/nwords-wordlists/src/semantic_weather.rs",
            40,
        ),
        (
            "nwords-plants",
            "tests/vectors/semantic-wordlists/plant/nwords-plants.txt",
            "crates/nwords-wordlists/src/semantic_plants.rs",
            128,
        ),
        (
            "nwords-foods",
            "tests/vectors/semantic-wordlists/food/nwords-foods.txt",
            "crates/nwords-wordlists/src/semantic_foods.rs",
            128,
        ),
    ] {
        let words = read_word_file(&root.join(vector_path))?;
        let rust_words = read_rust_array_words(&root.join(rust_path))?;
        validate_words(&words, WordPolicy::any_len())?;
        require_len(artifact, &words, expected_len)?;
        require_sorted(artifact, &words)?;
        require_exact(artifact, &words, &rust_words)?;
    }
    let plants =
        read_word_file(&root.join("tests/vectors/semantic-wordlists/plant/nwords-plants.txt"))?;
    let foods =
        read_word_file(&root.join("tests/vectors/semantic-wordlists/food/nwords-foods.txt"))?;
    require_disjoint("nwords-plants-foods", &plants, &foods)?;
    Ok(())
}

fn require_len(artifact: &'static str, words: &[String], expected: usize) -> Result<(), ToolError> {
    if words.len() == expected {
        Ok(())
    } else {
        Err(ToolError::Mismatch { artifact })
    }
}

fn require_sorted(artifact: &'static str, words: &[String]) -> Result<(), ToolError> {
    if words.windows(2).all(|pair| pair[0] < pair[1]) {
        Ok(())
    } else {
        Err(ToolError::Mismatch { artifact })
    }
}

fn require_disjoint(
    artifact: &'static str,
    left: &[String],
    right: &[String],
) -> Result<(), ToolError> {
    let left_words = left.iter().map(String::as_str).collect::<BTreeSet<_>>();
    if let Some(index) = right
        .iter()
        .position(|word| left_words.contains(word.as_str()))
    {
        Err(ToolError::Overlap { artifact, index })
    } else {
        Ok(())
    }
}

fn validate_const_name(name: &str) -> Result<(), ToolError> {
    if name.is_empty()
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
        || name
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_digit())
    {
        return Err(ToolError::Usage(
            "const name must be uppercase ASCII with underscores".to_owned(),
        ));
    }
    Ok(())
}

fn sample_indices(len: usize) -> Vec<usize> {
    let mut indices = Vec::new();
    for index in [0, len / 2, len.saturating_sub(1)] {
        if !indices.contains(&index) {
            indices.push(index);
        }
    }
    indices
}

fn collect_samples(
    lists: &[Vec<String>],
    index_sets: &[Vec<usize>],
    position: usize,
    current: &mut Vec<String>,
    samples: &mut Vec<String>,
    max_samples: usize,
) {
    if samples.len() >= max_samples {
        return;
    }
    if position == lists.len() {
        samples.push(current.join(" "));
        return;
    }
    for &index in &index_sets[position] {
        if let Some(word) = lists[position].get(index) {
            current.push(word.clone());
            collect_samples(
                lists,
                index_sets,
                position + 1,
                current,
                samples,
                max_samples,
            );
            current.pop();
        }
        if samples.len() >= max_samples {
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        check_existing_vectors, derive_curated_list, emit_rust_array, require_disjoint,
        require_sorted, sample_shape, validate_words, ToolError, WordPolicy,
    };
    use std::path::Path;

    #[test]
    fn rejects_invalid_and_duplicate_words() {
        assert_eq!(
            validate_words(
                &["alpha".to_owned(), "alpha".to_owned()],
                WordPolicy::any_len()
            ),
            Err(ToolError::DuplicateWord {
                first: 0,
                duplicate: 1
            })
        );
        assert_eq!(
            validate_words(
                &["alpha".to_owned(), "Bravo".to_owned()],
                WordPolicy::any_len()
            ),
            Err(ToolError::InvalidWord { index: 1 })
        );
        assert_eq!(
            validate_words(
                &["alpha".to_owned(), "two-words".to_owned()],
                WordPolicy::any_len()
            ),
            Err(ToolError::InvalidWord { index: 1 })
        );
    }

    #[test]
    fn derives_curated_list_in_upstream_order() {
        let upstream = ["able", "bad", "calm", "daring"]
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        let blocklist = ["bad"].into_iter().map(str::to_owned).collect::<Vec<_>>();

        assert_eq!(
            derive_curated_list(&upstream, &blocklist, WordPolicy::bounded(4, 6)).unwrap(),
            vec!["able".to_owned(), "calm".to_owned(), "daring".to_owned()]
        );
    }

    #[test]
    fn empty_blocklist_blocks_nothing() {
        let upstream = ["able", "calm"]
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>();

        assert_eq!(
            derive_curated_list(&upstream, &[], WordPolicy::any_len()).unwrap(),
            vec!["able".to_owned(), "calm".to_owned()]
        );
    }

    #[test]
    fn stale_blocklist_entry_is_rejected() {
        let upstream = vec!["able".to_owned(), "calm".to_owned()];
        let blocklist = vec!["absent".to_owned()];
        assert_eq!(
            derive_curated_list(&upstream, &blocklist, WordPolicy::any_len()),
            Err(ToolError::UnknownBlockedWord { index: 0 })
        );
    }

    #[test]
    fn emits_deterministic_rust_array() {
        let words = ["alpha", "bravo"]
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>();

        assert_eq!(
            emit_rust_array("WORDS", &words).unwrap(),
            "pub(crate) const WORDS: &[&str] = &[\n    \"alpha\",\n    \"bravo\",\n];\n"
        );
    }

    #[test]
    fn samples_shape_edges_and_middle() {
        let first = ["alpha", "bravo", "charlie"]
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        let second = ["delta", "echo"]
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>();

        assert_eq!(
            sample_shape(&[first, second], 4).unwrap(),
            vec![
                "alpha delta".to_owned(),
                "alpha echo".to_owned(),
                "bravo delta".to_owned(),
                "bravo echo".to_owned()
            ]
        );
    }

    #[test]
    fn authored_lists_must_be_strictly_sorted() {
        let sorted = ["alpha", "bravo"]
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        let unsorted = ["bravo", "alpha"]
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>();

        assert_eq!(require_sorted("words", &sorted), Ok(()));
        assert_eq!(
            require_sorted("words", &unsorted),
            Err(ToolError::Mismatch { artifact: "words" })
        );
    }

    #[test]
    fn disjoint_lists_reject_shared_words() {
        let left = ["alpha", "bravo"]
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        let right = ["charlie", "delta"]
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        let overlapping = ["bravo", "charlie"]
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>();

        assert_eq!(require_disjoint("lists", &left, &right), Ok(()));
        assert_eq!(
            require_disjoint("lists", &left, &overlapping),
            Err(ToolError::Overlap {
                artifact: "lists",
                index: 0,
            })
        );
        let message = require_disjoint("lists", &left, &overlapping)
            .expect_err("shared word must fail")
            .to_string();
        assert!(message.contains("right-list index 0"));
        assert!(!message.contains("bravo"));
    }

    #[test]
    fn reproduces_existing_vectors_and_checks_authored_wordlists() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        check_existing_vectors(&root).unwrap();
    }
}
