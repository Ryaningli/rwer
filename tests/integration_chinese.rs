#![cfg(feature = "chinese-word")]

use rwer::{ChineseWordSegment, Compose, ToLower, Transform, process_words, visualize_alignment};

#[test]
fn chinese_word_segment_transform_basic() {
    let t = ChineseWordSegment::new();
    let result = t.transform("今天天气真好");
    assert!(!result.is_empty());
    assert!(result.contains(' '));
}

#[test]
fn chinese_word_segment_transform_empty() {
    let t = ChineseWordSegment::new();
    assert_eq!(t.transform(""), "");
}

#[test]
fn chinese_word_segment_in_pipeline() {
    let pipeline = Compose::new(vec![Box::new(ChineseWordSegment::new())]);
    let ref_text = pipeline.transform("今天天气真好");
    let hyp_text = pipeline.transform("今天天气很棒");

    let output = process_words(&ref_text, &hyp_text);
    assert!(output.wer > 0.0);
    assert!(output.wer <= 1.0);
}

#[test]
fn chinese_segment_with_lowercase() {
    let pipeline = Compose::new(vec![Box::new(ChineseWordSegment::new()), Box::new(ToLower)]);
    let ref_text = pipeline.transform("Hello世界");
    let hyp_text = pipeline.transform("hello世界");
    let output = process_words(&ref_text, &hyp_text);
    assert!(output.wer < 1e-10);
}

#[test]
fn chinese_wer_perfect_via_transform() {
    let pipeline = Compose::new(vec![Box::new(ChineseWordSegment::new())]);
    let ref_text = pipeline.transform("今天天气真好");
    let hyp_text = pipeline.transform("今天天气真好");
    let output = process_words(&ref_text, &hyp_text);
    assert!(output.wer < 1e-10);
}

#[test]
fn chinese_wer_empty_both_via_transform() {
    let pipeline = Compose::new(vec![Box::new(ChineseWordSegment::new())]);
    let ref_text = pipeline.transform("");
    let hyp_text = pipeline.transform("");
    let output = process_words(&ref_text, &hyp_text);
    assert!((output.wer - 0.0).abs() < 1e-10);
}

#[test]
fn chinese_wer_empty_ref_via_transform() {
    let pipeline = Compose::new(vec![Box::new(ChineseWordSegment::new())]);
    let ref_text = pipeline.transform("");
    let hyp_text = pipeline.transform("你好");
    let output = process_words(&ref_text, &hyp_text);
    assert!((output.wer - 0.0).abs() < 1e-10);
}

#[test]
fn chinese_wer_longer_text_via_transform() {
    let pipeline = Compose::new(vec![Box::new(ChineseWordSegment::new())]);
    let ref_text = pipeline.transform("自然语言处理是人工智能领域的一个重要方向");
    let hyp_text = pipeline.transform("自然语言处理是人工智能领域的一个核心方向");
    let output = process_words(&ref_text, &hyp_text);
    assert!((0.0..=1.0).contains(&output.wer));
}

#[test]
fn chinese_wer_with_alignment() {
    let pipeline = Compose::new(vec![Box::new(ChineseWordSegment::new())]);
    let ref_text = pipeline.transform("今天天气真好");
    let hyp_text = pipeline.transform("今天天气很棒");

    let output = process_words(&ref_text, &hyp_text);
    assert!(output.wer > 0.0);
    assert!(output.hits > 0);

    let viz = visualize_alignment(&output);
    assert!(viz.contains("REF:"));
    assert!(viz.contains("HYP:"));
}
