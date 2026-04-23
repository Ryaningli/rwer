//! # rwer
//!
//! A modern Rust crate for Word Error Rate (WER), Character Error Rate (CER),
//! and related ASR (Automatic Speech Recognition) evaluation metrics.
//!
//! ## Quick Start
//!
//! ```
//! use rwer::{cer, wer};
//!
//! let reference = "the cat sat on the mat";
//! let hypothesis = "the cat sat on a mat";
//!
//! println!("WER: {:.2}%", wer(reference, hypothesis) * 100.0);
//! println!("CER: {:.2}%", cer(reference, hypothesis) * 100.0);
//! ```
//!
//! ## Metrics
//!
//! - **WER** (Word Error Rate): `(S + D + I) / N`
//! - **CER** (Character Error Rate): Same formula at Unicode grapheme cluster level
//! - **MER** (Match Error Rate): `(S + D + I) / (H + S + D + I)`
//! - **WIP** (Word Information Preserved): `(H/N) * (H/(H+S+D+I))`
//! - **WIL** (Word Information Lost): `1 - WIP`
//!
//! ## Transforms
//!
//! ```
//! use rwer::{wer, Compose, ToLower, RemovePunctuation, Transform};
//!
//! let pipeline = Compose::new(vec![
//!     Box::new(ToLower),
//!     Box::new(RemovePunctuation),
//! ]);
//!
//! let ref_text = pipeline.transform("Hello, World!");
//! let hyp_text = pipeline.transform("hello world");
//! assert!(wer(&ref_text, &hyp_text) < 1e-10);
//! ```
//!
//! ## Feature Flags
//!
//! - `chinese-word` — Enable Chinese word segmentation using jieba-rs
//! - `chinese-variant` — Enable Traditional/Simplified Chinese conversion using zhconv
//! - `cli` — Enable the CLI binary

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]

/// Levenshtein alignment and edit operations.
pub mod alignment;

/// WER, CER, MER, WIP, WIL metric computation.
pub mod metrics;

/// Alignment output, visualization, and error analysis.
pub mod output;

/// Text transform pipeline for preprocessing.
pub mod transform;

/// Prelude module with commonly used items.
pub mod prelude {
    pub use crate::metrics::{cer, mer, wer, wil, wip};
    pub use crate::output::{AlignmentOutput, ErrorCounts, visualize_alignment};
    pub use crate::transform::{Compose, ToLower, Transform};

    #[cfg(feature = "chinese-variant")]
    pub use crate::transform::{ToSimplified, ToTraditional};
}

// Re-export main API items
pub use crate::alignment::{EditOp, OperationCounts, align, count_operations};
pub use crate::metrics::{cer, mer, process_chars, process_words, wer, wer_sentences, wil, wip};
pub use crate::output::{
    AlignmentChunk, AlignmentOutput, ErrorCounts, collect_error_counts, visualize_alignment,
};
pub use crate::transform::{
    Compose, ExpandCommonEnglishContractions, RemoveMultipleSpaces, RemovePunctuation,
    RemoveSpecificWords, RemoveWhitespace, Strip, SubstituteWords, ToLower, ToUpper, Transform,
};

#[cfg(feature = "chinese-word")]
pub use crate::transform::ChineseWordSegment;

#[cfg(feature = "chinese-variant")]
pub use crate::transform::{ToSimplified, ToTraditional};

#[cfg(feature = "cli")]
pub mod cli;
