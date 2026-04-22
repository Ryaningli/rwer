#![cfg(feature = "chinese-variant")]

use rwer::{Compose, ToLower, ToSimplified, ToTraditional, Transform, cer, wer};

#[test]
fn traditional_to_simplified_reduces_wer() {
    let pipeline = Compose::new(vec![Box::new(ToSimplified)]);

    let ref_text = pipeline.transform("简体中文测试");
    let hyp_text = pipeline.transform("簡體中文測試");
    assert_eq!(wer(&ref_text, &hyp_text), 0.0);
}

#[test]
fn simplified_to_traditional_reduces_wer() {
    let pipeline = Compose::new(vec![Box::new(ToTraditional)]);

    let ref_text = pipeline.transform("簡體中文測試");
    let hyp_text = pipeline.transform("简体中文测试");
    assert_eq!(wer(&ref_text, &hyp_text), 0.0);
}

#[test]
fn variant_normalization_in_cer() {
    let pipeline = Compose::new(vec![Box::new(ToSimplified)]);

    let ref_text = pipeline.transform("繁體中文");
    let hyp_text = pipeline.transform("繁体中文");
    assert_eq!(cer(&ref_text, &hyp_text), 0.0);
}

#[test]
fn variant_with_lowercase_pipeline() {
    let pipeline = Compose::new(vec![Box::new(ToSimplified), Box::new(ToLower)]);

    let ref_text = pipeline.transform("繁體中文");
    let hyp_text = pipeline.transform("繁體中文");
    assert_eq!(wer(&ref_text, &hyp_text), 0.0);
}

#[test]
fn no_variant_mismatch_without_transform() {
    // Without the transform, traditional and simplified are different
    let ref_text = "简体中文";
    let hyp_text = "簡體中文";
    assert!(wer(&ref_text, &hyp_text) > 0.0);
}
