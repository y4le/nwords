use alloc::string::String;

use nwords_core::Formatter;

/// Formatter that joins words with ASCII spaces.
#[derive(Debug, Clone, Copy, Default)]
pub struct AsciiSpace;

impl Formatter for AsciiSpace {
    fn join(&self, words: &[&str]) -> String {
        let mut phrase = String::new();
        for (index, word) in words.iter().enumerate() {
            if index > 0 {
                phrase.push(' ');
            }
            phrase.push_str(word);
        }
        phrase
    }
}

#[cfg(test)]
mod tests {
    use super::AsciiSpace;
    use nwords_core::Formatter;

    #[test]
    fn joins_with_ascii_spaces() {
        assert_eq!(AsciiSpace.join(&["alpha", "bravo"]), "alpha bravo");
    }
}
