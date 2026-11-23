//! `wspeed` (native and wasm32-wasip1): the reports of `cr_wasm`.
//!
//! `wspeed [n] [reps]`                  ns per call (cr-eval's `speed`; default 2,000,000, 5)
//! `wspeed --hash [n]`                  hashes of the results on the accuracy arguments
//!                                      (default 200,000, as cr-eval's `accuracy`)
//! `wspeed --acos-args <file> [n] [reps]`   `acos` on an acos-args file (cr-eval's)
//! `wspeed --acos-hash <file>`          hashes of `acos` on it
use std::process::exit;

fn main() {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let num = |i: usize, d: usize| raw.get(i).and_then(|s| s.parse().ok()).unwrap_or(d);
    let file = |i: usize| match raw.get(i) {
        Some(p) => std::fs::read_to_string(p).unwrap_or_else(|e| {
            eprintln!("wspeed: cannot read {p}: {e}");
            exit(2)
        }),
        None => {
            eprintln!("wspeed: {} needs a file", raw[0]);
            exit(2)
        }
    };
    let r = match raw.first().map(String::as_str) {
        Some("--hash") => cr_wasm::run(1, "", num(1, 200_000), 0),
        Some("--acos-args") => cr_wasm::run(2, &file(1), num(2, 2_000_000), num(3, 5)),
        Some("--acos-hash") => cr_wasm::run(3, &file(1), 0, 0),
        _ => cr_wasm::run(0, "", num(0, 2_000_000), num(1, 5)),
    };
    match r {
        Ok(s) => print!("{s}"),
        Err(e) => {
            eprintln!("wspeed: {e}");
            exit(2)
        }
    }
}
