//! CLI interface for WER/CER evaluation.

#[cfg(feature = "cli")]
use clap::Parser;

#[cfg(feature = "cli")]
use crate::{
    Compose, RemoveMultipleSpaces, RemovePunctuation, Strip, ToLower, Transform, cer,
    process_chars, process_words, visualize_alignment, wer,
};

/// Word Error Rate evaluation tool
#[cfg(feature = "cli")]
#[derive(Parser, Debug)]
#[command(
    name = "rwer",
    version,
    about = "Evaluate WER/CER between reference and hypothesis text"
)]
#[allow(clippy::struct_excessive_bools)]
pub struct Cli {
    /// Reference text
    pub reference: Option<String>,

    /// Hypothesis text
    pub hypothesis: Option<String>,

    /// Use character-level evaluation (CER) instead of word-level (WER)
    #[arg(short, long)]
    pub character: bool,

    /// Show detailed alignment visualization
    #[arg(short = 'a', long)]
    pub alignment: bool,

    /// Show all metrics (WER, MER, WIP, WIL)
    #[arg(long)]
    pub all: bool,

    /// Apply lowercase normalization
    #[arg(short = 'l', long)]
    pub lowercase: bool,

    /// Remove punctuation before evaluation
    #[arg(short = 'r', long)]
    pub remove_punctuation: bool,

    /// Use Chinese word segmentation (requires 'chinese-word' feature)
    #[arg(short = 'z', long)]
    #[cfg(feature = "chinese-word")]
    pub chinese: bool,

    /// Convert to Simplified Chinese before evaluation (requires 'chinese-variant' feature)
    #[arg(short = 's', long)]
    #[cfg(feature = "chinese-variant")]
    pub simplified: bool,
}

/// Build a transform pipeline based on CLI flags.
///
/// Returns `None` if no transforms are needed.
#[cfg(feature = "cli")]
#[must_use]
pub fn build_pipeline(cli: &Cli) -> Option<Box<dyn Transform>> {
    let mut transforms: Vec<Box<dyn Transform>> = Vec::new();

    #[cfg(feature = "chinese-variant")]
    if cli.simplified {
        transforms.push(Box::new(crate::ToSimplified));
    }

    if cli.lowercase {
        transforms.push(Box::new(Strip));
        transforms.push(Box::new(ToLower));
    }
    if cli.remove_punctuation {
        transforms.push(Box::new(RemovePunctuation));
        transforms.push(Box::new(RemoveMultipleSpaces));
    }

    #[cfg(feature = "chinese-word")]
    if cli.chinese && !cli.character {
        transforms.push(Box::new(crate::ChineseWordSegment::new()));
    }

    if transforms.is_empty() {
        None
    } else {
        Some(Box::new(Compose::new(transforms)))
    }
}

/// Input parameters for evaluation, decoupled from CLI argument parsing.
#[cfg(feature = "cli")]
#[derive(Debug, Clone)]
pub struct EvalInput {
    /// Reference text.
    pub reference: String,
    /// Hypothesis text.
    pub hypothesis: String,
    /// Use character-level evaluation (CER) instead of word-level (WER).
    pub character: bool,
    /// Show detailed alignment visualization.
    pub alignment: bool,
    /// Show all metrics (WER, MER, WIP, WIL).
    pub all: bool,
}

/// Process input texts and return formatted evaluation result.
#[cfg(feature = "cli")]
#[must_use]
pub fn process_and_format(input: &EvalInput, pipeline: Option<&dyn Transform>) -> String {
    use std::fmt::Write;

    let ref_processed = pipeline.map_or_else(
        || input.reference.clone(),
        |p| p.transform(&input.reference),
    );
    let hyp_processed = pipeline.map_or_else(
        || input.hypothesis.clone(),
        |p| p.transform(&input.hypothesis),
    );

    let mut result = String::new();

    if input.character {
        if input.all || input.alignment {
            let output = process_chars(&ref_processed, &hyp_processed);
            let _ = write!(result, "{output}");
            if input.alignment {
                let _ = write!(result, "\n{}", visualize_alignment(&output));
            }
        } else {
            let _ = write!(result, "{:.4}", cer(&ref_processed, &hyp_processed));
        }
    } else if input.all || input.alignment {
        let output = process_words(&ref_processed, &hyp_processed);
        let _ = write!(result, "{output}");
        if input.alignment {
            let _ = write!(result, "\n{}", visualize_alignment(&output));
        }
    } else {
        let _ = write!(result, "{:.4}", wer(&ref_processed, &hyp_processed));
    }

    result
}

/// Parse CLI arguments and run the WER/CER evaluation.
#[cfg(feature = "cli")]
pub fn run() {
    let cli = Cli::parse();

    let input = EvalInput {
        reference: cli.reference.clone().unwrap_or_default(),
        hypothesis: cli.hypothesis.clone().unwrap_or_default(),
        character: cli.character,
        alignment: cli.alignment,
        all: cli.all,
    };

    let pipeline = build_pipeline(&cli);
    let pipeline_ref = pipeline.as_deref();
    let result = process_and_format(&input, pipeline_ref);
    print!("{result}");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cli_with_ref_hyp(ref_text: &str, hyp_text: &str) -> Cli {
        Cli {
            reference: Some(ref_text.to_string()),
            hypothesis: Some(hyp_text.to_string()),
            character: false,
            alignment: false,
            all: false,
            lowercase: false,
            remove_punctuation: false,
            #[cfg(feature = "chinese-word")]
            chinese: false,
            #[cfg(feature = "chinese-variant")]
            simplified: false,
        }
    }

    // --- build_pipeline tests ---

    #[test]
    fn build_pipeline_no_transforms() {
        let cli = cli_with_ref_hyp("hello", "hello");
        assert!(build_pipeline(&cli).is_none());
    }

    #[test]
    fn build_pipeline_lowercase() {
        let mut cli = cli_with_ref_hyp("Hello", "hello");
        cli.lowercase = true;
        let pipeline = build_pipeline(&cli).unwrap();
        let result = pipeline.transform("Hello, World!");
        assert_eq!(result, "hello, world!");
    }

    #[test]
    fn build_pipeline_remove_punctuation() {
        let mut cli = cli_with_ref_hyp("hello", "hello");
        cli.remove_punctuation = true;
        let pipeline = build_pipeline(&cli).unwrap();
        let result = pipeline.transform("Hello, World!");
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn build_pipeline_lowercase_and_remove_punctuation() {
        let mut cli = cli_with_ref_hyp("Hello, World!", "hello world");
        cli.lowercase = true;
        cli.remove_punctuation = true;
        let pipeline = build_pipeline(&cli).unwrap();
        let result = pipeline.transform("Hello, World!");
        assert_eq!(result, "hello world");
    }

    // --- process_and_format tests ---

    #[test]
    fn process_and_format_wer_basic() {
        let input = EvalInput {
            reference: "the cat sat on the mat".to_string(),
            hypothesis: "the cat sat on a mat".to_string(),
            character: false,
            alignment: false,
            all: false,
        };
        let result = process_and_format(&input, None);
        let wer_val: f64 = result.trim().parse().unwrap();
        assert!(wer_val > 0.0);
        assert!(wer_val <= 1.0);
    }

    #[test]
    fn process_and_format_wer_perfect() {
        let input = EvalInput {
            reference: "hello world".to_string(),
            hypothesis: "hello world".to_string(),
            character: false,
            alignment: false,
            all: false,
        };
        let result = process_and_format(&input, None);
        assert_eq!(result.trim(), "0.0000");
    }

    #[test]
    fn process_and_format_cer() {
        let input = EvalInput {
            reference: "hello world".to_string(),
            hypothesis: "hello".to_string(),
            character: true,
            alignment: false,
            all: false,
        };
        let result = process_and_format(&input, None);
        let cer_val: f64 = result.trim().parse().unwrap();
        assert!(cer_val > 0.0);
    }

    #[test]
    fn process_and_format_cer_perfect() {
        let input = EvalInput {
            reference: "hello".to_string(),
            hypothesis: "hello".to_string(),
            character: true,
            alignment: false,
            all: false,
        };
        let result = process_and_format(&input, None);
        assert_eq!(result.trim(), "0.0000");
    }

    #[test]
    fn process_and_format_all() {
        let input = EvalInput {
            reference: "the cat sat".to_string(),
            hypothesis: "the dog sat".to_string(),
            character: false,
            alignment: false,
            all: true,
        };
        let result = process_and_format(&input, None);
        assert!(result.contains("WER:"));
        assert!(result.contains("MER:"));
        assert!(result.contains("WIP:"));
    }

    #[test]
    fn process_and_format_alignment() {
        let input = EvalInput {
            reference: "the cat sat".to_string(),
            hypothesis: "the dog sat".to_string(),
            character: false,
            alignment: true,
            all: false,
        };
        let result = process_and_format(&input, None);
        assert!(result.contains("REF:"));
        assert!(result.contains("HYP:"));
    }

    #[test]
    fn process_and_format_wer_all_with_alignment() {
        let input = EvalInput {
            reference: "a b c".to_string(),
            hypothesis: "a x c".to_string(),
            character: false,
            alignment: true,
            all: true,
        };
        let result = process_and_format(&input, None);
        assert!(result.contains("REF:"));
        assert!(result.contains("HYP:"));
        assert!(result.contains("WER:"));
    }

    #[test]
    fn process_and_format_cer_all() {
        let input = EvalInput {
            reference: "ab".to_string(),
            hypothesis: "ac".to_string(),
            character: true,
            alignment: false,
            all: true,
        };
        let result = process_and_format(&input, None);
        assert!(result.contains("WER:"));
        assert!(result.contains("CER:"));
    }

    #[test]
    fn process_and_format_cer_alignment() {
        let input = EvalInput {
            reference: "abc".to_string(),
            hypothesis: "axc".to_string(),
            character: true,
            alignment: true,
            all: false,
        };
        let result = process_and_format(&input, None);
        assert!(result.contains("REF:"));
        assert!(result.contains("HYP:"));
    }

    #[test]
    fn process_and_format_with_pipeline() {
        let input = EvalInput {
            reference: "Hello, World!".to_string(),
            hypothesis: "hello world".to_string(),
            character: false,
            alignment: false,
            all: false,
        };
        let pipeline: Box<dyn Transform> = Box::new(Compose::new(vec![
            Box::new(ToLower),
            Box::new(RemovePunctuation),
            Box::new(RemoveMultipleSpaces),
        ]));
        let result = process_and_format(&input, Some(pipeline.as_ref()));
        let wer_val: f64 = result.trim().parse().unwrap();
        assert!(wer_val < 1e-10);
    }

    #[test]
    fn process_and_format_empty_both() {
        let input = EvalInput {
            reference: String::new(),
            hypothesis: String::new(),
            character: false,
            alignment: false,
            all: false,
        };
        let result = process_and_format(&input, None);
        assert_eq!(result.trim(), "0.0000");
    }

    #[test]
    fn process_and_format_empty_ref() {
        let input = EvalInput {
            reference: String::new(),
            hypothesis: "hello".to_string(),
            character: false,
            alignment: false,
            all: false,
        };
        let result = process_and_format(&input, None);
        assert_eq!(result.trim(), "0.0000");
    }

    // --- Chinese word segmentation tests ---

    #[cfg(feature = "chinese-word")]
    mod chinese_tests {
        use super::*;
        use crate::ChineseWordSegment;

        #[test]
        fn build_pipeline_chinese_word() {
            let cli = Cli {
                reference: Some("今天天气真好".to_string()),
                hypothesis: Some("今天天气很棒".to_string()),
                character: false,
                alignment: false,
                all: false,
                lowercase: false,
                remove_punctuation: false,
                chinese: true,
                #[cfg(feature = "chinese-variant")]
                simplified: false,
            };
            let pipeline = build_pipeline(&cli).unwrap();
            let result = pipeline.transform("今天天气真好");
            assert!(result.contains(' '));
        }

        #[test]
        fn build_pipeline_chinese_word_skipped_for_cer() {
            let cli = Cli {
                reference: Some("今天天气真好".to_string()),
                hypothesis: Some("今天天气很棒".to_string()),
                character: true,
                alignment: false,
                all: false,
                lowercase: false,
                remove_punctuation: false,
                chinese: true,
                #[cfg(feature = "chinese-variant")]
                simplified: false,
            };
            let pipeline = build_pipeline(&cli);
            assert!(pipeline.is_none());
        }

        #[test]
        fn process_and_format_chinese_wer() {
            let input = EvalInput {
                reference: "今天天气真好".to_string(),
                hypothesis: "今天天气很棒".to_string(),
                character: false,
                alignment: false,
                all: false,
            };
            let pipeline: Box<dyn Transform> = Box::new(ChineseWordSegment::new());
            let result = process_and_format(&input, Some(pipeline.as_ref()));
            let wer_val: f64 = result.trim().parse().unwrap();
            assert!(wer_val > 0.0);
            assert!(wer_val <= 1.0);
        }
    }

    // --- Chinese variant tests ---

    #[cfg(feature = "chinese-variant")]
    mod variant_tests {
        use super::*;
        use crate::ToSimplified;

        #[test]
        fn build_pipeline_simplified() {
            let cli = Cli {
                reference: Some("今天天氣真好".to_string()),
                hypothesis: Some("今天天气很棒".to_string()),
                character: false,
                alignment: false,
                all: false,
                lowercase: false,
                remove_punctuation: false,
                #[cfg(feature = "chinese-word")]
                chinese: false,
                simplified: true,
            };
            let pipeline = build_pipeline(&cli).unwrap();
            let result = pipeline.transform("今天天氣真好");
            assert_eq!(result, "今天天气真好");
        }

        #[test]
        fn process_and_format_simplified_wer() {
            let input = EvalInput {
                reference: "今天天氣真好".to_string(),
                hypothesis: "今天天气很棒".to_string(),
                character: false,
                alignment: false,
                all: false,
            };
            let pipeline: Box<dyn Transform> = Box::new(ToSimplified);
            let result = process_and_format(&input, Some(pipeline.as_ref()));
            let wer_val: f64 = result.trim().parse().unwrap();
            assert!(wer_val > 0.0);
        }
    }
}
