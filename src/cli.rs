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

/// Parse CLI arguments and run the WER/CER evaluation.
#[cfg(feature = "cli")]
pub fn run() {
    let cli = Cli::parse();

    let ref_text = cli.reference.as_deref().unwrap_or("");
    let hyp_text = cli.hypothesis.as_deref().unwrap_or("");

    let pipeline = build_pipeline(&cli);
    let ref_processed = pipeline
        .as_ref()
        .map_or_else(|| ref_text.to_string(), |p| p.transform(ref_text));
    let hyp_processed = pipeline
        .as_ref()
        .map_or_else(|| hyp_text.to_string(), |p| p.transform(hyp_text));

    if cli.character {
        if cli.all || cli.alignment {
            let output = process_chars(&ref_processed, &hyp_processed);
            println!("{output}");
            if cli.alignment {
                println!("{}", visualize_alignment(&output));
            }
        } else {
            println!("{:.4}", cer(&ref_processed, &hyp_processed));
        }
    } else if cli.all || cli.alignment {
        let output = process_words(&ref_processed, &hyp_processed);
        println!("{output}");
        if cli.alignment {
            println!("{}", visualize_alignment(&output));
        }
    } else {
        println!("{:.4}", wer(&ref_processed, &hyp_processed));
    }
}
