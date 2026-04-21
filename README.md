# rwer

**English** | [简体中文](README.zh-CN.md)

A modern Rust crate for Word Error Rate (WER), Character Error Rate (CER), and related ASR evaluation metrics.

## Features

- **WER** (Word Error Rate): `(S + D + I) / N`
- **CER** (Character Error Rate): Same formula at Unicode grapheme cluster level
- **MER** (Match Error Rate): `(S + D + I) / (H + S + D + I)`
- **WIP** (Word Information Preserved): `(H/N) * (H/(H+S+D+I))`
- **WIL** (Word Information Lost): `1 - WIP`
- **Transform pipeline** for text preprocessing (lowercase, remove punctuation, etc.)
- **Chinese word segmentation** via jieba-rs for word-level WER (optional feature)
- **Alignment visualization** with error frequency analysis

## Quick Start

```rust
use rwer::{cer, wer};

let reference = "the cat sat on the mat";
let hypothesis = "the cat sat on a mat";

println!("WER: {:.2}%", wer(reference, hypothesis) * 100.0);
println!("CER: {:.2}%", cer(reference, hypothesis) * 100.0);
```

## All Metrics at Once

```rust
use rwer::{process_words, visualize_alignment};

let output = process_words("the cat sat", "the dog sat");
println!("{output}");
println!("{}", visualize_alignment(&output));
```

Output:
```
WER:  16.67%
MER:  16.67%
WIP:  0.7778
WIL:  0.2222
Hits: 4  Sub: 1  Del: 0  Ins: 0
REF: the cat sat
HYP: the dog sat
```

## Transform Pipeline

```rust
use rwer::{wer, Compose, ToLower, RemovePunctuation, Transform};

let pipeline: Box<dyn Transform> = Box::new(Compose::new(vec![
    Box::new(ToLower),
    Box::new(RemovePunctuation),
]));

let ref_text = pipeline.transform("Hello, World!");
let hyp_text = pipeline.transform("hello world");
assert!(wer(&ref_text, &hyp_text) < 1e-10);
```

### Available Transforms

| Transform | Description |
|-----------|-------------|
| `ToLower` | Convert to lowercase |
| `ToUpper` | Convert to uppercase |
| `Strip` | Strip leading/trailing whitespace |
| `RemovePunctuation` | Remove Unicode punctuation |
| `RemoveMultipleSpaces` | Collapse consecutive spaces |
| `RemoveWhitespace` | Remove all whitespace |
| `SubstituteWords` | Replace whole words via a map |
| `RemoveSpecificWords` | Remove specified words |
| `ExpandCommonEnglishContractions` | Expand contractions (e.g., "don't" -> "do not") |

## Chinese Word-Level WER

> **Note:** Character-level metrics (CER) work with Chinese text out of the box — no feature flag needed.

Chinese word segmentation via jieba-rs is enabled by default. If you want to disable it:

```toml
[dependencies]
rwer = { version = "0.1", default-features = false }
```

```rust
use rwer::chinese_wer;

let result = chinese_wer("今天天气真好", "今天天气很棒");
println!("Chinese WER: {:.2}%", result * 100.0);
```

You can also use the tokenizer directly:

```rust
use rwer::ChineseTokenizer;

let tokenizer = ChineseTokenizer::new();
let words = tokenizer.cut("我们中出了一个叛徒");
println!("{:?}", words);
```

## CLI

Enable the `cli` feature:

```toml
[dependencies]
rwer = { version = "0.1", features = ["cli"] }
```

```bash
# Install
cargo install rwer --features cli

# Basic WER
rwer "the cat sat on the mat" "the cat sat on a mat"

# CER mode
rwer --character "hello" "helo"

# Show alignment
rwer --alignment "the cat sat" "the dog sat"

# All metrics
rwer --all "the cat sat" "the dog sat"

# With normalization
rwer --lowercase --remove-punctuation "Hello, World!" "hello world"
```

## Error Analysis

```rust
use rwer::{collect_error_counts, process_words};

let output = process_words("the cat sat on the mat", "a cat stood on a mat");
let errors = collect_error_counts(&output);

println!("Substitutions: {:?}", errors.substitutions);
println!("Insertions: {:?}", errors.insertions);
println!("Deletions: {:?}", errors.deletions);
```

## Feature Flags

| Feature | Description | Dependencies |
|---------|-------------|--------------|
| `chinese-word` | Chinese word segmentation for word-level WER (default) | `jieba-rs` |
| `cli` | CLI binary | `clap`, `serde`, `serde_json` |

## Benchmarks

```bash
cargo bench
```

## License

Licensed under [MIT](LICENSE-MIT).
