use rwer::{AlignmentChunk, process_words, wer, wer_sentences};

#[test]
fn wer_basic() {
    let result = wer("the cat sat on the mat", "the cat sat on the mat");
    assert!(result < 1e-10);
}

#[test]
fn wer_with_errors() {
    let result = wer("the cat sat on the mat", "the dog sat on a mat");
    assert!(result > 0.0 && result < 1.0);
}

#[test]
fn wer_process_words_detailed() {
    let output = process_words("the cat sat", "the cat stood");
    assert_eq!(output.ref_len, 3);
    assert_eq!(output.hyp_len, 3);
    assert_eq!(output.hits, 2);
    assert_eq!(output.substitutions, 1);
}

#[test]
fn wer_process_words_chunks() {
    let output = process_words("hello", "world");
    assert!(
        output
            .chunks
            .iter()
            .any(|c| matches!(c, AlignmentChunk::Substitute { .. }))
    );
}

#[test]
fn wer_sentences_integration() {
    let ref_sents = ["the cat sat", "the dog ran"];
    let hyp_sents = ["the cat sat", "a dog ran"];
    let result = wer_sentences(&ref_sents, &hyp_sents);
    assert!(result > 0.0 && result < 1.0);
}

#[test]
fn wer_empty_cases() {
    assert!(wer("", "") < 1e-10);
    assert!(wer("", "hello") < 1e-10);
    assert!(!wer("hello", "").is_nan());
}

#[test]
fn wer_whitespace_handling() {
    assert!(wer("  hello  world  ", "hello world") < 1e-10);
}

#[test]
fn wer_case_sensitive() {
    let result = wer("Hello World", "hello world");
    assert!(result > 0.0);
}

#[test]
fn wer_unicode_text() {
    assert!(wer("你好世界", "你好世界") < 1e-10);
    let result = wer("你好世界", "你好地球");
    assert!(result > 0.0);
}

#[test]
fn wer_hallucination() {
    // All hypothesis words are insertions
    assert!(wer("", "hello world") < 1e-10);
}

#[test]
fn wer_all_deleted() {
    // All reference words are deletions
    let result = wer("a b c", "");
    assert!(result > 0.0 && result <= 1.0);
}

#[test]
fn wer_consistency_with_process() {
    let ref_text = "the quick brown fox";
    let hyp_text = "the slow brown fox";
    assert_eq!(
        wer(ref_text, hyp_text),
        process_words(ref_text, hyp_text).wer
    );
}
