use rwer::{cer, process_chars};

#[test]
fn cer_basic() {
    assert!(cer("hello", "hello") < 1e-10);
}

#[test]
fn cer_substitution() {
    // N=3, S=1 → CER = 1/3
    let result = cer("abc", "axc");
    assert!((result - 1.0 / 3.0).abs() < 1e-10);
}

#[test]
fn cer_deletion() {
    // N=3, D=1 → CER = 1/3
    let result = cer("abc", "ac");
    assert!((result - 1.0 / 3.0).abs() < 1e-10);
}

#[test]
fn cer_insertion() {
    // N=2, I=1 → CER = 1/2
    let result = cer("ac", "abc");
    assert!((result - 1.0 / 2.0).abs() < 1e-10);
}

#[test]
fn cer_emoji() {
    assert!(cer("hello 👋🌍", "hello 👋🌍") < 1e-10);
}

#[test]
fn cer_composed_vs_decomposed() {
    // N=1, S=1 → CER = 1/1 = 1.0
    let result = cer("\u{00E9}", "e\u{0301}");
    assert!((result - 1.0).abs() < 1e-10);
}

#[test]
fn cer_process_chars() {
    // N=3, S=1 → CER = 1/3
    let output = process_chars("abc", "axc");
    assert!((output.cer - 1.0 / 3.0).abs() < 1e-10);
    assert_eq!(output.ref_len, 3);
}

#[test]
fn cer_empty() {
    assert!(cer("", "") < 1e-10);
    assert!(cer("", "hello") < 1e-10);
}

#[test]
fn cer_cjk() {
    assert!(cer("你好", "你好") < 1e-10);
    // N=4, S=2 → CER = 2/4 = 0.5
    let result = cer("你好世界", "你好地球");
    assert!((result - 0.5).abs() < 1e-10);
}

#[test]
fn cer_longer_text() {
    let result = cer("the quick brown fox", "the quick red fox");
    assert!(result > 0.0 && result < 1.0);
}

#[test]
fn cer_consistency_with_process() {
    let ref_text = "hello world";
    let hyp_text = "hello earth";
    assert!((cer(ref_text, hyp_text) - process_chars(ref_text, hyp_text).cer).abs() < 1e-10);
}
