//! CLI interface for WER/CER evaluation.

#[cfg(feature = "cli")]
use clap::Parser;

#[cfg(feature = "cli")]
use crate::{
    Compose, NormalizeSpaces, RemovePunctuation, Strip, ToLower, Transform, cer, process_chars,
    process_words, visualize_alignment, wer,
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

    /// Read reference text from file (use `-` for stdin)
    #[arg(long)]
    pub ref_file: Option<String>,

    /// Read hypothesis text from file (use `-` for stdin)
    #[arg(long)]
    pub hyp_file: Option<String>,

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

    /// Normalize spaces (collapse consecutive spaces and remove spaces between CJK characters)
    #[arg(short = 'w', long)]
    pub normalize_spaces: bool,

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
    }
    if cli.normalize_spaces {
        transforms.push(Box::new(NormalizeSpaces));
    }

    if transforms.is_empty() {
        None
    } else {
        Some(Box::new(Compose::new(transforms)))
    }
}

/// Resolve reference and hypothesis texts from CLI arguments.
///
/// When `--ref-file` is set, the first positional arg (if any) is treated as
/// hypothesis instead of reference. Similarly, when `--hyp-file` is set,
/// the first positional arg is treated as reference.
/// A file path of `-` reads from stdin.
///
/// # Errors
///
/// Returns an error if `--ref-file` or `--hyp-file` points to a file
/// that cannot be read, or if stdin cannot be read when the path is `-`.
#[cfg(feature = "cli")]
pub fn resolve_inputs(cli: &Cli) -> Result<(String, String), String> {
    resolve_inputs_with_reader(cli, &mut std::io::stdin().lock())
}

/// Resolve inputs using a custom reader for stdin paths (testable).
fn resolve_inputs_with_reader<R: std::io::Read>(
    cli: &Cli,
    mut stdin: R,
) -> Result<(String, String), String> {
    let (reference, hypothesis) = match (&cli.ref_file, &cli.hyp_file) {
        // Both from files — positional args ignored
        (Some(ref_path), Some(hyp_path)) => (
            read_text_from(ref_path, "reference", &mut stdin)?,
            read_text_from(hyp_path, "hypothesis", &mut stdin)?,
        ),
        // Ref from file — positional args map to hypothesis
        (Some(ref_path), None) => (
            read_text_from(ref_path, "reference", &mut stdin)?,
            cli.hypothesis
                .as_deref()
                .or(cli.reference.as_deref())
                .unwrap_or("")
                .to_string(),
        ),
        // Hyp from file — positional args map to reference
        (None, Some(hyp_path)) => (
            cli.reference
                .as_deref()
                .or(cli.hypothesis.as_deref())
                .unwrap_or("")
                .to_string(),
            read_text_from(hyp_path, "hypothesis", &mut stdin)?,
        ),
        // No file flags — use positional args directly
        (None, None) => (
            cli.reference.clone().unwrap_or_default(),
            cli.hypothesis.clone().unwrap_or_default(),
        ),
    };

    Ok((reference, hypothesis))
}

/// Read text from a file path or a reader (when path is `-`).
fn read_text_from<R: std::io::Read>(
    path: &str,
    label: &str,
    stdin: &mut R,
) -> Result<String, String> {
    if path == "-" {
        let mut buf = String::new();
        stdin
            .read_to_string(&mut buf)
            .map_err(|e| format!("failed to read {label} from stdin: {e}"))?;
        Ok(buf)
    } else {
        std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read {label} file '{path}': {e}"))
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

#[cfg(test)]
mod tests {
    use super::*;

    fn cli_with_ref_hyp(ref_text: &str, hyp_text: &str) -> Cli {
        Cli {
            reference: Some(ref_text.to_string()),
            hypothesis: Some(hyp_text.to_string()),
            ref_file: None,
            hyp_file: None,
            character: false,
            alignment: false,
            all: false,
            lowercase: false,
            remove_punctuation: false,
            normalize_spaces: false,
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
        cli.normalize_spaces = true;
        let pipeline = build_pipeline(&cli).unwrap();
        let result = pipeline.transform("Hello,  World!");
        assert_eq!(result, "hello world");
    }

    #[test]
    fn build_pipeline_normalize_spaces() {
        let mut cli = cli_with_ref_hyp("hello", "hello");
        cli.normalize_spaces = true;
        let pipeline = build_pipeline(&cli).unwrap();
        let result = pipeline.transform("hello   world");
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
            Box::new(NormalizeSpaces),
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
                ref_file: None,
                hyp_file: None,
                character: false,
                alignment: false,
                all: false,
                lowercase: false,
                remove_punctuation: false,
                normalize_spaces: false,
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

    // --- resolve_inputs tests ---

    #[test]
    fn resolve_inputs_positional_only() {
        let cli = cli_with_ref_hyp("hello", "world");
        let (r, h) = resolve_inputs(&cli).unwrap();
        assert_eq!(r, "hello");
        assert_eq!(h, "world");
    }

    #[test]
    fn resolve_inputs_ref_file_overrides_positional() {
        let dir = std::env::temp_dir().join("rwer_test_resolve_ref");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("ref.txt");
        std::fs::write(&path, "file reference content").unwrap();

        let mut cli = cli_with_ref_hyp("positional ref", "positional hyp");
        cli.ref_file = Some(path.to_string_lossy().to_string());
        let (r, h) = resolve_inputs(&cli).unwrap();
        assert_eq!(r, "file reference content");
        assert_eq!(h, "positional hyp");

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn resolve_inputs_hyp_file_overrides_positional() {
        let dir = std::env::temp_dir().join("rwer_test_resolve_hyp");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("hyp.txt");
        std::fs::write(&path, "file hypothesis content").unwrap();

        let mut cli = cli_with_ref_hyp("positional ref", "positional hyp");
        cli.hyp_file = Some(path.to_string_lossy().to_string());
        let (r, h) = resolve_inputs(&cli).unwrap();
        assert_eq!(r, "positional ref");
        assert_eq!(h, "file hypothesis content");

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn resolve_inputs_both_files() {
        let dir = std::env::temp_dir().join("rwer_test_resolve_both");
        std::fs::create_dir_all(&dir).unwrap();
        let ref_path = dir.join("ref.txt");
        let hyp_path = dir.join("hyp.txt");
        std::fs::write(&ref_path, "file ref").unwrap();
        std::fs::write(&hyp_path, "file hyp").unwrap();

        let mut cli = cli_with_ref_hyp("pos ref", "pos hyp");
        cli.ref_file = Some(ref_path.to_string_lossy().to_string());
        cli.hyp_file = Some(hyp_path.to_string_lossy().to_string());
        let (r, h) = resolve_inputs(&cli).unwrap();
        assert_eq!(r, "file ref");
        assert_eq!(h, "file hyp");

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn resolve_inputs_missing_file() {
        let mut cli = cli_with_ref_hyp("ref", "hyp");
        cli.ref_file = Some("/nonexistent/path/file.txt".to_string());
        let result = resolve_inputs(&cli);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("failed to read reference file")
        );
    }

    #[test]
    fn resolve_inputs_empty_when_no_positional_no_file() {
        let cli = Cli {
            reference: None,
            hypothesis: None,
            ref_file: None,
            hyp_file: None,
            character: false,
            alignment: false,
            all: false,
            lowercase: false,
            remove_punctuation: false,
            normalize_spaces: false,
            #[cfg(feature = "chinese-variant")]
            simplified: false,
        };
        let (r, h) = resolve_inputs(&cli).unwrap();
        assert_eq!(r, "");
        assert_eq!(h, "");
    }

    #[test]
    fn resolve_inputs_ref_file_remaps_positional_to_hypothesis() {
        let dir = std::env::temp_dir().join("rwer_test_remap_ref");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("ref.txt");
        std::fs::write(&path, "file ref").unwrap();

        // Only one positional arg — should be treated as hypothesis
        let mut cli = cli_with_ref_hyp("positional text", "positional hyp");
        cli.ref_file = Some(path.to_string_lossy().to_string());
        let (r, h) = resolve_inputs(&cli).unwrap();
        assert_eq!(r, "file ref");
        assert_eq!(h, "positional hyp");

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn resolve_inputs_hyp_file_remaps_positional_to_reference() {
        let dir = std::env::temp_dir().join("rwer_test_remap_hyp");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("hyp.txt");
        std::fs::write(&path, "file hyp").unwrap();

        let mut cli = cli_with_ref_hyp("positional ref", "positional text");
        cli.hyp_file = Some(path.to_string_lossy().to_string());
        let (r, h) = resolve_inputs(&cli).unwrap();
        assert_eq!(r, "positional ref");
        assert_eq!(h, "file hyp");

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn resolve_inputs_stdin_hyp() {
        let dir = std::env::temp_dir().join("rwer_test_stdin_hyp");
        std::fs::create_dir_all(&dir).unwrap();
        let ref_path = dir.join("ref.txt");
        std::fs::write(&ref_path, "file ref").unwrap();

        let mut cli = cli_with_ref_hyp("pos ref", "pos hyp");
        cli.ref_file = Some(ref_path.to_string_lossy().to_string());
        cli.hyp_file = Some("-".to_string());

        let stdin = std::io::Cursor::new(b"stdin hypothesis content");
        let (r, h) = resolve_inputs_with_reader(&cli, stdin).unwrap();
        assert_eq!(r, "file ref");
        assert_eq!(h, "stdin hypothesis content");

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn resolve_inputs_stdin_ref() {
        let dir = std::env::temp_dir().join("rwer_test_stdin_ref");
        std::fs::create_dir_all(&dir).unwrap();
        let hyp_path = dir.join("hyp.txt");
        std::fs::write(&hyp_path, "file hyp").unwrap();

        let mut cli = cli_with_ref_hyp("pos ref", "pos hyp");
        cli.ref_file = Some("-".to_string());
        cli.hyp_file = Some(hyp_path.to_string_lossy().to_string());

        let stdin = std::io::Cursor::new(b"stdin reference content");
        let (r, h) = resolve_inputs_with_reader(&cli, stdin).unwrap();
        assert_eq!(r, "stdin reference content");
        assert_eq!(h, "file hyp");

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn resolve_inputs_both_stdin() {
        let mut cli = cli_with_ref_hyp("pos ref", "pos hyp");
        cli.ref_file = Some("-".to_string());
        cli.hyp_file = Some("-".to_string());

        // stdin is consumed by the first read, second gets empty
        let stdin = std::io::Cursor::new(b"stdin ref content");
        let (r, h) = resolve_inputs_with_reader(&cli, stdin).unwrap();
        assert_eq!(r, "stdin ref content");
        assert_eq!(h, "");
    }
}
