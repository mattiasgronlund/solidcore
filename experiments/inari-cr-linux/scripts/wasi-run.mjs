// usage: node wasi-run.mjs [--dir HOST[::GUEST]]... [--env NAME=VALUE]... <module.wasm> [args...]
// Runs a wasm32-wasip1 program under V8 (node's WASI) with wasmtime's `run` options: each
// --dir preopens HOST under the guest path GUEST (default: HOST itself), each --env sets one
// variable. Exits with the program's exit code.
import { readFile } from "node:fs/promises";
import { WASI } from "node:wasi";
import process from "node:process";

const argv = process.argv.slice(2);
const preopens = {};
const env = {};
while (argv.length && argv[0].startsWith("--")) {
  const [opt, val] = [argv.shift(), argv.shift()];
  if (val === undefined) throw new Error(`wasi-run: ${opt} needs a value`);
  if (opt === "--dir") {
    const [host, guest] = val.split("::");
    preopens[guest ?? host] = host;
  } else if (opt === "--env") {
    const i = val.indexOf("=");
    if (i < 1) throw new Error(`wasi-run: --env ${val}: expected NAME=VALUE`);
    env[val.slice(0, i)] = val.slice(i + 1);
  } else {
    throw new Error(`wasi-run: unknown option ${opt}`);
  }
}
const [path, ...args] = argv;
const wasi = new WASI({ version: "preview1", args: [path, ...args], env, preopens, returnOnExit: true });
const module = await WebAssembly.compile(await readFile(path));
const instance = await WebAssembly.instantiate(module, wasi.getImportObject());
process.exitCode = wasi.start(instance);
