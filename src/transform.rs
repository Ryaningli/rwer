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

/// Convert Traditional Chinese text to Simplified Chinese.
///
/// Uses the `zhconv` crate with `OpenCC` rules for high-accuracy conversion.
///
/// # Examples
///
/// ```
/// use rwer::transform::{ToSimplified, Transform};
///
/// let t = ToSimplified;
/// assert_eq!(t.transform("繁體中文"), "繁体中文");
/// ```
#[cfg(feature = "chinese-variant")]
pub struct ToSimplified;

#[cfg(feature = "chinese-variant")]
impl Transform for ToSimplified {
    fn transform(&self, input: &str) -> String {
        zhconv::zhconv(input, zhconv::Variant::ZhCN)
    }
}

/// Convert Simplified Chinese text to Traditional Chinese.
///
/// Uses the `zhconv` crate with `OpenCC` rules for high-accuracy conversion.
///
/// # Examples
///
/// ```
/// use rwer::transform::{ToTraditional, Transform};
///
/// let t = ToTraditional;
/// assert_eq!(t.transform("简体中文"), "簡體中文");
/// ```
#[cfg(feature = "chinese-variant")]
pub struct ToTraditional;

#[cfg(feature = "chinese-variant")]
impl Transform for ToTraditional {
    fn transform(&self, input: &str) -> String {
        zhconv::zhconv(input, zhconv::Variant::ZhHant)
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
    fn expand_contractions_wouldnt() {
        let t = ExpandCommonEnglishContractions;
        assert_eq!(t.transform("wouldn't"), "would not");
    }

    #[test]
    fn expand_contractions_couldnt() {
        let t = ExpandCommonEnglishContractions;
        assert_eq!(t.transform("couldn't"), "could not");
    }

    #[test]
    fn expand_contractions_shouldnt() {
        let t = ExpandCommonEnglishContractions;
        assert_eq!(t.transform("shouldn't"), "should not");
    }

    #[test]
    fn expand_contractions_havent() {
        let t = ExpandCommonEnglishContractions;
        assert_eq!(t.transform("haven't"), "have not");
    }

    #[test]
    fn expand_contractions_theyll() {
        let t = ExpandCommonEnglishContractions;
        assert_eq!(t.transform("they'll go"), "they will go");
    }

    #[test]
    fn expand_contractions_theyre() {
        let t = ExpandCommonEnglishContractions;
        assert_eq!(t.transform("they're here"), "they are here");
    }

    #[test]
    fn expand_contractions_theyve() {
        let t = ExpandCommonEnglishContractions;
        assert_eq!(t.transform("they've done it"), "they have done it");
    }

    #[test]
    fn expand_contractions_thats() {
        let t = ExpandCommonEnglishContractions;
        assert_eq!(t.transform("that's great"), "that is great");
    }

    #[test]
    fn expand_contractions_shant() {
        let t = ExpandCommonEnglishContractions;
        assert_eq!(t.transform("shan't"), "shall not");
    }

    #[test]
    fn expand_contractions_mustnt() {
        let t = ExpandCommonEnglishContractions;
        assert_eq!(t.transform("mustn't"), "must not");
    }

    #[test]
    fn expand_contractions_neednt() {
        let t = ExpandCommonEnglishContractions;
        assert_eq!(t.transform("needn't"), "need not");
    }

    #[test]
    fn expand_contractions_hes() {
        let t = ExpandCommonEnglishContractions;
        assert_eq!(t.transform("he's here"), "he is here");
    }

    #[test]
    fn expand_contractions_shes() {
        let t = ExpandCommonEnglishContractions;
        assert_eq!(t.transform("she's here"), "she is here");
    }

    #[test]
    fn expand_contractions_wed() {
        let t = ExpandCommonEnglishContractions;
        assert_eq!(t.transform("we'd go"), "we would go");
    }

    #[test]
    fn expand_contractions_youre() {
        let t = ExpandCommonEnglishContractions;
        assert_eq!(t.transform("you're right"), "you are right");
    }

    #[test]
    fn expand_contractions_ive() {
        let t = ExpandCommonEnglishContractions;
        assert_eq!(t.transform("i've seen it"), "I have seen it");
    }

    #[test]
    fn expand_contractions_id() {
        let t = ExpandCommonEnglishContractions;
        assert_eq!(t.transform("i'd go"), "I would go");
    }

    #[test]
    fn expand_contractions_ill() {
        let t = ExpandCommonEnglishContractions;
        assert_eq!(t.transform("i'll do it"), "I will do it");
    }

    #[test]
    fn expand_contractions_doesnt() {
        let t = ExpandCommonEnglishContractions;
        assert_eq!(t.transform("doesn't"), "does not");
    }

    #[test]
    fn expand_contractions_didnt() {
        let t = ExpandCommonEnglishContractions;
        assert_eq!(t.transform("didn't"), "did not");
    }

    #[test]
    fn expand_contractions_isnt() {
        let t = ExpandCommonEnglishContractions;
        assert_eq!(t.transform("isn't"), "is not");
    }

    #[test]
    fn expand_contractions_arent() {
        let t = ExpandCommonEnglishContractions;
        assert_eq!(t.transform("aren't"), "are not");
    }

    #[test]
    fn expand_contractions_wasnt() {
        let t = ExpandCommonEnglishContractions;
        assert_eq!(t.transform("wasn't"), "was not");
    }

    #[test]
    fn expand_contractions_werent() {
        let t = ExpandCommonEnglishContractions;
        assert_eq!(t.transform("weren't"), "were not");
    }

    #[test]
    fn expand_contractions_hadnt() {
        let t = ExpandCommonEnglishContractions;
        assert_eq!(t.transform("hadn't"), "had not");
    }

    #[test]
    fn expand_contractions_herell() {
        let t = ExpandCommonEnglishContractions;
        assert_eq!(t.transform("here's"), "here is");
    }

    #[test]
    fn expand_contractions_theres() {
        let t = ExpandCommonEnglishContractions;
        assert_eq!(t.transform("there's"), "there is");
    }

    #[test]
    fn expand_contractions_whats() {
        let t = ExpandCommonEnglishContractions;
        assert_eq!(t.transform("what's"), "what is");
    }

    #[test]
    fn expand_contractions_lets() {
        let t = ExpandCommonEnglishContractions;
        assert_eq!(t.transform("let's go"), "let us go");
    }

    #[test]
    fn expand_contractions_youll() {
        let t = ExpandCommonEnglishContractions;
        assert_eq!(t.transform("you'll see"), "you will see");
    }

    #[test]
    fn expand_contractions_youd() {
        let t = ExpandCommonEnglishContractions;
        assert_eq!(t.transform("you'd like"), "you would like");
    }

    #[test]
    fn expand_contractions_youve() {
        let t = ExpandCommonEnglishContractions;
        assert_eq!(t.transform("you've got"), "you have got");
    }

    #[test]
    fn expand_contractions_shell() {
        let t = ExpandCommonEnglishContractions;
        assert_eq!(t.transform("she'll go"), "she will go");
    }

    #[test]
    fn expand_contractions_shed() {
        let t = ExpandCommonEnglishContractions;
        assert_eq!(t.transform("she'd go"), "she would go");
    }

    #[test]
    fn expand_contractions_well() {
        let t = ExpandCommonEnglishContractions;
        assert_eq!(t.transform("we'll go"), "we will go");
    }

    #[test]
    fn expand_contractions_weve() {
        let t = ExpandCommonEnglishContractions;
        assert_eq!(t.transform("we've been"), "we have been");
    }

    #[test]
    fn expand_contractions_were() {
        let t = ExpandCommonEnglishContractions;
        assert_eq!(t.transform("we're here"), "we are here");
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

    #[cfg(feature = "chinese-variant")]
    mod chinese_variant_tests {
        use super::*;

        #[test]
        fn to_simplified_traditional_to_simplified() {
            let t = ToSimplified;
            assert_eq!(t.transform("繁體中文"), "繁体中文");
        }

        #[test]
        fn to_simplified_mixed_text() {
            let t = ToSimplified;
            assert_eq!(t.transform("這是個測試"), "这是个测试");
        }

        #[test]
        fn to_simplified_already_simplified() {
            let t = ToSimplified;
            assert_eq!(t.transform("简体中文"), "简体中文");
        }

        #[test]
        fn to_simplified_empty() {
            let t = ToSimplified;
            assert_eq!(t.transform(""), "");
        }

        #[test]
        fn to_simplified_with_punctuation() {
            let t = ToSimplified;
            assert_eq!(t.transform("你好，世界！"), "你好，世界！");
        }

        #[test]
        fn to_traditional_simplified_to_traditional() {
            let t = ToTraditional;
            assert_eq!(t.transform("简体中文"), "簡體中文");
        }

        #[test]
        fn to_traditional_already_traditional() {
            let t = ToTraditional;
            assert_eq!(t.transform("繁體中文"), "繁體中文");
        }

        #[test]
        fn to_traditional_empty() {
            let t = ToTraditional;
            assert_eq!(t.transform(""), "");
        }

        #[test]
        fn to_traditional_with_punctuation() {
            let t = ToTraditional;
            assert_eq!(t.transform("你好，世界！"), "你好，世界！");
        }

        #[test]
        fn roundtrip_simplified_traditional_simplified() {
            let original = "简体中文测试";
            let t = ToTraditional;
            let traditional = t.transform(original);
            let s = ToSimplified;
            let back = s.transform(&traditional);
            assert_eq!(back, original);
        }

        #[test]
        fn compose_with_to_simplified() {
            let pipeline = Compose::new(vec![Box::new(ToSimplified), Box::new(ToLower)]);
            assert_eq!(pipeline.transform("繁體中文"), "繁体中文");
        }
    }
}
