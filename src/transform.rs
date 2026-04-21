use std::collections::HashMap;
use unicode_segmentation::UnicodeSegmentation;

/// Trait for text transformations that can be chained in a pipeline.
pub trait Transform: Send + Sync {
    /// Apply the transformation to input text and return the result.
    fn transform(&self, input: &str) -> String;
}

/// Check if a char is a Unicode punctuation character.
/// Uses unicode-segmentation's word boundary logic: a char is punctuation
/// if it's not a letter, number, or whitespace and has zero word segments.
fn is_unicode_punctuation(c: char) -> bool {
    !c.is_alphanumeric() && !c.is_whitespace() && c.to_string().unicode_words().count() == 0
}

/// Chain multiple transforms into a pipeline.
///
/// # Examples
///
/// ```
/// use rwer::transform::{Compose, ToLower, Strip, Transform};
///
/// let pipeline = Compose::new(vec![
///     Box::new(Strip),
///     Box::new(ToLower),
/// ]);
/// assert_eq!(pipeline.transform("  Hello  "), "hello");
/// ```
pub struct Compose {
    transforms: Vec<Box<dyn Transform>>,
}

impl Compose {
    /// Create a new composition of transforms.
    #[must_use]
    pub fn new(transforms: Vec<Box<dyn Transform>>) -> Self {
        Self { transforms }
    }
}

impl Transform for Compose {
    fn transform(&self, input: &str) -> String {
        self.transforms
            .iter()
            .fold(input.to_string(), |text, t| t.transform(&text))
    }
}

/// Convert text to lowercase.
pub struct ToLower;

impl Transform for ToLower {
    fn transform(&self, input: &str) -> String {
        input.to_lowercase()
    }
}

/// Convert text to uppercase.
pub struct ToUpper;

impl Transform for ToUpper {
    fn transform(&self, input: &str) -> String {
        input.to_uppercase()
    }
}

/// Strip leading and trailing whitespace.
pub struct Strip;

impl Transform for Strip {
    fn transform(&self, input: &str) -> String {
        input.trim().to_string()
    }
}

/// Remove Unicode punctuation characters.
pub struct RemovePunctuation;

impl Transform for RemovePunctuation {
    fn transform(&self, input: &str) -> String {
        input
            .chars()
            .filter(|c| !is_unicode_punctuation(*c))
            .collect()
    }
}

/// Collapse multiple consecutive spaces into one.
pub struct RemoveMultipleSpaces;

impl Transform for RemoveMultipleSpaces {
    fn transform(&self, input: &str) -> String {
        let mut result = String::with_capacity(input.len());
        let mut prev_space = false;
        for c in input.chars() {
            if c == ' ' {
                if !prev_space {
                    result.push(c);
                    prev_space = true;
                }
            } else {
                result.push(c);
                prev_space = false;
            }
        }
        result
    }
}

/// Remove all whitespace characters.
pub struct RemoveWhitespace;

impl Transform for RemoveWhitespace {
    fn transform(&self, input: &str) -> String {
        input.chars().filter(|c| !c.is_whitespace()).collect()
    }
}

/// Replace whole words using a substitution map.
///
/// # Examples
///
/// ```
/// use rwer::transform::{SubstituteWords, Transform};
///
/// let t = SubstituteWords::new(vec![("hello", "hi")]);
/// assert_eq!(t.transform("hello world hello"), "hi world hi");
/// ```
pub struct SubstituteWords {
    substitutions: HashMap<String, String>,
}

impl SubstituteWords {
    /// Create a new substitution map.
    #[must_use]
    pub fn new(pairs: Vec<(&str, &str)>) -> Self {
        let substitutions = pairs
            .into_iter()
            .map(|(k, v)| (k.to_lowercase(), v.to_owned()))
            .collect();
        Self { substitutions }
    }
}

impl Transform for SubstituteWords {
    fn transform(&self, input: &str) -> String {
        input
            .split_whitespace()
            .map(|word| {
                self.substitutions
                    .get(&word.to_lowercase())
                    .cloned()
                    .unwrap_or_else(|| word.to_string())
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Remove specific words from text.
///
/// # Examples
///
/// ```
/// use rwer::transform::{RemoveSpecificWords, Transform};
///
/// let t = RemoveSpecificWords::new(&["the", "a"]);
/// assert_eq!(t.transform("the cat sat on a mat"), "cat sat on mat");
/// ```
pub struct RemoveSpecificWords {
    words: Vec<String>,
}

impl RemoveSpecificWords {
    /// Create a new filter that removes the specified words.
    #[must_use]
    pub fn new(words: &[&str]) -> Self {
        Self {
            words: words.iter().map(|w| w.to_lowercase()).collect(),
        }
    }
}

impl Transform for RemoveSpecificWords {
    fn transform(&self, input: &str) -> String {
        input
            .split_whitespace()
            .filter(|word| !self.words.contains(&word.to_lowercase()))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Expand common English contractions (e.g., "don't" → "do not").
pub struct ExpandCommonEnglishContractions;

impl Transform for ExpandCommonEnglishContractions {
    fn transform(&self, input: &str) -> String {
        let mut result = input.to_string();
        // Sorted by length descending to avoid partial matches (e.g., "wouldn't" before "n't")
        let contractions: &[(&str, &str)] = &[
            ("wouldn't", "would not"),
            ("couldn't", "could not"),
            ("shouldn't", "should not"),
            ("mustn't", "must not"),
            ("needn't", "need not"),
            ("hasn't", "has not"),
            ("haven't", "have not"),
            ("hadn't", "had not"),
            ("wasn't", "was not"),
            ("weren't", "were not"),
            ("isn't", "is not"),
            ("aren't", "are not"),
            ("don't", "do not"),
            ("doesn't", "does not"),
            ("didn't", "did not"),
            ("won't", "will not"),
            ("can't", "cannot"),
            ("shan't", "shall not"),
            ("they'll", "they will"),
            ("they'd", "they would"),
            ("they've", "they have"),
            ("they're", "they are"),
            ("that's", "that is"),
            ("there's", "there is"),
            ("here's", "here is"),
            ("what's", "what is"),
            ("let's", "let us"),
            ("you'll", "you will"),
            ("you'd", "you would"),
            ("you've", "you have"),
            ("you're", "you are"),
            ("we'll", "we will"),
            ("we'd", "we would"),
            ("we've", "we have"),
            ("we're", "we are"),
            ("i'll", "I will"),
            ("i'd", "I would"),
            ("i've", "I have"),
            ("i'm", "I am"),
            ("he'll", "he will"),
            ("he'd", "he would"),
            ("he's", "he is"),
            ("she'll", "she will"),
            ("she'd", "she would"),
            ("she's", "she is"),
            ("it's", "it is"),
        ];
        for (contraction, expansion) in contractions {
            result = result.replace(contraction, expansion);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_lower_transform() {
        let t = ToLower;
        assert_eq!(t.transform("Hello WORLD"), "hello world");
    }

    #[test]
    fn to_lower_empty() {
        let t = ToLower;
        assert_eq!(t.transform(""), "");
    }

    #[test]
    fn to_lower_unicode() {
        let t = ToLower;
        assert_eq!(t.transform("HELLO 您好"), "hello 您好");
    }

    #[test]
    fn to_upper_transform() {
        let t = ToUpper;
        assert_eq!(t.transform("hello"), "HELLO");
    }

    #[test]
    fn to_upper_empty() {
        let t = ToUpper;
        assert_eq!(t.transform(""), "");
    }

    #[test]
    fn strip_transform() {
        let t = Strip;
        assert_eq!(t.transform("  hello  "), "hello");
    }

    #[test]
    fn strip_no_whitespace() {
        let t = Strip;
        assert_eq!(t.transform("hello"), "hello");
    }

    #[test]
    fn strip_empty() {
        let t = Strip;
        assert_eq!(t.transform(""), "");
    }

    #[test]
    fn strip_only_whitespace() {
        let t = Strip;
        assert_eq!(t.transform("   "), "");
    }

    #[test]
    fn remove_punctuation_transform() {
        let t = RemovePunctuation;
        assert_eq!(t.transform("hello, world!"), "hello world");
    }

    #[test]
    fn remove_punctuation_no_punctuation() {
        let t = RemovePunctuation;
        assert_eq!(t.transform("hello world"), "hello world");
    }

    #[test]
    fn remove_punctuation_empty() {
        let t = RemovePunctuation;
        assert_eq!(t.transform(""), "");
    }

    #[test]
    fn remove_punctuation_unicode() {
        let t = RemovePunctuation;
        assert_eq!(t.transform("你好，世界！"), "你好世界");
    }

    #[test]
    fn remove_punctuation_only_punctuation() {
        let t = RemovePunctuation;
        assert_eq!(t.transform("!@#$%"), "");
    }

    #[test]
    fn remove_multiple_spaces_transform() {
        let t = RemoveMultipleSpaces;
        assert_eq!(t.transform("hello   world  foo"), "hello world foo");
    }

    #[test]
    fn remove_multiple_spaces_empty() {
        let t = RemoveMultipleSpaces;
        assert_eq!(t.transform(""), "");
    }

    #[test]
    fn remove_multiple_spaces_no_extra_spaces() {
        let t = RemoveMultipleSpaces;
        assert_eq!(t.transform("hello world"), "hello world");
    }

    #[test]
    fn remove_multiple_spaces_leading_trailing() {
        let t = RemoveMultipleSpaces;
        assert_eq!(t.transform("  hello  "), " hello ");
    }

    #[test]
    fn remove_multiple_spaces_only_spaces() {
        let t = RemoveMultipleSpaces;
        assert_eq!(t.transform("     "), " ");
    }

    #[test]
    fn remove_whitespace_transform() {
        let t = RemoveWhitespace;
        assert_eq!(t.transform("hello world"), "helloworld");
    }

    #[test]
    fn remove_whitespace_empty() {
        let t = RemoveWhitespace;
        assert_eq!(t.transform(""), "");
    }

    #[test]
    fn remove_whitespace_tabs_and_newlines() {
        let t = RemoveWhitespace;
        assert_eq!(t.transform("hello\tworld\n"), "helloworld");
    }

    #[test]
    fn substitute_words_transform() {
        let t = SubstituteWords::new(vec![("hello", "hi")]);
        assert_eq!(t.transform("hello world hello"), "hi world hi");
    }

    #[test]
    fn substitute_words_case_insensitive() {
        let t = SubstituteWords::new(vec![("hello", "hi")]);
        assert_eq!(t.transform("Hello WORLD"), "hi WORLD");
    }

    #[test]
    fn substitute_words_no_partial_match() {
        let t = SubstituteWords::new(vec![("he", "she")]);
        assert_eq!(t.transform("hello"), "hello");
    }

    #[test]
    fn substitute_words_empty() {
        let t = SubstituteWords::new(vec![]);
        assert_eq!(t.transform("hello world"), "hello world");
    }

    #[test]
    fn substitute_words_not_found() {
        let t = SubstituteWords::new(vec![("foo", "bar")]);
        assert_eq!(t.transform("hello world"), "hello world");
    }

    #[test]
    fn remove_specific_words_transform() {
        let t = RemoveSpecificWords::new(&["the", "a", "an"]);
        assert_eq!(t.transform("the cat sat on a mat"), "cat sat on mat");
    }

    #[test]
    fn remove_specific_words_case_insensitive() {
        let t = RemoveSpecificWords::new(&["the"]);
        assert_eq!(t.transform("The cat"), "cat");
    }

    #[test]
    fn remove_specific_words_empty() {
        let t = RemoveSpecificWords::new(&[]);
        assert_eq!(t.transform("hello world"), "hello world");
    }

    #[test]
    fn remove_specific_words_all_removed() {
        let t = RemoveSpecificWords::new(&["hello"]);
        assert_eq!(t.transform("hello"), "");
    }

    #[test]
    fn expand_contractions_dont() {
        let t = ExpandCommonEnglishContractions;
        assert_eq!(t.transform("don't"), "do not");
    }

    #[test]
    fn expand_contractions_cant() {
        let t = ExpandCommonEnglishContractions;
        assert_eq!(t.transform("can't"), "cannot");
    }

    #[test]
    fn expand_contractions_its() {
        let t = ExpandCommonEnglishContractions;
        assert_eq!(t.transform("it's"), "it is");
    }

    #[test]
    fn expand_contractions_wont() {
        let t = ExpandCommonEnglishContractions;
        assert_eq!(t.transform("won't"), "will not");
    }

    #[test]
    fn expand_contractions_im() {
        let t = ExpandCommonEnglishContractions;
        assert_eq!(t.transform("i'm"), "I am");
    }

    #[test]
    fn expand_contractions_im_uppercase() {
        let t = ExpandCommonEnglishContractions;
        // Uppercase "I'm" has no matching contraction (table is lowercase)
        assert_eq!(t.transform("I'm"), "I'm");
    }

    #[test]
    fn expand_contractions_multiple() {
        let t = ExpandCommonEnglishContractions;
        // "i'm" → "I am", "can't" → "cannot"
        assert_eq!(t.transform("i can't do it"), "i cannot do it");
    }

    #[test]
    fn expand_contractions_with_im() {
        let t = ExpandCommonEnglishContractions;
        assert_eq!(t.transform("i'm here"), "I am here");
    }

    #[test]
    fn expand_contractions_no_contraction() {
        let t = ExpandCommonEnglishContractions;
        assert_eq!(t.transform("hello world"), "hello world");
    }

    #[test]
    fn expand_contractions_empty() {
        let t = ExpandCommonEnglishContractions;
        assert_eq!(t.transform(""), "");
    }

    #[test]
    fn expand_contractions_preserves_case() {
        let t = ExpandCommonEnglishContractions;
        // Uppercase input has no matching contraction (table is lowercase)
        assert_eq!(t.transform("DON'T"), "DON'T");
    }

    #[test]
    fn compose_chaining() {
        let pipeline = Compose::new(vec![
            Box::new(Strip),
            Box::new(ToLower),
            Box::new(RemovePunctuation),
            Box::new(RemoveMultipleSpaces),
        ]);
        assert_eq!(pipeline.transform("  Hello, World!  "), "hello world");
    }

    #[test]
    fn compose_empty() {
        let pipeline = Compose::new(vec![]);
        assert_eq!(pipeline.transform("hello"), "hello");
    }

    #[test]
    fn compose_single() {
        let pipeline = Compose::new(vec![Box::new(ToLower)]);
        assert_eq!(pipeline.transform("HELLO"), "hello");
    }

    #[test]
    fn compose_empty_input() {
        let pipeline = Compose::new(vec![Box::new(ToLower), Box::new(Strip)]);
        assert_eq!(pipeline.transform(""), "");
    }
}
