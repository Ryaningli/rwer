use unicode_segmentation::UnicodeSegmentation;

use crate::alignment::{align, count_operations};
use crate::output::{AlignmentOutput, SplitKind, build_output};

#[allow(clippy::cast_precision_loss)]
#[inline]
fn to_f64(n: usize) -> f64 {
    n as f64
}

fn split_words(text: &str) -> Vec<&str> {
    text.split_whitespace().collect()
}

fn split_graphemes(text: &str) -> Vec<&str> {
    text.graphemes(true).collect()
}

/// Compute Word Error Rate between reference and hypothesis strings.
///
/// WER = (S + D + I) / N
///
/// Returns 0.0 if both are empty or if the reference is empty.
///
/// # Examples
///
/// ```
/// use rwer::wer;
///
/// assert!(wer("hello world", "hello world") < 1e-10);
/// assert!(wer("hello", "world") > 0.0);
/// ```
#[must_use]
pub fn wer(reference: &str, hypothesis: &str) -> f64 {
    let ref_words = split_words(reference);
    let hyp_words = split_words(hypothesis);
    compute_wer(&ref_words, &hyp_words)
}

/// Compute WER for multiple sentence pairs (flattened).
#[must_use]
pub fn wer_sentences(ref_sentences: &[&str], hyp_sentences: &[&str]) -> f64 {
    let all_ref: Vec<&str> = ref_sentences
        .iter()
        .flat_map(|s| s.split_whitespace())
        .collect();
    let all_hyp: Vec<&str> = hyp_sentences
        .iter()
        .flat_map(|s| s.split_whitespace())
        .collect();
    compute_wer(&all_ref, &all_hyp)
}

/// Internal WER computation from token sequences.
pub(crate) fn compute_wer<S: AsRef<str> + PartialEq>(reference: &[S], hypothesis: &[S]) -> f64 {
    let n = reference.len();
    if n == 0 {
        return 0.0;
    }
    let ops = align(reference, hypothesis);
    let counts = count_operations(&ops);
    let s_d_i = counts.substitutions + counts.deletions + counts.insertions;
    to_f64(s_d_i) / to_f64(n)
}

/// Compute Character Error Rate at the Unicode grapheme cluster level.
///
/// CER = (S + D + I) / N
///
/// # Examples
///
/// ```
/// use rwer::cer;
///
/// assert!(cer("hello", "hello") < 1e-10);
/// assert!(cer("abc", "axc") > 0.0);
/// ```
#[must_use]
pub fn cer(reference: &str, hypothesis: &str) -> f64 {
    let ref_chars = split_graphemes(reference);
    let hyp_chars = split_graphemes(hypothesis);
    compute_wer(&ref_chars, &hyp_chars)
}

/// Compute Match Error Rate.
///
/// MER = (S + D + I) / (H + S + D + I) where H = hits.
#[must_use]
pub fn mer(reference: &str, hypothesis: &str) -> f64 {
    let ref_words = split_words(reference);
    let hyp_words = split_words(hypothesis);
    compute_mer(&ref_words, &hyp_words)
}

fn compute_mer<S: AsRef<str> + PartialEq>(reference: &[S], hypothesis: &[S]) -> f64 {
    let ops = align(reference, hypothesis);
    let counts = count_operations(&ops);
    let total = counts.hits + counts.substitutions + counts.deletions + counts.insertions;
    if total == 0 {
        return 0.0;
    }
    let errors = counts.substitutions + counts.deletions + counts.insertions;
    to_f64(errors) / to_f64(total)
}

/// Compute Word Information Preserved.
///
/// WIP = (H / N) * (H / (H + S + D + I))
#[must_use]
pub fn wip(reference: &str, hypothesis: &str) -> f64 {
    let ref_words = split_words(reference);
    let hyp_words = split_words(hypothesis);
    compute_wip(&ref_words, &hyp_words)
}

fn compute_wip<S: AsRef<str> + PartialEq>(reference: &[S], hypothesis: &[S]) -> f64 {
    let n = reference.len();
    let h = hypothesis.len();
    if n == 0 && h == 0 {
        return 1.0;
    }
    if n == 0 || h == 0 {
        return 0.0;
    }
    let ops = align(reference, hypothesis);
    let counts = count_operations(&ops);
    let hits = counts.hits;
    if hits == 0 {
        return 0.0;
    }
    let recall = to_f64(hits) / to_f64(n);
    let precision =
        to_f64(hits) / to_f64(hits + counts.substitutions + counts.deletions + counts.insertions);
    recall * precision
}

/// Compute Word Information Lost.
///
/// WIL = 1 - WIP
#[must_use]
pub fn wil(reference: &str, hypothesis: &str) -> f64 {
    1.0 - wip(reference, hypothesis)
}

/// Compute all word-level metrics at once and return detailed output.
///
/// # Examples
///
/// ```
/// use rwer::process_words;
///
/// let output = process_words("the cat sat", "the cat sat on");
/// assert_eq!(output.hits, 3);
/// assert_eq!(output.substitutions, 0);
/// assert_eq!(output.insertions, 1);
/// ```
#[must_use]
pub fn process_words(reference: &str, hypothesis: &str) -> AlignmentOutput {
    let ref_words = split_words(reference);
    let hyp_words = split_words(hypothesis);
    let ops = align(&ref_words, &hyp_words);
    let counts = count_operations(&ops);
    build_output(&ref_words, &hyp_words, &ops, &counts, SplitKind::Words)
}

/// Compute all character-level metrics at once and return detailed output.
///
/// # Examples
///
/// ```
/// use rwer::process_chars;
///
/// let output = process_chars("abc", "axc");
/// assert!(output.cer > 0.0);
/// ```
#[must_use]
pub fn process_chars(reference: &str, hypothesis: &str) -> AlignmentOutput {
    let ref_chars = split_graphemes(reference);
    let hyp_chars = split_graphemes(hypothesis);
    let ops = align(&ref_chars, &hyp_chars);
    let counts = count_operations(&ops);
    build_output(&ref_chars, &hyp_chars, &ops, &counts, SplitKind::Graphemes)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Assert two f64 values are approximately equal.
    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-10
    }

    macro_rules! assert_approx_eq {
        ($left:expr, $right:expr) => {
            let left = $left;
            let right = $right;
            assert!(
                approx_eq(left, right),
                "assertion failed: {left:?} != {right:?}"
            );
        };
    }

    // === WER tests ===

    #[test]
    fn wer_perfect_match() {
        assert_approx_eq!(wer("hello world", "hello world"), 0.0);
    }

    #[test]
    fn wer_all_substituted() {
        let result = wer("hello world", "foo bar");
        assert!((result - 1.0).abs() < 1e-10);
    }

    #[test]
    fn wer_with_deletion() {
        // N=3, D=1 → WER = 1/3
        let result = wer("the cat sat", "the sat");
        assert!((result - 1.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn wer_with_insertion() {
        // N=2, I=1 → WER = 1/2
        let result = wer("the sat", "the cat sat");
        assert!((result - 0.5).abs() < 1e-10);
    }

    #[test]
    fn wer_empty_both() {
        assert_approx_eq!(wer("", ""), 0.0);
    }

    #[test]
    fn wer_empty_reference() {
        assert_approx_eq!(wer("", "hello world"), 0.0);
    }

    #[test]
    fn wer_empty_hypothesis() {
        let result = wer("hello world", "");
        assert!((result - 1.0).abs() < 1e-10);
    }

    #[test]
    fn wer_multiple_sentences() {
        // ref: "the cat sat the dog ran" (6 words)
        // hyp: "the cat sat the dog walked" (6 words)
        // N=6, S=1 → WER = 1/6
        let ref_sents = ["the cat sat", "the dog ran"];
        let hyp_sents = ["the cat sat", "the dog walked"];
        let result = wer_sentences(&ref_sents, &hyp_sents);
        assert!((result - 1.0 / 6.0).abs() < 1e-10);
    }

    #[test]
    fn wer_whitespace_agnostic() {
        assert_approx_eq!(wer("  hello  world  ", "hello world"), 0.0);
    }

    #[test]
    fn wer_single_word_match() {
        assert_approx_eq!(wer("hello", "hello"), 0.0);
    }

    #[test]
    fn wer_single_word_mismatch() {
        // N=1, S=1 → WER = 1/1 = 1.0
        assert!((wer("hello", "world") - 1.0).abs() < 1e-10);
    }

    // === CER tests ===

    #[test]
    fn cer_perfect_match() {
        assert_approx_eq!(cer("hello", "hello"), 0.0);
    }

    #[test]
    fn cer_with_substitution() {
        // N=5, S=1 → CER = 1/5
        let result = cer("abcde", "axcde");
        assert!((result - 0.2).abs() < 1e-10);
    }

    #[test]
    fn cer_empty_both() {
        assert_approx_eq!(cer("", ""), 0.0);
    }

    #[test]
    fn cer_empty_reference() {
        assert_approx_eq!(cer("", "hello"), 0.0);
    }

    #[test]
    fn cer_empty_hypothesis() {
        // N=3, D=3 → CER = 3/3 = 1.0
        let result = cer("abc", "");
        assert!((result - 1.0).abs() < 1e-10);
    }

    #[test]
    fn cer_with_unicode() {
        assert_approx_eq!(cer("hello 👋", "hello 👋"), 0.0);
    }

    #[test]
    fn cer_grapheme_cluster() {
        // Both are 1 grapheme, S=1 → CER = 1/1 = 1.0
        let result = cer("\u{00E9}", "e\u{0301}");
        assert!((result - 1.0).abs() < 1e-10);
    }

    #[test]
    fn cer_insertion() {
        let result = cer("ac", "abc");
        assert!((result - 1.0 / 2.0).abs() < 1e-10);
    }

    #[test]
    fn cer_deletion() {
        // N=3, D=1 → CER = 1/3
        let result = cer("abc", "ac");
        assert!((result - 1.0 / 3.0).abs() < 1e-10);
    }

    // === MER tests ===

    #[test]
    fn mer_perfect_match() {
        assert_approx_eq!(mer("hello world", "hello world"), 0.0);
    }

    #[test]
    fn mer_with_insertion() {
        let result = mer("a", "a b");
        assert!((result - 0.5).abs() < 1e-10);
    }

    #[test]
    fn mer_with_deletion() {
        let result = mer("a b", "a");
        assert!((result - 0.5).abs() < 1e-10);
    }

    #[test]
    fn mer_empty_both() {
        assert_approx_eq!(mer("", ""), 0.0);
    }

    // === WIP / WIL tests ===

    #[test]
    fn wip_perfect_match() {
        assert_approx_eq!(wip("hello world", "hello world"), 1.0);
    }

    #[test]
    fn wip_empty_both() {
        assert_approx_eq!(wip("", ""), 1.0);
    }

    #[test]
    fn wip_empty_reference() {
        assert_approx_eq!(wip("", "hello"), 0.0);
    }

    #[test]
    fn wip_empty_hypothesis() {
        assert_approx_eq!(wip("hello", ""), 0.0);
    }

    #[test]
    fn wip_no_match() {
        assert_approx_eq!(wip("hello", "world"), 0.0);
    }

    #[test]
    fn wil_perfect_match() {
        assert_approx_eq!(wil("hello world", "hello world"), 0.0);
    }

    #[test]
    fn wil_no_match() {
        assert_approx_eq!(wil("hello", "world"), 1.0);
    }

    #[test]
    fn wil_empty_both() {
        assert_approx_eq!(wil("", ""), 0.0);
    }

    // === process_words tests ===

    #[test]
    fn process_words_returns_output() {
        let output = process_words("the cat sat", "the cat sat on");
        // N=3, I=1 → WER = 1/3
        assert!((output.wer - 1.0 / 3.0).abs() < 1e-10);
        // MER = 1/(3+0+0+1) = 0.25
        assert!((output.mer - 0.25).abs() < 1e-10);
        assert!((output.wip - 0.75).abs() < 1e-10);
        assert!((output.wil - 0.25).abs() < 1e-10);
    }

    #[test]
    fn process_words_empty() {
        let output = process_words("", "");
        assert_approx_eq!(output.wer, 0.0);
        assert_eq!(output.hits, 0);
    }

    #[test]
    fn process_words_cer_zero_for_word_mode() {
        let output = process_words("hello", "world");
        assert_approx_eq!(output.cer, 0.0);
    }

    #[test]
    fn process_words_perfect() {
        let output = process_words("a b c", "a b c");
        assert_approx_eq!(output.wer, 0.0);
        assert_eq!(output.hits, 3);
    }

    // === process_chars tests ===

    #[test]
    fn process_chars_returns_output() {
        let output = process_chars("abcde", "axcde");
        // N=5, S=1 → CER = 1/5
        assert!((output.cer - 0.2).abs() < 1e-10);
        assert!((output.wer - 0.2).abs() < 1e-10);
    }

    #[test]
    fn process_chars_empty() {
        let output = process_chars("", "");
        assert_approx_eq!(output.cer, 0.0);
    }

    #[test]
    fn process_chars_perfect() {
        let output = process_chars("hello", "hello");
        assert_approx_eq!(output.cer, 0.0);
    }

    // === Internal compute_wer tests ===

    #[test]
    fn compute_wer_with_string_slices() {
        let ref_tokens: Vec<&str> = vec!["a", "b"];
        let hyp_tokens: Vec<&str> = vec!["a", "c"];
        // N=2, S=1 → WER = 1/2
        let result = compute_wer(&ref_tokens, &hyp_tokens);
        assert!((result - 0.5).abs() < 1e-10);
    }

    #[test]
    fn compute_wer_with_strings() {
        let ref_tokens = vec![String::from("a"), String::from("b")];
        let hyp_tokens = vec![String::from("a"), String::from("c")];
        let result = compute_wer(&ref_tokens, &hyp_tokens);
        assert!((result - 0.5).abs() < 1e-10);
    }

    #[test]
    fn compute_wer_empty_ref() {
        let ref_tokens: Vec<&str> = vec![];
        let hyp_tokens = vec!["a"];
        assert_approx_eq!(compute_wer(&ref_tokens, &hyp_tokens), 0.0);
    }

    #[test]
    fn wip_zero_hits_non_empty() {
        assert_approx_eq!(wip("a", "b"), 0.0);
    }

    #[test]
    fn mer_with_deletions_only() {
        let result = mer("a b", "a");
        assert!((result - 0.5).abs() < 1e-10);
    }

    #[test]
    fn mer_all_errors() {
        let result = mer("a b", "c d");
        assert!((result - 1.0).abs() < 1e-10);
    }

    #[test]
    fn process_words_with_substitution_and_deletion() {
        let output = process_words("a b c", "a c");
        assert_eq!(output.ref_len, 3);
        assert_eq!(output.hyp_len, 2);
        assert_eq!(output.hits, 2);
        assert_eq!(output.deletions, 1);
        assert!((output.wer - 1.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn process_words_with_insertion_only() {
        let output = process_words("a", "a b");
        assert_eq!(output.ref_len, 1);
        assert_eq!(output.hyp_len, 2);
        assert_eq!(output.hits, 1);
        assert_eq!(output.insertions, 1);
        assert!((output.wer - 1.0).abs() < 1e-10);
    }

    #[test]
    fn process_chars_with_all_operations() {
        let output = process_chars("abcd", "axd");
        assert_eq!(output.ref_len, 4);
        assert_eq!(output.hits, 2);
        assert_eq!(output.substitutions, 1);
        assert_eq!(output.deletions, 1);
        assert!((output.cer - 2.0 / 4.0).abs() < 1e-10);
    }

    #[test]
    fn process_chars_display_with_cer() {
        let output = process_chars("abc", "axc");
        let display = format!("{output}");
        assert!(display.contains("CER:"));
    }
}
