use clap::Parser;
use rwer::{
    Compose, RemoveMultipleSpaces, RemovePunctuation, Strip, ToLower, Transform, cer,
    process_chars, process_words, visualize_alignment, wer,
};

/// Word Error Rate evaluation tool
#[derive(Parser, Debug)]
#[command(
    name = "rwer",
    version,
    about = "Evaluate WER/CER between reference and hypothesis text"
)]
struct Cli {
    /// Reference text
    reference: Option<String>,

    /// Hypothesis text
    hypothesis: Option<String>,

    /// Use character-level evaluation (CER) instead of word-level (WER)
    #[arg(short, long)]
    character: bool,

    /// Show detailed alignment visualization
    #[arg(short = 'a', long)]
    alignment: bool,

    /// Show all metrics (WER, MER, WIP, WIL)
    #[arg(long)]
    all: bool,

    /// Apply lowercase normalization
    #[arg(short = 'l', long)]
    lowercase: bool,

    /// Remove punctuation before evaluation
    #[arg(short = 'r', long)]
    remove_punctuation: bool,

    /// Use Chinese word segmentation (requires 'chinese-word' feature)
    #[arg(short = 'z', long)]
    #[cfg(feature = "chinese-word")]
    chinese: bool,

    /// Convert to Simplified Chinese before evaluation (requires 'chinese-variant' feature)
    #[arg(short = 's', long)]
    #[cfg(feature = "chinese-variant")]
    simplified: bool,
}

fn build_pipeline(cli: &Cli) -> Option<Box<dyn Transform>> {
    let mut transforms: Vec<Box<dyn Transform>> = Vec::new();

    #[cfg(feature = "chinese-variant")]
    if cli.simplified {
        transforms.push(Box::new(rwer::ToSimplified));
    }

    if cli.lowercase {
        transforms.push(Box::new(Strip));
        transforms.push(Box::new(ToLower));
    }
    if cli.remove_punctuation {
        transforms.push(Box::new(RemovePunctuation));
        transforms.push(Box::new(RemoveMultipleSpaces));
    }

    if transforms.is_empty() {
        None
    } else {
        Some(Box::new(Compose::new(transforms)))
    }
}

fn main() {
    let cli = Cli::parse();

    let ref_text = cli.reference.as_deref().unwrap_or("");
    let hyp_text = cli.hypothesis.as_deref().unwrap_or("");

    let pipeline = build_pipeline(&cli);
    let ref_processed = pipeline
        .as_ref()
        .map(|p| p.transform(ref_text))
        .unwrap_or_else(|| ref_text.to_string());
    let hyp_processed = pipeline
        .as_ref()
        .map(|p| p.transform(hyp_text))
        .unwrap_or_else(|| hyp_text.to_string());

    #[cfg(feature = "chinese-word")]
    let (ref_final, hyp_final) = if cli.chinese && !cli.character {
        let tokenizer = rwer::ChineseTokenizer::new();
        let ref_cut = tokenizer.cut(&ref_processed);
        let hyp_cut = tokenizer.cut(&hyp_processed);
        let ref_words: Vec<&str> = ref_cut.iter().map(String::as_str).collect();
        let hyp_words: Vec<&str> = hyp_cut.iter().map(String::as_str).collect();
        (ref_words.join(" "), hyp_words.join(" "))
    } else {
        (ref_processed, hyp_processed)
    };
    #[cfg(not(feature = "chinese-word"))]
    let (ref_final, hyp_final) = (ref_processed, hyp_processed);

    if cli.character {
        if cli.all || cli.alignment {
            let output = process_chars(&ref_final, &hyp_final);
            println!("{output}");
            if cli.alignment {
                println!("{}", visualize_alignment(&output));
            }
        } else {
            println!("{:.4}", cer(&ref_final, &hyp_final));
        }
    } else if cli.all || cli.alignment {
        let output = process_words(&ref_final, &hyp_final);
        println!("{output}");
        if cli.alignment {
            println!("{}", visualize_alignment(&output));
        }
    } else {
        println!("{:.4}", wer(&ref_final, &hyp_final));
    }
}
