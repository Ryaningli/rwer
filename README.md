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
| `NormalizeSpaces` | Collapse consecutive spaces + remove spaces between CJK characters |
| `RemoveWhitespace` | Remove all whitespace |
| `SubstituteWords` | Replace whole words via a map |
| `RemoveSpecificWords` | Remove specified words |
| `ExpandCommonEnglishContractions` | Expand contractions (e.g., "don't" -> "do not") |
| `ToSimplified` | Convert Traditional Chinese to Simplified Chinese (`chinese-variant` feature) |
| `ToTraditional` | Convert Simplified Chinese to Traditional Chinese (`chinese-variant` feature) |


## Chinese Variant Normalization

When comparing ASR outputs that may use different Chinese scripts (Traditional vs Simplified), enable the `chinese-variant` feature:

```toml
[dependencies]
rwer = { version = "0.1", features = ["chinese-variant"] }
```

```rust
use rwer::{ToSimplified, Compose, Transform, wer};

// Normalize both texts to Simplified before comparison
let pipeline = Compose::new(vec![Box::new(ToSimplified)]);
let ref_text = pipeline.transform("繁體中文");
let hyp_text = pipeline.transform("简体中文");
assert_eq!(wer(&ref_text, &hyp_text), 0.0);
```

CLI usage:
```bash
rwer -s "繁體中文測試" "简体中文测试"
```

## CLI

Enable the `cli` feature:

```toml
[dependencies]
rwer = { version = "0.1", features = ["cli"] }
```

```bash
# Install
cargo install rwer --all-features

# Basic WER with text arguments
rwer "the cat sat on the mat" "the cat sat on a mat"

# Read from files
rwer --ref-file ref.txt --hyp-file hyp.txt

# Mix text and file input
rwer --ref-file ref.txt "the cat sat on a mat"
rwer "the cat sat on the mat" --hyp-file hyp.txt

# Read from stdin
echo "the cat sat on a mat" | rwer --ref-file ref.txt --hyp-file -

# CER mode
rwer --character "hello" "helo"

# Show alignment
rwer --alignment "the cat sat" "the dog sat"

# All metrics
rwer --all "the cat sat" "the dog sat"

# With normalization
rwer --lowercase --remove-punctuation --normalize-spaces "Hello,  World!" "hello world"
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
| `chinese-variant` | Traditional/Simplified Chinese conversion | `zhconv` |
| `cli` | CLI binary | `clap`, `serde`, `serde_json` |

## Benchmarks

```bash
cargo bench
```

## Acknowledgments

- [jiwer](https://github.com/jitsi/jiwer) — API design and architecture reference for WER/CER metrics
- [zhconv](https://github.com/nicemayi/zhconv-rs) — Traditional/Simplified Chinese conversion

## License

Licensed under [MIT](LICENSE-MIT).
