use crate::WordMap;

/// Linear word map over an index-ordered word slice.
#[derive(Debug, Clone, Copy)]
pub struct Linear<'a> {
    words: &'a [&'a str],
}

impl<'a> Linear<'a> {
    /// Creates a linear word map from an index-ordered word slice.
    pub const fn new(words: &'a [&'a str]) -> Self {
        Self { words }
    }

    /// Returns the backing word slice.
    pub const fn words(&self) -> &'a [&'a str] {
        self.words
    }
}

impl WordMap for Linear<'_> {
    fn len(&self, _position: usize) -> usize {
        self.words.len()
    }

    fn word(&self, index: usize, _position: usize) -> Option<&str> {
        self.words.get(index).copied()
    }

    fn index_of(&self, word: &str, _position: usize) -> Option<usize> {
        self.words.iter().position(|candidate| *candidate == word)
    }
}

/// Binary-search word map over an index-ordered, lexicographically sorted slice.
#[derive(Debug, Clone, Copy)]
pub struct Sorted<'a> {
    words: &'a [&'a str],
}

impl<'a> Sorted<'a> {
    /// Creates a sorted word map from an index-ordered, sorted word slice.
    pub const fn new(words: &'a [&'a str]) -> Self {
        Self { words }
    }

    /// Returns the backing word slice.
    pub const fn words(&self) -> &'a [&'a str] {
        self.words
    }
}

impl WordMap for Sorted<'_> {
    fn len(&self, _position: usize) -> usize {
        self.words.len()
    }

    fn word(&self, index: usize, _position: usize) -> Option<&str> {
        self.words.get(index).copied()
    }

    fn index_of(&self, word: &str, _position: usize) -> Option<usize> {
        self.words
            .binary_search_by(|candidate| candidate.cmp(&word))
            .ok()
    }
}

#[cfg(test)]
mod tests {
    use super::{Linear, Sorted};
    use crate::WordMap;

    const WORDS: &[&str] = &["alpha", "bravo", "charlie", "delta"];

    #[test]
    fn linear_lookup_round_trips() {
        let map = Linear::new(WORDS);

        assert_eq!(map.len(0), 4);
        assert_eq!(map.word(2, 0), Some("charlie"));
        assert_eq!(map.index_of("delta", 0), Some(3));
        assert_eq!(map.index_of("echo", 0), None);
    }

    #[test]
    fn sorted_lookup_matches_linear_on_sorted_words() {
        let linear = Linear::new(WORDS);
        let sorted = Sorted::new(WORDS);

        for position in 0..3 {
            for (index, word) in WORDS.iter().enumerate() {
                assert_eq!(sorted.word(index, position), linear.word(index, position));
                assert_eq!(
                    sorted.index_of(word, position),
                    linear.index_of(word, position)
                );
            }
        }
    }
}
