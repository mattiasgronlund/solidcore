# inari `cr` as solidcore's libm: the Linux x86-64 run

This directory holds the harness and raw results of the Linux run of the evaluation in
[mattiasgronlund/inari#2](https://github.com/mattiasgronlund/inari/issues/2). It was rebuilt
from the issue's description, before the macOS run's harness was pushed to branch
[`exp/inari-cr-macos`](https://github.com/mattiasgronlund/solidcore/tree/exp/inari-cr-macos)
(solidcore `955f8c2ca`). It is an experiment branch: there is no pull request, and none is
planned.

These are the results at solidcore `dc5cfd7e7`, kept as a record. The two platforms used
different solidcore commits, so their goldens results are not compared. Both platforms
measure everything again at solidcore `ed65e20` with inari `6a9fa7a` and `ca6faf9`
([the issue's comment](https://github.com/mattiasgronlund/inari/issues/2#issuecomment-6865592034)),
and those results are pushed separately.

## Setup

- Host: AMD Ryzen 9 3900X (Zen 2, 24 threads), Linux 7.0, Ubuntu glibc 2.39. Governor
  `powersave`, boost on.
- rustc 1.99.1, the toolchain solidcore pins.
- solidcore `dc5cfd7e7`, the parent of this branch's commit.
- inari at two commits of [mattiasgronlund/inari](https://github.com/mattiasgronlund/inari):
  - `174092e` (main), which adds an f64 first stage for `atan`, `atan2`, `asin` and `acos`;
  - `ca6faf9`, the macOS run's commit.
- The run built inari from local clones of these commits. The manifests here name the same
  commits as git dependencies, so the directory builds without the clones.

## Layout

- `cr-libm-shim/`: `#[no_mangle] extern "C"` definitions of `sin cos tan asin acos atan
  sinh cosh tanh asinh acosh atanh exp exp2 log log2 log10 cbrt atan2 pow hypot sincos`,
  each calling `inari::cr::*_rn`.
  - On Linux, LLVM fuses a `sin` and a `cos` of the same argument into glibc's `sincos`, the
    counterpart of Apple's `__sincos_stret`.
  - `link()` keeps the definitions in the link. The goldens binary calls it at the top of
    `main` when the `cr-libm` feature is on.
  - Feature `count`: every call is counted, and at exit each process writes its counts to
    `$CR_COUNT_DIR/<pid>.tsv`.
- `crates/goldens` (outside this directory): an optional dependency on the shim, with
  features `cr-libm`, `cr-libm-generic` (adds inari's `force-generic`, so it builds on the
  default x86-64 target) and `cr-libm-count`.
- `cr-eval/` (inari `174092e`) and `cr-eval-ca6/` (inari `ca6faf9`; its `src` is a symlink to
  `cr-eval/src`): three programs.
  - `accuracy`: glibc (through std) and `cr` against MPFR (`rug`), 200,000 random arguments
    per row.
  - `speed`: ns per call, glibc against `cr`. Independent calls through a function pointer,
    summed into an accumulator, 2,000,000 arguments per row, median of 5 passes.
  - `interval`: a 3-vector dot product of point intervals, building the intervals included,
    best of 7 passes. It times plain `f64`, solidcore's `Ivl` and `inari::Interval`. `src/ivl.rs`
    is solidcore's `Ivl`, copied from `crates/solidcore/src/blend/fillet_section/interval.rs`.
- `scripts/`: the scripts as they ran. They name the original work directory (`S`, a scratch
  directory) and worktree (`W`) by absolute path; see "Rerunning".
  - `build-eval.sh`: the eval programs, for both inari commits, in three builds each:
    - `gen-default`: the default target with `force-generic`, what solidcore's CI would build
      (its portable backend has no FMA on this target);
    - `native`: `-Ctarget-cpu=native`, inari's default (`fma`) backend;
    - `gen-native`: `-Ctarget-cpu=native` with `force-generic`.
  - `build-goldens-variants.sh`: the goldens binary with the shim, in three variants:
    `cr-count` (inari main, counting), `cr-main` and `cr-ca6`. All three use the default
    target with `force-generic`.
  - `check-run.sh <variant>`: one untimed full goldens run.
  - `timing.sh [micro|whole|heavy|all]` and `timed-run.sh`: the timed runs.
    `timed-run.sh` runs `timing.sh` with a sampler that logs the load and the number of other
    build processes every 10 s.
- `results/`: see below.

## Results

- `accuracy-<commit>-<build>.tsv`: misrounding against MPFR, per row. The same in all six
  builds.
- `imports-baseline.txt`: the math functions the unmodified goldens binary imports from glibc.
  The `cr` builds import none of them.
- `check-cr-count.log`: the full goldens run with every libm call through `cr` (12 units fail,
  48 cases change, 0 broken). `cr-changes.txt` lists the changed values from it.
  - The goldens log lists at most ten changed cases per unit, so 15 of `surface.volume_p2`'s
    25 are not in it. The ten listed all go from 0 to 8.9e-16.
- `call-counts.tsv`: libm calls per function in that run, summed over its processes.
- `timing/`: the timed runs, on a quiet host, with `load.log` beside them.
  - `speed-*.tsv` and `interval-*.txt`: the micro benchmarks, per build, pinned to CPU 5.
  - `whole-<variant>-<round>/`: full goldens runs at `--jobs 4` (`stdout.log`, and the
    per-unit times in `times.tsv`).
  - `heavy-<unit>-<variant>-<round>.log`: five heavy units, pinned to CPU 5.
  - `timing.log`: start and end of every step, and wall times.
  - `aborted-1321-whole-baseline-1/`: a run stopped when another job loaded the host. It is
    kept only as a record; don't use its times.

The timed goldens runs, in short. All three binaries use the default target with
`force-generic`. Whole runs went one after another: unmodified, `cr` at inari main,
`cr` at `ca6faf9`, then unmodified again.

| Whole run, 283 units, `--jobs 4` | Unmodified (mean of 2) | `cr`, inari main | `cr`, `ca6faf9` |
|---|---|---|---|
| Wall time | 509.6 s | 519.5 s (+1.9%) | 527.3 s (+3.5%) |
| Units' times added up | 1,945.4 s | 1,976.1 s (+1.6%) | 2,002.1 s (+2.9%) |

| Heavy unit, alone, pinned to CPU 5 (mean of 2 rounds) | Unmodified | `cr`, inari main | `cr`, `ca6faf9` |
|---|---|---|---|
| `corpus.parity` | 91.6 s | 93.2 s (+1.7%) | 94.1 s (+2.8%) |
| `offset.offset_executive_p2` | 74.6 s | 78.4 s (+5.1%) | 79.9 s (+7.1%) |
| `blend.fillet_executive_p3` | 54.2 s | 56.1 s (+3.5%) | 57.2 s (+5.7%) |
| `blend.feature_executive` | 19.3 s | 19.9 s (+3.1%) | 20.4 s (+5.6%) |
| `surface.surface_p7` | 7.15 s | 9.39 s (+31%) | 10.09 s (+41%) |

`blend.fillet_executive_p3` exits 1 in the `cr` runs: it holds 5 of the 48 changed cases.

## Notes

- Run the goldens binary from the worktree root: some units read `upstream/TestFiles/...`
  relative to the current directory (`upstream/` is solidcore's pinned clone, see
  `scripts/upstream-pin.env`).
- The `goldens` profile builds every dependency, inari included, with `overflow-checks` and
  `debug-assertions`. The eval programs use a plain release profile (`lto = "fat"`,
  `codegen-units = 1`).
- glibc's `exp`, `log` and `pow` are versioned `@GLIBC_2.29`. The shim's unversioned
  definitions still win at static link time (checked with `nm`).

## Rerunning

1. Build the goldens variants: `cargo build --profile goldens -p goldens --features
   cr-libm-generic` (or `cr-libm-count`); without a feature, the unmodified binary. For the
   `ca6faf9` variant, change the shim's `rev`.
2. Build the eval programs: `cargo +1.99.1 build --release` in `cr-eval/` and `cr-eval-ca6/`,
   with `--features force-generic` and `RUSTFLAGS` as listed above.
3. To use the scripts, set the `S=` and `W=` lines to your own paths. `S` must hold `bins/`
   with the built binaries, named as the scripts expect.
