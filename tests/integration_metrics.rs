use rwer::{mer, process_words, wer, wil, wip};

#[test]
fn mer_perfect() {
    assert!(mer("hello world", "hello world") < 1e-10);
}

#[test]
fn mer_with_errors() {
    let result = mer("a b c", "a x c");
    assert!((result - 1.0 / 3.0).abs() < 1e-10);
}

#[test]
fn wip_perfect() {
    assert!((wip("hello world", "hello world") - 1.0).abs() < 1e-10);
}

#[test]
fn wip_no_match() {
    assert!(wip("hello", "world") < 1e-10);
}

#[test]
fn wil_perfect() {
    assert!(wil("hello world", "hello world") < 1e-10);
}

#[test]
fn wil_no_match() {
    assert!((wil("hello", "world") - 1.0).abs() < 1e-10);
}

#[test]
fn process_words_all_metrics_consistent() {
    let output = process_words("the cat sat", "the dog sat");
    assert_eq!(output.wer, wer("the cat sat", "the dog sat"));
    assert_eq!(output.mer, mer("the cat sat", "the dog sat"));
    assert!((output.wip - wip("the cat sat", "the dog sat")).abs() < 1e-10);
    assert!((output.wil - wil("the cat sat", "the dog sat")).abs() < 1e-10);
}

#[test]
fn all_metrics_empty() {
    assert!(mer("", "") < 1e-10);
    assert!((wip("", "") - 1.0).abs() < 1e-10);
    assert!(wil("", "") < 1e-10);
}

#[test]
fn wip_partial_match() {
    // 2 out of 3 match: WIP = (2/3) * (2/3) = 4/9
    let result = wip("a b c", "a b d");
    assert!((result - 4.0 / 9.0).abs() < 1e-10);
}

#[test]
fn mer_with_only_insertions() {
    // ref: ["a"], hyp: ["a", "b"] → S=0, D=0, I=1, H=1
    // MER = 1/(1+0+0+1) = 0.5
    let result = mer("a", "a b");
    assert!((result - 0.5).abs() < 1e-10);
}
