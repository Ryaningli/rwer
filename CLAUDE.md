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
- **Feature gates** — `chinese-variant` feature for zhconv Traditional/Simplified conversion, `cli` feature for binary
- **No unwraps in library code** — use `Result`/`Option` properly

## Architecture

- Trait-based transform pipeline for text preprocessing
- Wagner-Fischer alignment algorithm for edit distance
- Separate modules: alignment, metrics, transform, output

## Module Overview

| Module | Responsibility |
|--------|---------------|
| `alignment` | Wagner-Fischer edit distance, `EditOp` enum, `align()` function |
| `metrics` | `wer()`, `cer()`, `mer()`, `wip()`, `wil()`, `process_words()`, `process_chars()` |
| `transform` | `Transform` trait, `Compose`, `ToLower`, `RemovePunctuation`, etc. |
| `output` | `AlignmentOutput`, `AlignmentChunk`, `visualize_alignment()`, error analysis |


## Release Checklist

1. Update version in `Cargo.toml`
2. Run all checks:
   - `cargo fmt -- --check`
   - `cargo clippy -- -D warnings`
   - `cargo test --all-features`
   - `cargo llvm-cov --features cli --ignore-filename-regex 'bin/' --fail-under-lines 100`
3. Commit: `release: vX.Y.Z`
4. Tag: `git tag vX.Y.Z`
5. Push: `git push && git push --tags` — GitHub Actions will auto-publish to crates.io on tag push

## Commit Checklist

- [ ] `cargo fmt`
- [ ] `cargo clippy -- -D warnings`
- [ ] `cargo test`
- [ ] `cargo llvm-cov --features cli --ignore-filename-regex 'bin/' --fail-under-lines 100` (if llvm-cov available)

## Testing Commands

```bash
cargo test                    # All tests
cargo test -- --nocapture     # Verbose output
cargo test --lib              # Unit tests only
cargo test --test integration_wer  # Specific integration test
cargo test --all-features     # All tests with all features
```

## Key Formulas

- **WER** = (S + D + I) / N
- **CER** = Same as WER but at Unicode grapheme cluster level
- **MER** = (S + D + I) / (H + S + D + I)
- **WIP** = (H / N) * (H / (H + S + D + I))
- **WIL** = 1 - WIP

Where: S = substitutions, D = deletions, I = insertions, H = hits, N = reference length
