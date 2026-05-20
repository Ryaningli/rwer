//! Fast edit distance computation using rapidfuzz's Myers bit-parallel algorithm.
//!
//! Provides O([K/64]×M) Levenshtein distance for char-level comparison.
//! Use [`crate::alignment::align`] when you need S/D/I breakdown or alignment traceback.

/// Compute Levenshtein distance between two char sequences using rapidfuzz.
///
/// Uses Myers bit-parallel algorithm for O([K/64]×M) complexity where K is
/// the edit distance and M is the shorter sequence length.
///
/// # Examples
///
/// ```
/// use rwer::alignment_fast::rapidfuzz_char_distance;
///
/// let dist = rapidfuzz_char_distance("hello".chars(), "hallo".chars());
/// assert_eq!(dist, 1);
/// ```
#[must_use]
pub fn rapidfuzz_char_distance(
    s1: impl IntoIterator<Item = char>,
    s2: impl IntoIterator<Item = char>,
) -> usize {
    let v1: Vec<char> = s1.into_iter().collect();
    let v2: Vec<char> = s2.into_iter().collect();
    rapidfuzz::distance::levenshtein::distance(v1.iter().copied(), v2.iter().copied())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn char_distance_identical() {
        assert_eq!(rapidfuzz_char_distance("hello".chars(), "hello".chars()), 0);
    }

    #[test]
    fn char_distance_substitution() {
        assert_eq!(rapidfuzz_char_distance("hello".chars(), "hallo".chars()), 1);
    }

    #[test]
    fn char_distance_empty_both() {
        assert_eq!(rapidfuzz_char_distance("".chars(), "".chars()), 0);
    }

    #[test]
    fn char_distance_empty_one() {
        assert_eq!(rapidfuzz_char_distance("abc".chars(), "".chars()), 3);
        assert_eq!(rapidfuzz_char_distance("".chars(), "abc".chars()), 3);
    }

    #[test]
    fn char_distance_all_different() {
        assert_eq!(rapidfuzz_char_distance("abc".chars(), "xyz".chars()), 3);
    }

    #[test]
    fn char_distance_unicode() {
        assert_eq!(rapidfuzz_char_distance("你好".chars(), "你好".chars()), 0);
        assert_eq!(rapidfuzz_char_distance("你好".chars(), "你们".chars()), 1);
    }

    #[test]
    fn char_distance_insertion() {
        assert_eq!(rapidfuzz_char_distance("ac".chars(), "abc".chars()), 1);
    }

    #[test]
    fn char_distance_deletion() {
        assert_eq!(rapidfuzz_char_distance("abc".chars(), "ac".chars()), 1);
    }
}
