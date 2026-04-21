use rwer::{Compose, RemovePunctuation, ToLower, Transform};
use rwer::{cer, process_words, visualize_alignment, wer};

fn main() {
    let reference = "the cat sat on the mat";
    let hypothesis = "the cat sat on a mat";

    println!("=== Basic WER/CER ===");
    println!("WER: {:.2}%", wer(reference, hypothesis) * 100.0);
    println!("CER: {:.2}%", cer(reference, hypothesis) * 100.0);

    println!("\n=== All Metrics ===");
    let output = process_words(reference, hypothesis);
    println!("{output}");

    println!("=== Alignment Visualization ===");
    println!("{}", visualize_alignment(&output));

    println!("=== Transform Pipeline ===");
    let pipeline: Box<dyn Transform> = Box::new(Compose::new(vec![
        Box::new(ToLower),
        Box::new(RemovePunctuation),
    ]));

    let ref_norm = pipeline.transform("Hello, World!");
    let hyp_norm = pipeline.transform("hello world");
    println!("Normalized WER: {:.2}%", wer(&ref_norm, &hyp_norm) * 100.0);

    println!("\n=== Chinese WER (if enabled) ===");
    #[cfg(feature = "chinese-word")]
    {
        let cn_ref = "今天天气真好";
        let cn_hyp = "今天天气很棒";
        let cn_result = rwer::chinese_wer(cn_ref, cn_hyp);
        println!("Chinese WER: {:.2}%", cn_result * 100.0);
    }
    #[cfg(not(feature = "chinese-word"))]
    {
        println!("(enable 'chinese-word' feature for Chinese word-level WER)");
    }
}
