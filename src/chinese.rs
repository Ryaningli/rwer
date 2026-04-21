//! Chinese word segmentation using jieba-rs for word-level WER.
//!
//! This module provides a Chinese tokenizer that segments text into words
//! so that word-level WER can be computed for Chinese text.
//!
//! Note: Character-level metrics (CER) work with Chinese text out of the box
//! without enabling any feature flag — this module is only needed for word-level WER.
//!
//! This module is only available when the `chinese-word` feature is enabled.
//!
//! # Example
//!
//! ```
//! use rwer::ChineseTokenizer;
//!
//! let tokenizer = ChineseTokenizer::new();
//! let words = tokenizer.cut("我们中出了一个叛徒");
//! println!("{:?}", words);
//! ```

use jieba_rs::Jieba;

/// Chinese word tokenizer backed by jieba-rs.
pub struct ChineseTokenizer {
    jieba: Jieba,
}

impl ChineseTokenizer {
    /// Create a new Chinese tokenizer with the default dictionary.
    #[must_use]
    pub fn new() -> Self {
        Self {
            jieba: Jieba::new(),
        }
    }

    /// Segment Chinese text into words.
    ///
    /// Returns a vector of word strings.
    pub fn cut(&self, text: &str) -> Vec<String> {
        self.jieba
            .cut(text, false)
            .into_iter()
            .map(String::from)
            .collect()
    }

    /// Segment Chinese text into words for search mode (handles unknown words better).
    pub fn cut_for_search(&self, text: &str) -> Vec<String> {
        self.jieba
            .cut_for_search(text, false)
            .into_iter()
            .map(String::from)
            .collect()
    }
}

impl Default for ChineseTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute word-level WER for Chinese text using jieba segmentation.
///
/// For character-level evaluation, use [`crate::cer`] instead — no feature flag needed.
///
/// # Example
///
/// ```
/// use rwer::chinese_wer;
///
/// let result = chinese_wer("今天天气真好", "今天天气很棒");
/// println!("Chinese WER: {:.2}%", result * 100.0);
/// ```
#[must_use]
pub fn chinese_wer(reference: &str, hypothesis: &str) -> f64 {
    if reference.is_empty() && hypothesis.is_empty() {
        return 0.0;
    }
    let tokenizer = ChineseTokenizer::new();
    let ref_words = tokenizer.cut(reference);
    let ref_strs: Vec<&str> = ref_words.iter().map(String::as_str).collect();
    let hyp_words = tokenizer.cut(hypothesis);
    let hyp_strs: Vec<&str> = hyp_words.iter().map(String::as_str).collect();
    crate::metrics::compute_wer(&ref_strs, &hyp_strs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chinese_tokenizer_basic() {
        let tokenizer = ChineseTokenizer::new();
        let words = tokenizer.cut("我们中出了一个叛徒");
        assert!(!words.is_empty());
        assert!(words.contains(&String::from("我们")));
    }

    #[test]
    fn chinese_tokenizer_empty() {
        let tokenizer = ChineseTokenizer::new();
        let words = tokenizer.cut("");
        assert!(words.is_empty());
    }

    #[test]
    fn chinese_tokenizer_mixed() {
        let tokenizer = ChineseTokenizer::new();
        let words = tokenizer.cut("我喜欢rust编程");
        assert!(!words.is_empty());
    }

    #[test]
    fn chinese_tokenizer_for_search() {
        let tokenizer = ChineseTokenizer::new();
        let words = tokenizer.cut_for_search("南京市长江大桥");
        assert!(!words.is_empty());
    }

    #[test]
    fn chinese_tokenizer_default() {
        let tokenizer = ChineseTokenizer::default();
        assert!(tokenizer.cut("你好").contains(&String::from("你好")));
    }

    #[test]
    fn chinese_wer_perfect() {
        let result = chinese_wer("今天天气真好", "今天天气真好");
        assert!((result - 0.0).abs() < 1e-10);
    }

    #[test]
    fn chinese_wer_with_errors() {
        let result = chinese_wer("今天天气真好", "今天天气很棒");
        assert!((0.0..=1.0).contains(&result));
    }

    #[test]
    fn chinese_wer_empty() {
        assert!((chinese_wer("", "") - 0.0).abs() < 1e-10);
    }

    #[test]
    fn chinese_wer_empty_ref() {
        assert!((chinese_wer("", "你好") - 0.0).abs() < 1e-10);
    }

    #[test]
    fn chinese_wer_empty_hyp() {
        let result = chinese_wer("你好世界", "");
        assert!((0.0..=1.0).contains(&result));
    }

    #[test]
    fn chinese_wer_longer_text() {
        let ref_text = "自然语言处理是人工智能领域的一个重要方向";
        let hyp_text = "自然语言处理是人工智能领域的一个核心方向";
        let result = chinese_wer(ref_text, hyp_text);
        assert!((0.0..=1.0).contains(&result));
    }
}
