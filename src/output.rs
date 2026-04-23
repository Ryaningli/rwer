use std::collections::HashMap;
use std::fmt;

use crate::alignment::{EditOp, OperationCounts};

#[allow(clippy::cast_precision_loss)]
#[inline]
fn to_f64(n: usize) -> f64 {
    n as f64
}

/// A single chunk in the alignment, representing one edit operation with its text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlignmentChunk {
    /// Matching text between reference and hypothesis.
    Equal {
        /// The matching text.
        text: String,
    },
    /// Reference text was substituted with hypothesis text.
    Substitute {
        /// The reference text.
        reference: String,
        /// The hypothesis text.
        hypothesis: String,
    },
    /// Text present in hypothesis but not in reference.
    Insert {
        /// The inserted hypothesis text.
        hypothesis: String,
    },
    /// Text present in reference but not in hypothesis.
    Delete {
        /// The deleted reference text.
        reference: String,
    },
}

/// Comprehensive output from computing alignment metrics.
#[derive(Debug, Clone)]
pub struct AlignmentOutput {
    /// Word Error Rate.
    pub wer: f64,
    /// Match Error Rate.
    pub mer: f64,
    /// Word Information Preserved.
    pub wip: f64,
    /// Word Information Lost.
    pub wil: f64,
    /// Character Error Rate (only populated by `process_chars`).
    pub cer: f64,
    /// Number of matching tokens.
    pub hits: usize,
    /// Number of substitutions.
    pub substitutions: usize,
    /// Number of deletions.
    pub deletions: usize,
    /// Number of insertions.
    pub insertions: usize,
    /// Reference length (in tokens).
    pub ref_len: usize,
    /// Hypothesis length (in tokens).
    pub hyp_len: usize,
    /// Alignment chunks with text.
    pub chunks: Vec<AlignmentChunk>,
}

/// Indicates how the input was split for alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SplitKind {
    /// Split by whitespace into words.
    Words,
    /// Split by Unicode grapheme clusters.
    Graphemes,
}

impl fmt::Display for AlignmentOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "WER:  {:.2}%", self.wer * 100.0)?;
        writeln!(f, "MER:  {:.2}%", self.mer * 100.0)?;
        writeln!(f, "WIP:  {:.4}", self.wip)?;
        writeln!(f, "WIL:  {:.4}", self.wil)?;
        if self.cer > 0.0 {
            writeln!(f, "CER:  {:.2}%", self.cer * 100.0)?;
        }
        writeln!(
            f,
            "Hits: {}  Sub: {}  Del: {}  Ins: {}",
            self.hits, self.substitutions, self.deletions, self.insertions
        )?;
        Ok(())
    }
}

/// Error frequency analysis from alignment.
#[derive(Debug, Clone, Default)]
pub struct ErrorCounts {
    /// Map of "ref → hyp" to count for substitutions.
    pub substitutions: HashMap<String, usize>,
    /// Map of inserted text to count.
    pub insertions: HashMap<String, usize>,
    /// Map of deleted text to count.
    pub deletions: HashMap<String, usize>,
}

/// Build an [`AlignmentOutput`] from alignment results.
pub(crate) fn build_output<S: AsRef<str>>(
    reference: &[S],
    hypothesis: &[S],
    ops: &[EditOp],
    counts: &OperationCounts,
    kind: SplitKind,
) -> AlignmentOutput {
    let ref_len = reference.len();
    let hyp_len = hypothesis.len();
    let s_d_i = counts.substitutions + counts.deletions + counts.insertions;
    let total = counts.hits + s_d_i;

    let wer = if ref_len == 0 {
        0.0
    } else {
        to_f64(s_d_i) / to_f64(ref_len)
    };
    let mer = if total == 0 {
        0.0
    } else {
        to_f64(s_d_i) / to_f64(total)
    };
    let wip = if ref_len == 0 && hyp_len == 0 {
        1.0
    } else if ref_len == 0 || hyp_len == 0 || counts.hits == 0 {
        0.0
    } else {
        let recall = to_f64(counts.hits) / to_f64(ref_len);
        let precision = to_f64(counts.hits) / to_f64(total);
        recall * precision
    };
    let wil = 1.0 - wip;

    let cer = match kind {
        SplitKind::Graphemes => {
            if ref_len == 0 {
                0.0
            } else {
                to_f64(s_d_i) / to_f64(ref_len)
            }
        }
        SplitKind::Words => 0.0,
    };

    let chunks = ops
        .iter()
        .map(|op| match op {
            EditOp::Equal { index } => AlignmentChunk::Equal {
                text: reference[*index].as_ref().to_string(),
            },
            EditOp::Substitute {
                ref_index,
                hyp_index,
            } => AlignmentChunk::Substitute {
                reference: reference[*ref_index].as_ref().to_string(),
                hypothesis: hypothesis[*hyp_index].as_ref().to_string(),
            },
            EditOp::Insert { hyp_index } => AlignmentChunk::Insert {
                hypothesis: hypothesis[*hyp_index].as_ref().to_string(),
            },
            EditOp::Delete { ref_index } => AlignmentChunk::Delete {
                reference: reference[*ref_index].as_ref().to_string(),
            },
        })
        .collect();

    AlignmentOutput {
        wer,
        mer,
        wip,
        wil,
        cer,
        hits: counts.hits,
        substitutions: counts.substitutions,
        deletions: counts.deletions,
        insertions: counts.insertions,
        ref_len,
        hyp_len,
        chunks,
    }
}

/// Generate a human-readable alignment visualization.
///
/// # Examples
///
/// ```
/// use rwer::process_words;
/// use rwer::visualize_alignment;
///
/// let output = process_words("the cat sat", "the dog sat");
/// println!("{}", visualize_alignment(&output));
/// ```
#[must_use]
pub fn visualize_alignment(output: &AlignmentOutput) -> String {
    use std::fmt::Write;

    let mut result = String::new();
    let mut ref_parts = Vec::new();
    let mut hyp_parts = Vec::new();

    for chunk in &output.chunks {
        match chunk {
            AlignmentChunk::Equal { text } => {
                ref_parts.push(text.clone());
                hyp_parts.push(text.clone());
            }
            AlignmentChunk::Substitute {
                reference,
                hypothesis,
            } => {
                ref_parts.push(reference.clone());
                hyp_parts.push(hypothesis.clone());
            }
            AlignmentChunk::Insert { hypothesis } => {
                ref_parts.push("*".to_string());
                hyp_parts.push(hypothesis.clone());
            }
            AlignmentChunk::Delete { reference } => {
                ref_parts.push(reference.clone());
                hyp_parts.push("*".to_string());
            }
        }
    }

    writeln!(result, "REF: {}", ref_parts.join(" ")).unwrap();
    writeln!(result, "HYP: {}", hyp_parts.join(" ")).unwrap();
    result
}

/// Collect error frequency counts from an alignment output.
///
/// # Examples
///
/// ```
/// use rwer::{collect_error_counts, process_words};
///
/// let output = process_words("the cat sat", "the dog sat");
/// let errors = collect_error_counts(&output);
/// assert!(errors.substitutions.contains_key(&"cat → dog".to_string()));
/// ```
#[must_use]
pub fn collect_error_counts(output: &AlignmentOutput) -> ErrorCounts {
    let mut counts = ErrorCounts::default();
    for chunk in &output.chunks {
        match chunk {
            AlignmentChunk::Substitute {
                reference,
                hypothesis,
            } => {
                *counts
                    .substitutions
                    .entry(format!("{reference} → {hypothesis}"))
                    .or_insert(0) += 1;
            }
            AlignmentChunk::Insert { hypothesis } => {
                *counts.insertions.entry(hypothesis.clone()).or_insert(0) += 1;
            }
            AlignmentChunk::Delete { reference } => {
                *counts.deletions.entry(reference.clone()).or_insert(0) += 1;
            }
            AlignmentChunk::Equal { .. } => {}
        }
    }
    counts
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_approx_eq {
        ($left:expr, $right:expr) => {
            assert!(
                (&$left - &$right).abs() < 1e-10,
                "assertion failed: {:?} != {:?}",
                $left,
                $right
            );
        };
    }

    fn sample_output() -> AlignmentOutput {
        AlignmentOutput {
            wer: 0.333,
            mer: 0.25,
            wip: 0.75,
            wil: 0.25,
            cer: 0.0,
            hits: 2,
            substitutions: 1,
            deletions: 0,
            insertions: 0,
            ref_len: 3,
            hyp_len: 3,
            chunks: vec![
                AlignmentChunk::Equal { text: "the".into() },
                AlignmentChunk::Equal { text: "cat".into() },
                AlignmentChunk::Substitute {
                    reference: "sat".into(),
                    hypothesis: "stood".into(),
                },
            ],
        }
    }

    #[test]
    fn alignment_output_display() {
        let output = sample_output();
        let display = format!("{output}");
        assert!(display.contains("WER:"));
        assert!(display.contains("33.30%"));
        assert!(display.contains("MER:"));
        assert!(display.contains("25.00%"));
        assert!(display.contains("WIP:"));
        assert!(display.contains("0.7500"));
        assert!(display.contains("Hits: 2"));
    }

    #[test]
    fn alignment_output_display_with_cer() {
        let mut output = sample_output();
        output.cer = 0.1;
        let display = format!("{output}");
        assert!(display.contains("CER:"));
    }

    #[test]
    fn alignment_output_display_no_cer() {
        let output = sample_output();
        let display = format!("{output}");
        assert!(!display.contains("CER:"));
    }

    #[test]
    fn visualize_alignment_basic() {
        let output = sample_output();
        let viz = visualize_alignment(&output);
        assert!(viz.contains("REF:"));
        assert!(viz.contains("HYP:"));
        assert!(viz.contains("the"));
        assert!(viz.contains("cat"));
    }

    #[test]
    fn visualize_alignment_with_insert() {
        let output = AlignmentOutput {
            wer: 0.5,
            mer: 0.333,
            wip: 0.5,
            wil: 0.5,
            cer: 0.0,
            hits: 1,
            substitutions: 0,
            deletions: 0,
            insertions: 1,
            ref_len: 1,
            hyp_len: 2,
            chunks: vec![
                AlignmentChunk::Equal { text: "a".into() },
                AlignmentChunk::Insert {
                    hypothesis: "b".into(),
                },
            ],
        };
        let viz = visualize_alignment(&output);
        assert!(viz.contains("REF: a *"));
        assert!(viz.contains("HYP: a b"));
    }

    #[test]
    fn visualize_alignment_with_delete() {
        let output = AlignmentOutput {
            wer: 1.0,
            mer: 1.0,
            wip: 0.0,
            wil: 1.0,
            cer: 0.0,
            hits: 0,
            substitutions: 0,
            deletions: 1,
            insertions: 0,
            ref_len: 1,
            hyp_len: 0,
            chunks: vec![AlignmentChunk::Delete {
                reference: "a".into(),
            }],
        };
        let viz = visualize_alignment(&output);
        assert!(viz.contains("REF: a"));
        assert!(viz.contains("HYP: *"));
    }

    #[test]
    fn visualize_empty() {
        let output = AlignmentOutput {
            wer: 0.0,
            mer: 0.0,
            wip: 1.0,
            wil: 0.0,
            cer: 0.0,
            hits: 0,
            substitutions: 0,
            deletions: 0,
            insertions: 0,
            ref_len: 0,
            hyp_len: 0,
            chunks: vec![],
        };
        let viz = visualize_alignment(&output);
        assert!(viz.contains("REF:"));
        assert!(viz.contains("HYP:"));
    }

    #[test]
    fn collect_error_counts_mixed() {
        let output = AlignmentOutput {
            wer: 0.5,
            mer: 0.5,
            wip: 0.5,
            wil: 0.5,
            cer: 0.0,
            hits: 1,
            substitutions: 1,
            deletions: 0,
            insertions: 1,
            ref_len: 2,
            hyp_len: 3,
            chunks: vec![
                AlignmentChunk::Equal { text: "a".into() },
                AlignmentChunk::Substitute {
                    reference: "b".into(),
                    hypothesis: "x".into(),
                },
                AlignmentChunk::Insert {
                    hypothesis: "y".into(),
                },
            ],
        };
        let errors = collect_error_counts(&output);
        assert_eq!(errors.substitutions.get("b → x").copied(), Some(1));
        assert_eq!(errors.insertions.get("y").copied(), Some(1));
        assert!(errors.deletions.is_empty());
    }

    #[test]
    fn collect_error_counts_empty() {
        let output = AlignmentOutput {
            wer: 0.0,
            mer: 0.0,
            wip: 1.0,
            wil: 0.0,
            cer: 0.0,
            hits: 0,
            substitutions: 0,
            deletions: 0,
            insertions: 0,
            ref_len: 0,
            hyp_len: 0,
            chunks: vec![],
        };
        let errors = collect_error_counts(&output);
        assert!(errors.substitutions.is_empty());
        assert!(errors.insertions.is_empty());
        assert!(errors.deletions.is_empty());
    }

    #[test]
    fn collect_error_counts_with_deletions() {
        let output = AlignmentOutput {
            wer: 0.5,
            mer: 0.5,
            wip: 0.5,
            wil: 0.5,
            cer: 0.0,
            hits: 0,
            substitutions: 0,
            deletions: 2,
            insertions: 0,
            ref_len: 2,
            hyp_len: 0,
            chunks: vec![
                AlignmentChunk::Delete {
                    reference: "a".into(),
                },
                AlignmentChunk::Delete {
                    reference: "b".into(),
                },
            ],
        };
        let errors = collect_error_counts(&output);
        assert_eq!(errors.deletions.get("a").copied(), Some(1));
        assert_eq!(errors.deletions.get("b").copied(), Some(1));
    }

    #[test]
    fn collect_error_counts_duplicate_errors() {
        let output = AlignmentOutput {
            wer: 0.0,
            mer: 1.0,
            wip: 0.0,
            wil: 1.0,
            cer: 0.0,
            hits: 0,
            substitutions: 2,
            deletions: 0,
            insertions: 0,
            ref_len: 2,
            hyp_len: 2,
            chunks: vec![
                AlignmentChunk::Substitute {
                    reference: "a".into(),
                    hypothesis: "x".into(),
                },
                AlignmentChunk::Substitute {
                    reference: "a".into(),
                    hypothesis: "x".into(),
                },
            ],
        };
        let errors = collect_error_counts(&output);
        assert_eq!(errors.substitutions.get("a → x").copied(), Some(2));
    }

    #[test]
    fn build_output_integration() {
        let ref_words: Vec<&str> = vec!["hello", "world"];
        let hyp_words: Vec<&str> = vec!["hello", "earth"];
        let ops = crate::alignment::align(&ref_words, &hyp_words);
        let counts = crate::alignment::count_operations(&ops);
        let output = build_output(&ref_words, &hyp_words, &ops, &counts, SplitKind::Words);
        assert_eq!(output.hits, 1);
        assert_eq!(output.substitutions, 1);
        // N=2, S=1 → WER = 1/2
        assert!((output.wer - 0.5).abs() < 1e-10);
        assert_eq!(output.ref_len, 2);
        assert_eq!(output.hyp_len, 2);
    }

    #[test]
    fn build_output_empty() {
        let ops: Vec<EditOp> = vec![];
        let counts = OperationCounts::default();
        let ref_words: Vec<&str> = vec![];
        let hyp_words: Vec<&str> = vec![];
        let output = build_output(&ref_words, &hyp_words, &ops, &counts, SplitKind::Words);
        assert_approx_eq!(output.wer, 0.0);
        assert_approx_eq!(output.mer, 0.0);
        assert!((output.wip - 1.0).abs() < 1e-10);
        assert!((output.wil).abs() < 1e-10);
        assert_eq!(output.hits, 0);
        assert_eq!(output.chunks.len(), 0);
    }

    #[test]
    fn build_output_grapheme_kind() {
        let ref_chars: Vec<&str> = vec!["a", "b", "c"];
        let hyp_chars: Vec<&str> = vec!["a", "x", "c"];
        let ops = crate::alignment::align(&ref_chars, &hyp_chars);
        let counts = crate::alignment::count_operations(&ops);
        let output = build_output(&ref_chars, &hyp_chars, &ops, &counts, SplitKind::Graphemes);
        // N=3, S=1 → CER = 1/3
        assert!((output.cer - 1.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn build_output_word_kind_no_cer() {
        let ref_words: Vec<&str> = vec!["a"];
        let hyp_words: Vec<&str> = vec!["b"];
        let ops = crate::alignment::align(&ref_words, &hyp_words);
        let counts = crate::alignment::count_operations(&ops);
        let output = build_output(&ref_words, &hyp_words, &ops, &counts, SplitKind::Words);
        assert_approx_eq!(output.cer, 0.0);
    }

    #[test]
    fn build_output_chunks() {
        let ref_words: Vec<&str> = vec!["a", "b"];
        let hyp_words: Vec<&str> = vec!["a"];
        let ops = crate::alignment::align(&ref_words, &hyp_words);
        let counts = crate::alignment::count_operations(&ops);
        let output = build_output(&ref_words, &hyp_words, &ops, &counts, SplitKind::Words);
        assert_eq!(output.chunks.len(), 2);
        assert_eq!(output.chunks[0], AlignmentChunk::Equal { text: "a".into() });
        assert_eq!(
            output.chunks[1],
            AlignmentChunk::Delete {
                reference: "b".into()
            }
        );
    }

    #[test]
    fn alignment_chunk_equality() {
        let c1 = AlignmentChunk::Equal { text: "a".into() };
        let c2 = AlignmentChunk::Equal { text: "a".into() };
        assert_eq!(c1, c2);
    }

    #[test]
    fn alignment_chunk_clone() {
        let c = AlignmentChunk::Substitute {
            reference: "a".into(),
            hypothesis: "b".into(),
        };
        let cloned = c.clone();
        assert_eq!(c, cloned);
    }

    #[test]
    fn error_counts_default() {
        let counts = ErrorCounts::default();
        assert!(counts.substitutions.is_empty());
        assert!(counts.insertions.is_empty());
        assert!(counts.deletions.is_empty());
    }

    #[test]
    fn build_output_non_empty_ref_empty_hyp() {
        let ref_words: Vec<&str> = vec!["a", "b"];
        let hyp_words: Vec<&str> = vec![];
        let ops = crate::alignment::align(&ref_words, &hyp_words);
        let counts = crate::alignment::count_operations(&ops);
        let output = build_output(&ref_words, &hyp_words, &ops, &counts, SplitKind::Words);
        assert_approx_eq!(output.wer, 1.0);
        assert_approx_eq!(output.mer, 1.0);
        assert_approx_eq!(output.wip, 0.0);
        assert_approx_eq!(output.wil, 1.0);
        assert_eq!(output.ref_len, 2);
        assert_eq!(output.hyp_len, 0);
    }

    #[test]
    fn build_output_empty_ref_non_empty_hyp() {
        let ref_words: Vec<&str> = vec![];
        let hyp_words: Vec<&str> = vec!["a"];
        let ops = crate::alignment::align(&ref_words, &hyp_words);
        let counts = crate::alignment::count_operations(&ops);
        let output = build_output(&ref_words, &hyp_words, &ops, &counts, SplitKind::Words);
        assert_approx_eq!(output.wer, 0.0);
        assert!(output.mer > 0.0);
        assert_approx_eq!(output.wip, 0.0);
        assert_approx_eq!(output.wil, 1.0);
        assert_eq!(output.ref_len, 0);
        assert_eq!(output.hyp_len, 1);
        assert_eq!(output.chunks.len(), 1);
        assert!(matches!(output.chunks[0], AlignmentChunk::Insert { .. }));
    }

    #[test]
    fn build_output_grapheme_with_cer_display() {
        let ref_chars: Vec<&str> = vec!["a", "b"];
        let hyp_chars: Vec<&str> = vec!["a", "x"];
        let ops = crate::alignment::align(&ref_chars, &hyp_chars);
        let counts = crate::alignment::count_operations(&ops);
        let output = build_output(&ref_chars, &hyp_chars, &ops, &counts, SplitKind::Graphemes);
        assert!((output.cer - 0.5).abs() < 1e-10);
        let display = format!("{output}");
        assert!(display.contains("CER:"));
    }

    #[test]
    fn build_output_perfect_match_all_fields() {
        let ref_words: Vec<&str> = vec!["a", "b", "c"];
        let hyp_words: Vec<&str> = vec!["a", "b", "c"];
        let ops = crate::alignment::align(&ref_words, &hyp_words);
        let counts = crate::alignment::count_operations(&ops);
        let output = build_output(&ref_words, &hyp_words, &ops, &counts, SplitKind::Words);
        assert_approx_eq!(output.wer, 0.0);
        assert_approx_eq!(output.mer, 0.0);
        assert_approx_eq!(output.wip, 1.0);
        assert_approx_eq!(output.wil, 0.0);
        assert_eq!(output.hits, 3);
        assert_eq!(output.substitutions, 0);
        assert_eq!(output.deletions, 0);
        assert_eq!(output.insertions, 0);
    }

    #[test]
    fn visualize_alignment_with_substitution() {
        let output = AlignmentOutput {
            wer: 0.5,
            mer: 0.5,
            wip: 0.5,
            wil: 0.5,
            cer: 0.0,
            hits: 1,
            substitutions: 1,
            deletions: 0,
            insertions: 0,
            ref_len: 2,
            hyp_len: 2,
            chunks: vec![
                AlignmentChunk::Equal { text: "a".into() },
                AlignmentChunk::Substitute {
                    reference: "b".into(),
                    hypothesis: "x".into(),
                },
            ],
        };
        let viz = visualize_alignment(&output);
        assert_eq!(viz.trim(), "REF: a b\nHYP: a x");
    }

    #[test]
    fn visualize_alignment_mixed_all_types() {
        let output = AlignmentOutput {
            wer: 0.6,
            mer: 0.5,
            wip: 0.4,
            wil: 0.6,
            cer: 0.0,
            hits: 2,
            substitutions: 1,
            deletions: 1,
            insertions: 1,
            ref_len: 4,
            hyp_len: 4,
            chunks: vec![
                AlignmentChunk::Equal { text: "a".into() },
                AlignmentChunk::Substitute {
                    reference: "b".into(),
                    hypothesis: "x".into(),
                },
                AlignmentChunk::Delete {
                    reference: "c".into(),
                },
                AlignmentChunk::Equal { text: "d".into() },
                AlignmentChunk::Insert {
                    hypothesis: "e".into(),
                },
            ],
        };
        let viz = visualize_alignment(&output);
        assert!(viz.contains("REF: a b c d *"));
        assert!(viz.contains("HYP: a x * d e"));
    }
}
