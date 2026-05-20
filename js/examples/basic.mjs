import * as rwer from "../../pkg/rwer.js";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const wasmPath = path.resolve(__dirname, "../../pkg/rwer_bg.wasm");
const wasmBytes = readFileSync(wasmPath);
rwer.initSync({ module: wasmBytes });

const ref = "the cat sat on the mat";
const hyp = "the cat sat on a mat";

console.log(`WER: ${(rwer.wer(ref, hyp) * 100).toFixed(2)}%`);
console.log(`CER: ${(rwer.cer(ref, hyp) * 100).toFixed(2)}%`);
console.log(`MER: ${(rwer.mer(ref, hyp) * 100).toFixed(2)}%`);
console.log(`WIP: ${rwer.wip(ref, hyp).toFixed(4)}`);
console.log(`WIL: ${rwer.wil(ref, hyp).toFixed(4)}`);

console.log("\nDetailed word-level analysis:");
const output = rwer.process_words(ref, hyp);
console.log(`  Hits: ${output.hits}`);
console.log(`  Substitutions: ${output.substitutions}`);
console.log(`  Deletions: ${output.deletions}`);
console.log(`  Insertions: ${output.insertions}`);
console.log(`\n${output.visualize()}`);
