# rwer

A modern Rust crate for Word Error Rate (WER), Character Error Rate (CER), and related ASR evaluation metrics.

## Constraints

- **Zero unsafe code** — no `unsafe` blocks anywhere in the codebase
- **Zero bugs** — all code paths must be covered by tests
- **100% unit test coverage** — use `cargo llvm-cov --fail-under-lines 100` to verify
- **clippy** — `cargo clippy -- -D warnings` must pass with zero warnings
- **fmt** — `cargo fmt -- --check` must pass
- **Documentation** — all public items must have doc comments with examples
- **Internationalized docs** — README.md in English (default) + README.zh-CN.md (Chinese)
- **Feature gates** — `chinese-word` feature (default) for jieba-rs word-level Chinese WER, `cli` feature for binary
- **No unwraps in library code** — use `Result`/`Option` properly

## Architecture

- Trait-based transform pipeline for text preprocessing
- Wagner-Fischer alignment algorithm for edit distance
- Separate modules: alignment, metrics, transform, output, chinese

## Module Overview

| Module | Responsibility |
|--------|---------------|
| `alignment` | Wagner-Fischer edit distance, `EditOp` enum, `align()` function |
| `metrics` | `wer()`, `cer()`, `mer()`, `wip()`, `wil()`, `process_words()`, `process_chars()` |
| `transform` | `Transform` trait, `Compose`, `ToLower`, `RemovePunctuation`, etc. |
| `output` | `AlignmentOutput`, `AlignmentChunk`, `visualize_alignment()`, error analysis |
| `chinese` | `ChineseTokenizer` using jieba-rs (behind `chinese-word` feature gate) |

## Chinese Support Note

CER works with Chinese text out of the box (grapheme-level). The `chinese-word` feature is only needed for **word-level** Chinese WER.

## Commit Checklist

- [ ] `cargo fmt`
- [ ] `cargo clippy -- -D warnings`
- [ ] `cargo test`
- [ ] `cargo llvm-cov --fail-under-lines 100` (if llvm-cov available)

## Testing Commands

```bash
cargo test                    # All tests
cargo test -- --nocapture     # Verbose output
cargo test --lib              # Unit tests only
cargo test --test integration_wer  # Specific integration test
cargo test --features chinese-word # Tests with Chinese word-level WER
cargo test --all-features     # All tests with all features
```

## Key Formulas

- **WER** = (S + D + I) / N
- **CER** = Same as WER but at Unicode grapheme cluster level
- **MER** = (S + D + I) / (H + S + D + I)
- **WIP** = (H / N) * (H / (H + S + D + I))
- **WIL** = 1 - WIP

Where: S = substitutions, D = deletions, I = insertions, H = hits, N = reference length
