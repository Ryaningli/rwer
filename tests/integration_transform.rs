use rwer::{Compose, NormalizeSpaces, RemovePunctuation, Strip, ToLower, Transform, wer};

#[test]
fn transform_pipeline_reduces_wer() {
    let pipeline = Compose::new(vec![
        Box::new(Strip),
        Box::new(ToLower),
        Box::new(RemovePunctuation),
        Box::new(NormalizeSpaces),
    ]);

    let ref_text = pipeline.transform("Hello, World!");
    let hyp_text = pipeline.transform("hello world");
    assert_eq!(wer(&ref_text, &hyp_text), 0.0);
}

#[test]
fn remove_specific_words_transform() {
    use rwer::RemoveSpecificWords;

    let pipeline = Compose::new(vec![
        Box::new(RemoveSpecificWords::new(&["um", "uh", "like"])),
        Box::new(ToLower),
    ]);

    let ref_text = pipeline.transform("the cat sat");
    let hyp_text = pipeline.transform("um the cat like sat uh");
    assert_eq!(wer(&ref_text, &hyp_text), 0.0);
}

#[test]
fn substitute_words_transform() {
    use rwer::SubstituteWords;

    let pipeline = Compose::new(vec![
        Box::new(SubstituteWords::new(vec![("color", "colour")])),
        Box::new(ToLower),
    ]);

    let ref_text = pipeline.transform("The colour is blue");
    let hyp_text = pipeline.transform("The color is blue");
    assert_eq!(wer(&ref_text, &hyp_text), 0.0);
}

#[test]
fn to_upper_transform() {
    let pipeline = Compose::new(vec![Box::new(rwer::ToUpper)]);
    assert_eq!(pipeline.transform("hello"), "HELLO");
}

#[test]
fn remove_whitespace_transform() {
    let pipeline = Compose::new(vec![Box::new(rwer::RemoveWhitespace)]);
    assert_eq!(pipeline.transform("a b c"), "abc");
}
