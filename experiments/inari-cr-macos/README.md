# inari `cr` in solidcore: the macOS run

The harness and raw results of the macOS evaluation in
[mattiasgronlund/inari#2](https://github.com/mattiasgronlund/inari/issues/2). An experiment, not a
proposal: nothing here is meant for solidcore's `main`.

## Versions

| | |
|---|---|
| solidcore | `955f8c2ca` (this branch's base) |
| inari | `ca6faf9c109ad62a7e53afc598d59f74af92adeb` |
| rustc | 1.101.0-nightly (c980177d8 2026-11-06) |
| Machine | Apple M-series (arm64), macOS (Darwin 25.6) |
| `RUSTFLAGS` | none |
| Features | none (inari's aarch64 backend) |
| Goldens | `cargo run --profile goldens -p goldens -- --all --jobs 4 --goldens`, the pinned upstream cloned into `upstream/` as CI does |

The full runs on 2026-11-12 used inari through a path dependency at the same commit. This
branch pins it as a git dependency; the source is identical.

## How `cr` is wired in

`crates/goldens/src/cr_interpose.rs` defines `#[no_mangle] extern "C"` functions for:

- `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `atan2`
- `sinh`, `cosh`, `tanh`, `asinh`, `acosh`, `atanh`
- `exp`, `exp2`, `log`, `log2`, `log10`, `pow`, `cbrt`, `hypot`
- `sincos` and `__sincos_stret`

Each calls `inari::cr::*_rn` and counts its calls.

- The static link resolves Rust std's libm references in the goldens binary to these definitions,
  so no kernel code changes.
- LLVM fuses a `sin` and a `cos` of the same argument into `__sincos_stret` on macOS, even
  when the source calls `sin` alone next to a `cos` (`bench/src/bin/interp.rs` shows it), so that
  symbol must be covered too.
- `log1p` and `expm1` are not covered, because `cr` has neither. Rust computes `asinh` and
  `acosh` through `log1p`, so solidcore's four calls to them still use Apple's libm.
- With `CR_COUNTS=<dir>`, each process writes its counts to `<dir>/<pid>.txt` when it exits.
- The workspace lints are dropped from `crates/goldens/Cargo.toml` for the shim's `unsafe`.

A later commit on this branch adds sampling of `acos` stack traces to the shim (every 100,000th
call). The timed runs and the goldens outputs come from the commit before it.

## `results/`

| File | What it is |
|---|---|
| `goldens-apple-libm.log` | The full goldens run on this commit, unmodified (Apple's libm), 2026-11-12. Each unit lists at most ten changed cases (`MAX_GOLDEN_LINES`) |
| `goldens-cr.log` | The same run with the shim |
| `unit-times.tsv` | Each unit's `rust=` time in the two runs above (`--jobs 4`) |
| `quiet-times.txt` | Five heavy units timed with `/usr/bin/time`, alternating Apple libm and `cr`, two rounds, nothing else running |
| `call-counts.tsv` | Calls per function in the `cr` run, summed over all processes |
| `call-counts-by-unit.tsv` | The same per unit, from the sampling run (`corpus.parity`'s child processes summed into one row) |
| `changed-cases-cr.jsonl` | Every case whose `cr` output differs from the committed golden bit for bit (1,367 cases in 78 units), one JSON object per line |
| `changed-cases-apple-libm.jsonl` | The same for Apple's libm (9,699 cases in 165 units) |
| `bless-apple-libm.log`, `bless-cr.log` | Logs of the 2026-11-13 runs that wrote the outputs behind the two files above |
| `bench-speed.txt`, `bench-speed-generic.txt` | What `bench` printed, with inari's aarch64 backend and with `force-generic` (2026-11-13 rerun) |
| `bench-mpfr-check.txt` | What `bench --bin check` printed |
| `bench-interp.txt` | What `bench --bin interp` printed (`2000.5 (2000.5, 3000.0)`: both calls reached the shim's `__sincos_stret`) |
| `acos-stacks.txt` | Sampled `acos` stack traces with their counts (sampling commit) |

**How the `changed-cases-*.jsonl` files were made.** Each configuration ran
`--goldens --bless --goldens-dir <copy of crates/goldens/expected>`, which rewrites every case's output
in the goldens' format. Each line then holds:

| Field | What it is |
|---|---|
| `unit`, `case` | The case |
| `golden` | The committed record, blessed on Linux x86-64 CI |
| `apple_libm`, `cr` | The two macOS records |
| `beyond_1e-9_relative` | Whether some number differs by more than 1e-9 relative, or anything else differs. Computed without the units' `abs_floors()`, so it flags 53 `cr` cases where the goldens flag 48 |
| `goldens_changed` | (`cr` file only) whether the goldens tool itself reported the case as changed in `goldens-cr.log`: the 48 |

The bless runs reproduced the earlier runs: the 48 cases match the ones the goldens tool listed.
In `bench-speed*.txt`, the `dot3 f64` time moved from 4.5 ns (2026-11-12) to 1.1 ns between
runs. That micro benchmark is not reliable; the issue's `Interval` table should not be used.

## `bench/`

A standalone crate, not a member of the solidcore workspace.

- **`cargo run --release`:** speed of Apple's libm against `cr` per function, and how often
  their results differ.
  - It also contains a 3-vector dot-product benchmark of point intervals. Its "widen-2u ivl"
    column is **not** solidcore's `Ivl`: it rounds to nearest and widens each result by
    `|x|·ε`, without the widening step of `blend/fillet_section/interval.rs`. The issue's
    "as in solidcore" label for it was wrong.
  - The `f64` and widened loops index slices from `chunks_exact`, so they time loop overhead
    as much as arithmetic.
- **`cargo run --release --bin check`:** misrounding against MPFR (through `rug`), 200,000
  arguments per function.
- **`cargo run --release --bin interp`:** shows that a `#[no_mangle]` definition replaces
  std's `sin`, and that `sin` alone next to a `sin_cos` becomes a `__sincos_stret` call.
- **`--features generic`:** builds inari with `force-generic`.
