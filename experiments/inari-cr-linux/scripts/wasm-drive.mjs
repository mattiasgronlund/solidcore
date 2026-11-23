// usage: node wasm-drive.mjs <cr_wasm.wasm> speed [n] [reps]
//        node wasm-drive.mjs <cr_wasm.wasm> hash [n]
//        node wasm-drive.mjs <cr_wasm.wasm> acos-args <file> [n] [reps]
//        node wasm-drive.mjs <cr_wasm.wasm> acos-hash <file>
// Drives cr-wasm's wasm32-unknown-unknown build under V8, as a browser would load it: its one
// import is `env.now_ms`, here `performance.now()`. Prints the report `wspeed` prints.
import { readFileSync } from "node:fs";
import process from "node:process";

const [path, mode, ...rest] = process.argv.slice(2);
const modes = { speed: 0, hash: 1, "acos-args": 2, "acos-hash": 3 };
if (!(mode in modes)) {
  console.error(`wasm-drive: unknown mode ${mode}`);
  process.exit(2);
}
const file = mode.startsWith("acos") ? rest.shift() : undefined;
const num = (i, d) => (rest[i] !== undefined ? Number(rest[i]) : d);
const [n, reps] =
  mode === "hash" ? [num(0, 200000), 0] : mode === "acos-hash" ? [0, 0] : [num(0, 2000000), num(1, 5)];

const { instance } = await WebAssembly.instantiate(readFileSync(path), {
  env: { now_ms: () => performance.now() },
});
const ex = instance.exports;
const input = file ? readFileSync(file) : new Uint8Array(0);
const p = ex.input_buffer(input.length);
new Uint8Array(ex.memory.buffer, p, input.length).set(input);
const code = ex.run(modes[mode], n, reps);
const out = new TextDecoder().decode(new Uint8Array(ex.memory.buffer, ex.output_ptr(), ex.output_len()));
if (code !== 0) {
  console.error(`wasm-drive: ${out}`);
  process.exit(2);
}
process.stdout.write(out);
