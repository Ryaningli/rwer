//! WebAssembly bindings for rwer.
//!
//! Exposes WER, CER, and related ASR evaluation metrics to JavaScript.

use wasm_bindgen::prelude::*;

/// Compute Word Error Rate between reference and hypothesis strings.
///
/// ```js
/// import { wer } from "rwer-wasm";
/// console.log(wer("the cat sat", "the cat sat on")); // 0.3333...
/// ```
#[wasm_bindgen]
pub fn wer(reference: &str, hypothesis: &str) -> f64 {
    rwer::wer(reference, hypothesis)
}

/// Compute Character Error Rate at the Unicode grapheme cluster level.
///
/// ```js
/// import { cer } from "rwer-wasm";
/// console.log(cer("hello", "hallo")); // 0.2
/// ```
#[wasm_bindgen]
pub fn cer(reference: &str, hypothesis: &str) -> f64 {
    rwer::cer(reference, hypothesis)
}

/// Compute Match Error Rate.
#[wasm_bindgen]
pub fn mer(reference: &str, hypothesis: &str) -> f64 {
    rwer::mer(reference, hypothesis)
}

/// Compute Word Information Preserved.
#[wasm_bindgen]
pub fn wip(reference: &str, hypothesis: &str) -> f64 {
    rwer::wip(reference, hypothesis)
}

/// Compute Word Information Lost.
#[wasm_bindgen]
pub fn wil(reference: &str, hypothesis: &str) -> f64 {
    rwer::wil(reference, hypothesis)
}

/// A single chunk in the alignment visualization.
#[wasm_bindgen]
#[derive(Clone)]
pub struct AlignmentChunk {
    kind: String,
    text: String,
    hypothesis: String,
}

#[wasm_bindgen]
impl AlignmentChunk {
    /// The type of this chunk: "equal", "substitute", "insert", or "delete".
    #[wasm_bindgen(getter)]
    pub fn kind(&self) -> String {
        self.kind.clone()
    }

    /// The reference text (for equal/substitute/delete chunks).
    #[wasm_bindgen(getter)]
    pub fn text(&self) -> String {
        self.text.clone()
    }

    /// The hypothesis text (for substitute/insert chunks).
    #[wasm_bindgen(getter)]
    pub fn hypothesis(&self) -> String {
        self.hypothesis.clone()
    }
}

/// Detailed alignment output with all metrics and chunks.
#[wasm_bindgen]
#[derive(Clone)]
pub struct AlignmentOutput {
    inner: rwer::AlignmentOutput,
    chunks: Vec<AlignmentChunk>,
}

#[wasm_bindgen]
impl AlignmentOutput {
    #[wasm_bindgen(getter)]
    pub fn wer(&self) -> f64 {
        self.inner.wer
    }

    #[wasm_bindgen(getter)]
    pub fn mer(&self) -> f64 {
        self.inner.mer
    }

    #[wasm_bindgen(getter)]
    pub fn wip(&self) -> f64 {
        self.inner.wip
    }

    #[wasm_bindgen(getter)]
    pub fn wil(&self) -> f64 {
        self.inner.wil
    }

    #[wasm_bindgen(getter)]
    pub fn cer(&self) -> f64 {
        self.inner.cer
    }

    #[wasm_bindgen(getter)]
    pub fn hits(&self) -> usize {
        self.inner.hits
    }

    #[wasm_bindgen(getter)]
    pub fn substitutions(&self) -> usize {
        self.inner.substitutions
    }

    #[wasm_bindgen(getter)]
    pub fn deletions(&self) -> usize {
        self.inner.deletions
    }

    #[wasm_bindgen(getter)]
    pub fn insertions(&self) -> usize {
        self.inner.insertions
    }

    #[wasm_bindgen(getter)]
    pub fn ref_len(&self) -> usize {
        self.inner.ref_len
    }

    #[wasm_bindgen(getter)]
    pub fn hyp_len(&self) -> usize {
        self.inner.hyp_len
    }

    /// Get all alignment chunks as a JS array.
    pub fn chunks(&self) -> Vec<AlignmentChunk> {
        self.chunks.clone()
    }

    /// Get a human-readable alignment visualization.
    pub fn visualize(&self) -> String {
        rwer::visualize_alignment(&self.inner)
    }
}

impl From<&rwer::AlignmentChunk> for AlignmentChunk {
    fn from(chunk: &rwer::AlignmentChunk) -> Self {
        match chunk {
            rwer::AlignmentChunk::Equal { text } => AlignmentChunk {
                kind: "equal".to_string(),
                text: text.clone(),
                hypothesis: String::new(),
            },
            rwer::AlignmentChunk::Substitute {
                reference,
                hypothesis,
            } => AlignmentChunk {
                kind: "substitute".to_string(),
                text: reference.clone(),
                hypothesis: hypothesis.clone(),
            },
            rwer::AlignmentChunk::Insert { hypothesis } => AlignmentChunk {
                kind: "insert".to_string(),
                text: String::new(),
                hypothesis: hypothesis.clone(),
            },
            rwer::AlignmentChunk::Delete { reference } => AlignmentChunk {
                kind: "delete".to_string(),
                text: reference.clone(),
                hypothesis: String::new(),
            },
        }
    }
}

impl From<rwer::AlignmentOutput> for AlignmentOutput {
    fn from(output: rwer::AlignmentOutput) -> Self {
        let chunks = output.chunks.iter().map(AlignmentChunk::from).collect();
        AlignmentOutput {
            inner: output,
            chunks,
        }
    }
}

/// Compute all word-level metrics and return detailed alignment output.
#[wasm_bindgen]
pub fn process_words(reference: &str, hypothesis: &str) -> AlignmentOutput {
    rwer::process_words(reference, hypothesis).into()
}

/// Compute all character-level metrics and return detailed alignment output.
#[wasm_bindgen]
pub fn process_chars(reference: &str, hypothesis: &str) -> AlignmentOutput {
    rwer::process_chars(reference, hypothesis).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-10
    }

    #[test]
    fn wer_perfect_match() {
        assert!(approx_eq(wer("hello world", "hello world"), 0.0));
    }

    #[test]
    fn wer_all_substituted() {
        assert!(approx_eq(wer("hello", "world"), 1.0));
    }

    #[test]
    fn wer_with_deletion() {
        let result = wer("the cat sat", "the sat");
        assert!((result - 1.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn cer_perfect_match() {
        assert!(approx_eq(cer("hello", "hello"), 0.0));
    }

    #[test]
    fn cer_with_substitution() {
        let result = cer("abcde", "axcde");
        assert!((result - 0.2).abs() < 1e-10);
    }

    #[test]
    fn cer_cjk() {
        let result = cer("你好世界", "你们世界");
        assert!((result - 0.25).abs() < 1e-10);
    }

    #[test]
    fn mer_basic() {
        let result = mer("a", "a b");
        assert!((result - 0.5).abs() < 1e-10);
    }

    #[test]
    fn wip_perfect() {
        assert!(approx_eq(wip("hello world", "hello world"), 1.0));
    }

    #[test]
    fn wil_perfect() {
        assert!(approx_eq(wil("hello world", "hello world"), 0.0));
    }

    #[test]
    fn process_words_output() {
        let output = process_words("the cat sat", "the cat sat on");
        assert!((output.wer() - 1.0 / 3.0).abs() < 1e-10);
        assert_eq!(output.hits(), 3);
        assert_eq!(output.insertions(), 1);
        assert_eq!(output.ref_len(), 3);
        assert_eq!(output.hyp_len(), 4);
    }

    #[test]
    fn process_words_chunks() {
        let output = process_words("hello world", "hello earth");
        let chunks = output.chunks();
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].kind(), "equal");
        assert_eq!(chunks[0].text(), "hello");
        assert_eq!(chunks[1].kind(), "substitute");
        assert_eq!(chunks[1].text(), "world");
        assert_eq!(chunks[1].hypothesis(), "earth");
    }

    #[test]
    fn process_chars_output() {
        let output = process_chars("abcde", "axcde");
        assert!((output.cer() - 0.2).abs() < 1e-10);
        assert_eq!(output.hits(), 4);
        assert_eq!(output.substitutions(), 1);
    }

    #[test]
    fn process_chars_cjk() {
        let output = process_chars("你好世界", "你们世纪");
        assert!((output.cer() - 0.5).abs() < 1e-10);
        assert_eq!(output.substitutions(), 2);
    }

    #[test]
    fn visualize_returns_string() {
        let output = process_words("hello world", "hello earth");
        let viz = output.visualize();
        assert!(viz.contains("REF:"));
        assert!(viz.contains("HYP:"));
    }

    #[test]
    fn chunk_insert_kind() {
        let output = process_words("hello", "hello world");
        let chunks = output.chunks();
        assert_eq!(chunks[1].kind(), "insert");
        assert_eq!(chunks[1].hypothesis(), "world");
    }

    #[test]
    fn chunk_delete_kind() {
        let output = process_words("hello world", "hello");
        let chunks = output.chunks();
        assert_eq!(chunks[1].kind(), "delete");
        assert_eq!(chunks[1].text(), "world");
    }
}
