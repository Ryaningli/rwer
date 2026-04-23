#![allow(deprecated)]
#![cfg(feature = "chinese-word")]

use rwer::{ChineseTokenizer, chinese_wer};

#[test]
fn chinese_tokenizer_basic() {
    let tokenizer = ChineseTokenizer::new();
    let words = tokenizer.cut("我们中出了一个叛徒");
    assert!(!words.is_empty());
}

#[test]
fn chinese_wer_perfect() {
    assert_eq!(chinese_wer("今天天气真好", "今天天气真好"), 0.0);
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
fn chinese_wer_longer_text() {
    let ref_text = "自然语言处理是人工智能领域的一个重要方向";
    let hyp_text = "自然语言处理是人工智能领域的一个核心方向";
    let result = chinese_wer(ref_text, hyp_text);
    assert!((0.0..=1.0).contains(&result));
}

#[test]
fn chinese_wer_mixed_content() {
    let result = chinese_wer("我喜欢rust编程", "我喜欢python编程");
    assert!((0.0..=1.0).contains(&result));
}

#[test]
fn chinese_wer_with_punctuation() {
    let result = chinese_wer("你好，世界！", "你好，地球！");
    assert!((0.0..=1.0).contains(&result));
}

#[test]
fn chinese_tokenizer_with_process_words() {
    use rwer::{process_words, visualize_alignment};

    let tokenizer = ChineseTokenizer::new();
    let ref_text = "今天天气真好";
    let hyp_text = "今天天气很棒";
    let ref_words: Vec<String> = tokenizer.cut(ref_text);
    let hyp_words: Vec<String> = tokenizer.cut(hyp_text);

    let output = process_words(&ref_words.join(" "), &hyp_words.join(" "));
    assert!(output.wer > 0.0);
    assert!(output.wer <= 1.0);
    assert!(output.hits > 0);

    let viz = visualize_alignment(&output);
    assert!(viz.contains("REF:"));
    assert!(viz.contains("HYP:"));
}

// --- Tests using the new transform-based approach ---

#[test]
fn chinese_word_segment_transform_basic() {
    use rwer::{ChineseWordSegment, Transform};

    let t = ChineseWordSegment::new();
    let result = t.transform("今天天气真好");
    assert!(!result.is_empty());
    assert!(result.contains(' '));
}

#[test]
fn chinese_word_segment_transform_empty() {
    use rwer::{ChineseWordSegment, Transform};

    let t = ChineseWordSegment::new();
    assert_eq!(t.transform(""), "");
}

#[test]
fn chinese_word_segment_in_pipeline() {
    use rwer::{ChineseWordSegment, Compose, Transform, process_words};

    let pipeline = Compose::new(vec![Box::new(ChineseWordSegment::new())]);
    let ref_text = pipeline.transform("今天天气真好");
    let hyp_text = pipeline.transform("今天天气很棒");

    let output = process_words(&ref_text, &hyp_text);
    assert!(output.wer > 0.0);
    assert!(output.wer <= 1.0);
}

#[test]
fn chinese_segment_with_lowercase() {
    use rwer::{ChineseWordSegment, Compose, ToLower, Transform, process_words};

    let pipeline = Compose::new(vec![Box::new(ChineseWordSegment::new()), Box::new(ToLower)]);
    let ref_text = pipeline.transform("Hello世界");
    let hyp_text = pipeline.transform("hello世界");
    let output = process_words(&ref_text, &hyp_text);
    assert!(output.wer < 1e-10);
}
