use criterion::{Criterion, criterion_group, criterion_main};
use rwer::{cer, process_words, wer};
use std::hint::black_box;

fn bench_wer(c: &mut Criterion) {
    let reference = "the quick brown fox jumps over the lazy dog";
    let hypothesis = "the quick brown fox jumped over the lazy dog";

    c.bench_function("wer_short", |b| {
        b.iter(|| wer(black_box(reference), black_box(hypothesis)))
    });

    let long_ref = "the cat sat on the mat and the dog ran in the park ".repeat(100);
    let long_hyp = "the cat sat on a mat and the dog ran in the park ".repeat(100);

    c.bench_function("wer_long", |b| {
        b.iter(|| wer(black_box(&long_ref), black_box(&long_hyp)))
    });
}

fn bench_cer(c: &mut Criterion) {
    let reference = "hello world this is a test";
    let hypothesis = "helo world this is a test";

    c.bench_function("cer_short", |b| {
        b.iter(|| cer(black_box(reference), black_box(hypothesis)))
    });
}

fn bench_process_words(c: &mut Criterion) {
    let reference = "the quick brown fox jumps over the lazy dog";
    let hypothesis = "the quick brown fox jumped over the lazy dog";

    c.bench_function("process_words", |b| {
        b.iter(|| process_words(black_box(reference), black_box(hypothesis)))
    });
}

criterion_group!(benches, bench_wer, bench_cer, bench_process_words);
criterion_main!(benches);
