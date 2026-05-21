use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use rwer::{cer, process_chars, process_words, wer};
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

    let long_ref: String = "今天天气真好我们可以出去玩".repeat(200);
    let long_hyp: String = "今天天气真好人我们可以出去玩".repeat(200);

    c.bench_function("cer_long_cjk", |b| {
        b.iter(|| cer(black_box(&long_ref), black_box(&long_hyp)))
    });

    let long_latin_ref = "the cat sat on the mat and the dog ran in the park ".repeat(100);
    let long_latin_hyp = "the cat sat on a mat and the dog ran in the park ".repeat(100);

    c.bench_function("cer_long_latin", |b| {
        b.iter(|| cer(black_box(&long_latin_ref), black_box(&long_latin_hyp)))
    });

    let emoji_ref = "👨‍👩‍👧你好世界👋🌟".repeat(100);
    let emoji_hyp = "👨‍👩‍👦你好世界👋⭐".repeat(100);

    c.bench_function("cer_emoji_fallback", |b| {
        b.iter(|| cer(black_box(&emoji_ref), black_box(&emoji_hyp)))
    });
}

fn bench_process_chars(c: &mut Criterion) {
    let reference = "hello world this is a test";
    let hypothesis = "helo world this is a test";

    c.bench_function("process_chars_short", |b| {
        b.iter(|| process_chars(black_box(reference), black_box(hypothesis)))
    });

    let long_ref: String = "今天天气真好我们可以出去玩".repeat(200);
    let long_hyp: String = "今天天气真好人我们可以出去玩".repeat(200);

    c.bench_function("process_chars_long_cjk", |b| {
        b.iter(|| process_chars(black_box(&long_ref), black_box(&long_hyp)))
    });
}

fn bench_process_words(c: &mut Criterion) {
    let reference = "the quick brown fox jumps over the lazy dog";
    let hypothesis = "the quick brown fox jumped over the lazy dog";

    c.bench_function("process_words_short", |b| {
        b.iter(|| process_words(black_box(reference), black_box(hypothesis)))
    });

    let long_ref = "the cat sat on the mat and the dog ran in the park ".repeat(100);
    let long_hyp = "the cat sat on a mat and the dog ran in the park ".repeat(100);

    c.bench_function("process_words_long", |b| {
        b.iter(|| process_words(black_box(&long_ref), black_box(&long_hyp)))
    });
}

fn bench_rapidfuzz_char_distance(c: &mut Criterion) {
    let short = "hello world";
    let short2 = "hallo world";

    c.bench_function("rapidfuzz_char_short", |b| {
        b.iter(|| {
            let v1: Vec<char> = short.chars().collect();
            let v2: Vec<char> = short2.chars().collect();
            rapidfuzz::distance::levenshtein::distance(
                black_box(v1.iter().copied()),
                black_box(v2.iter().copied()),
            )
        })
    });

    let long: String = "今天天气真好我们可以出去玩".repeat(200);
    let long2: String = "今天天气真好人我们可以出去玩".repeat(200);

    c.bench_function("rapidfuzz_char_long", |b| {
        b.iter(|| {
            let v1: Vec<char> = long.chars().collect();
            let v2: Vec<char> = long2.chars().collect();
            rapidfuzz::distance::levenshtein::distance(
                black_box(v1.iter().copied()),
                black_box(v2.iter().copied()),
            )
        })
    });
}

fn bench_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("cer_scaling");
    for size in [100, 500, 1000, 5000] {
        let text: String = "你好世界测试".repeat(size);
        let text2: String = "你好世界测验".repeat(size);
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| cer(black_box(&text), black_box(&text2)))
        });
    }
    group.finish();
}

fn bench_wer_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("wer_scaling");
    for size in [100, 500, 1000, 5000, 10000] {
        let ref_text: String = "the cat sat on the mat and the dog ran in the park "
            .split_whitespace()
            .cycle()
            .take(size)
            .collect::<Vec<_>>()
            .join(" ");
        let hyp_text: String = ref_text
            .split_whitespace()
            .enumerate()
            .map(|(i, w)| if i % 5 == 0 { "changed" } else { w })
            .collect::<Vec<_>>()
            .join(" ");
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| wer(black_box(&ref_text), black_box(&hyp_text)))
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_wer,
    bench_cer,
    bench_process_chars,
    bench_process_words,
    bench_rapidfuzz_char_distance,
    bench_scaling,
    bench_wer_scaling,
);
criterion_main!(benches);
