import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";

// Load wasm synchronously (Node.js fetch doesn't support file:// URLs)
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const wasmPath = path.resolve(__dirname, "../../pkg/rwer_bg.wasm");
const wasmBytes = readFileSync(wasmPath);

const mod = await import("../../pkg/rwer.js");
mod.initSync({ module: wasmBytes });
const rwer = mod;

describe("wer", () => {
  it("returns 0 for identical strings", () => {
    assert.strictEqual(rwer.wer("hello world", "hello world"), 0);
  });

  it("returns 1 for completely different strings", () => {
    assert.strictEqual(rwer.wer("hello", "world"), 1);
  });

  it("computes deletion correctly", () => {
    const result = rwer.wer("the cat sat", "the sat");
    assert.ok(Math.abs(result - 1 / 3) < 1e-10);
  });

  it("returns 0 for empty strings", () => {
    assert.strictEqual(rwer.wer("", ""), 0);
  });
});

describe("cer", () => {
  it("returns 0 for identical strings", () => {
    assert.strictEqual(rwer.cer("hello", "hello"), 0);
  });

  it("computes substitution correctly", () => {
    const result = rwer.cer("abcde", "axcde");
    assert.ok(Math.abs(result - 0.2) < 1e-10);
  });

  it("handles CJK characters", () => {
    const result = rwer.cer("你好世界", "你们世界");
    assert.ok(Math.abs(result - 0.25) < 1e-10);
  });

  it("handles empty strings", () => {
    assert.strictEqual(rwer.cer("", ""), 0);
  });
});

describe("mer", () => {
  it("returns 0 for identical strings", () => {
    assert.strictEqual(rwer.mer("hello world", "hello world"), 0);
  });

  it("computes correctly with insertion", () => {
    const result = rwer.mer("a", "a b");
    assert.ok(Math.abs(result - 0.5) < 1e-10);
  });
});

describe("wip and wil", () => {
  it("wip returns 1 for perfect match", () => {
    assert.strictEqual(rwer.wip("hello", "hello"), 1);
  });

  it("wil returns 0 for perfect match", () => {
    assert.strictEqual(rwer.wil("hello", "hello"), 0);
  });

  it("wip + wil = 1", () => {
    const wipVal = rwer.wip("the cat", "the dog");
    const wilVal = rwer.wil("the cat", "the dog");
    assert.ok(Math.abs(wipVal + wilVal - 1) < 1e-10);
  });
});

describe("process_words", () => {
  it("returns detailed output", () => {
    const output = rwer.process_words("the cat sat", "the cat sat on");
    assert.strictEqual(output.hits, 3);
    assert.strictEqual(output.insertions, 1);
    assert.strictEqual(output.ref_len, 3);
    assert.strictEqual(output.hyp_len, 4);
    assert.ok(Math.abs(output.wer - 1 / 3) < 1e-10);
  });

  it("returns correct chunks", () => {
    const output = rwer.process_words("hello world", "hello earth");
    const chunks = output.chunks();
    assert.strictEqual(chunks.length, 2);
    assert.strictEqual(chunks[0].kind, "equal");
    assert.strictEqual(chunks[0].text, "hello");
    assert.strictEqual(chunks[1].kind, "substitute");
    assert.strictEqual(chunks[1].text, "world");
    assert.strictEqual(chunks[1].hypothesis, "earth");
  });

  it("visualize returns readable string", () => {
    const output = rwer.process_words("hello world", "hello earth");
    const viz = output.visualize();
    assert.ok(viz.includes("REF:"));
    assert.ok(viz.includes("HYP:"));
  });
});

describe("process_chars", () => {
  it("returns CER and chunks", () => {
    const output = rwer.process_chars("abcde", "axcde");
    assert.ok(Math.abs(output.cer - 0.2) < 1e-10);
    assert.strictEqual(output.hits, 4);
    assert.strictEqual(output.substitutions, 1);
  });

  it("handles CJK correctly", () => {
    const output = rwer.process_chars("你好世界", "你们世纪");
    assert.ok(Math.abs(output.cer - 0.5) < 1e-10);
    assert.strictEqual(output.substitutions, 2);
  });
});
