# rwer

[English](README.md) | **简体中文**

一个现代化的 Rust 词错率/字错率及 ASR 评估指标计算库。

## 功能特性

- **WER**（词错率）：`(S + D + I) / N`
- **CER**（字错率）：基于 Unicode 字素簇的相同公式
- **MER**（匹配错误率）：`(S + D + I) / (H + S + D + I)`
- **WIP**（词信息保留度）：`(H/N) * (H/(H+S+D+I))`
- **WIL**（词信息丢失度）：`1 - WIP`
- **文本预处理管道**（小写转换、去除标点等）
- **对齐可视化**与错误频率分析

## 快速开始

```rust
use rwer::{cer, wer};

let reference = "the cat sat on the mat";
let hypothesis = "the cat sat on a mat";

println!("WER: {:.2}%", wer(reference, hypothesis) * 100.0);
println!("CER: {:.2}%", cer(reference, hypothesis) * 100.0);
```

## 一次性获取所有指标

```rust
use rwer::{process_words, visualize_alignment};

let output = process_words("the cat sat", "the dog sat");
println!("{output}");
println!("{}", visualize_alignment(&output));
```

输出：
```
WER:  16.67%
MER:  16.67%
WIP:  0.7778
WIL:  0.2222
Hits: 4  Sub: 1  Del: 0  Ins: 0
REF: the cat sat
HYP: the dog sat
```

## 文本预处理管道

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

### 可用的预处理变换

| 变换 | 说明 |
|------|------|
| `ToLower` | 转换为小写 |
| `ToUpper` | 转换为大写 |
| `Strip` | 去除首尾空白 |
| `RemovePunctuation` | 去除 Unicode 标点符号 |
| `NormalizeSpaces` | 合并连续空格 + 移除 CJK 字符间的空格 |
| `RemoveWhitespace` | 去除所有空白字符 |
| `SubstituteWords` | 按映射表替换整词 |
| `RemoveSpecificWords` | 去除指定词语 |
| `ExpandCommonEnglishContractions` | 展开英语缩写（如 "don't" -> "do not"） |
| `ToSimplified` | 繁体转简体（`chinese-variant` 功能） |
| `ToTraditional` | 简体转繁体（`chinese-variant` 功能） |


## 中文繁简转换

当参考文本和 ASR 输出使用不同的中文书写体系（繁体/简体）时，启用 `chinese-variant` 功能：

```toml
[dependencies]
rwer = { version = "0.1", features = ["chinese-variant"] }
```

```rust
use rwer::{ToSimplified, Compose, Transform, wer};

let pipeline = Compose::new(vec![Box::new(ToSimplified)]);
let ref_text = pipeline.transform("繁體中文");
let hyp_text = pipeline.transform("简体中文");
assert_eq!(wer(&ref_text, &hyp_text), 0.0);
```

CLI 用法：
```bash
rwer -s "繁體中文測試" "简体中文测试"
```

## 命令行工具

启用 `cli` 功能：

```toml
[dependencies]
rwer = { version = "0.1", features = ["cli"] }
```

```bash
# 安装
cargo install rwer --all-features

# 基本文本参数模式
rwer "the cat sat on the mat" "the cat sat on a mat"

# 从文件读取
rwer --ref-file ref.txt --hyp-file hyp.txt

# 混合文本和文件输入
rwer --ref-file ref.txt "the cat sat on a mat"
rwer "the cat sat on the mat" --hyp-file hyp.txt

# 从标准输入读取
echo "the cat sat on a mat" | rwer --ref-file ref.txt --hyp-file -

# CER 模式
rwer --character "hello" "helo"

# 显示对齐
rwer --alignment "the cat sat" "the dog sat"

# 所有指标
rwer --all "the cat sat" "the dog sat"

# 带文本规范化
rwer --lowercase --remove-punctuation --normalize-spaces "Hello,  World!" "hello world"
```

## 错误分析

```rust
use rwer::{collect_error_counts, process_words};

let output = process_words("the cat sat on the mat", "a cat stood on a mat");
let errors = collect_error_counts(&output);

println!("替换: {:?}", errors.substitutions);
println!("插入: {:?}", errors.insertions);
println!("删除: {:?}", errors.deletions);
```

## 功能开关

| 功能 | 说明 | 依赖 |
|------|------|------|
| `chinese-variant` | 中文繁简转换 | `zhconv` |
| `cli` | 命令行工具 | `clap`, `serde`, `serde_json` |

## JavaScript / WebAssembly

rwer 也提供 WebAssembly 包，可在浏览器和 Node.js 中使用：

```bash
npm install rwer
```

```js
import * as rwer from "rwer";

// 浏览器
await rwer.default();

// Node.js
import { readFileSync } from "node:fs";
rwer.initSync({ module: readFileSync("node_modules/rwer/rwer_bg.wasm") });

console.log(rwer.wer("the cat sat", "the dog sat")); // 0.333...
console.log(rwer.cer("hello", "hallo"));             // 0.2

const output = rwer.process_words("the cat sat", "the dog sat");
console.log(output.wer);           // 0.333...
console.log(output.hits);          // 2
console.log(output.substitutions); // 1
console.log(output.chunks());      // [{kind:"equal",...}, {kind:"substitute",...}]
console.log(output.visualize());   // REF: the cat sat\nHYP: the dog sat
```

### 从源码构建

```bash
# 安装 wasm-pack
cargo install wasm-pack

# 构建
cd js && npm run build:wasm

# 或手动构建
wasm-pack build crates/rwer-wasm --target web --out-dir ../../pkg --out-name rwer
```

### 运行 JS 测试

```bash
cd js && npm test
```

## 基准测试

```bash
cargo bench
```

## 致谢

- [jiwer](https://github.com/jitsi/jiwer) — WER/CER 指标的 API 设计和架构参考
- [zhconv](https://github.com/nicemayi/zhconv-rs) — 中文繁简转换

## 许可证

基于 [MIT](LICENSE-MIT) 许可协议。
