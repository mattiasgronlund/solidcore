# inari `cr` as solidcore's libm: the Linux x86-64 run, at solidcore `ed65e20`

This directory holds the harness, and in time the raw results, of the Linux run of the
evaluation in [mattiasgronlund/inari#2](https://github.com/mattiasgronlund/inari/issues/2), at
the commits both platforms now measure at
([the issue's comment](https://github.com/mattiasgronlund/inari/issues/2#issuecomment-6865592034)).
It is an experiment branch: there is no pull request, and none is planned.

- The first Linux run, at solidcore `dc5cfd7e7` with inari `174092e`, is kept as a record on
  branch [`exp/inari-cr-linux`](https://github.com/mattiasgronlund/solidcore/tree/exp/inari-cr-linux/experiments/inari-cr-linux).
- The macOS run's harness is on branch
  [`exp/inari-cr-macos`](https://github.com/mattiasgronlund/solidcore/tree/exp/inari-cr-macos).

## Setup

- solidcore `ed65e20dac563a0688a1eab9ee25fdd1fc02d98e`, the parent of this branch's first
  commit, so the branch's diff is the experiment alone.
- inari at two commits of [mattiasgronlund/inari](https://github.com/mattiasgronlund/inari):
  - `6a9fa7aad2a3ed0551e01239dac94ae149773d1c` (main);
  - `ca6faf9c109ad62a7e53afc598d59f74af92adeb`, the commit of the first macOS run.
- rustc 1.99.1, the toolchain solidcore pins.
- Host: AMD Ryzen 9 3900X (Zen 2, 24 threads), Linux 7.0, Ubuntu glibc 2.39. Governor
  `powersave`, boost on.

## Layout

- `cr-libm-shim/`: `#[no_mangle] extern "C"` definitions of `sin cos tan asin acos atan
  sinh cosh tanh asinh acosh atanh exp exp2 log log2 log10 cbrt atan2 pow hypot sincos`,
  each calling `inari::cr::*_rn`, pinned to inari main.
  - On Linux, LLVM fuses a `sin` and a `cos` of the same argument into glibc's `sincos`, the
    counterpart of Apple's `__sincos_stret`.
  - `link()` keeps the definitions in the link. The goldens binary calls it at the top of
    `main` when the `cr-libm` feature is on.
  - Feature `count`: every call is counted, and at exit each process writes its counts to
    `$CR_COUNT_DIR/<pid>.tsv`.
- `crates/goldens` (outside this directory):
  - an optional dependency on the shim, with features `cr-libm`, `cr-libm-generic` (adds
    inari's `force-generic`, so it builds on the default x86-64 target) and `cr-libm-count`;
  - `GOLDENS_MAX_LINES`: how many changed cases the log lists per unit (default 10, as
    before), so a run can list every one.
- `cr-eval/` (inari main) and `cr-eval-ca6/` (inari `ca6faf9`; its `src` is a symlink to
  `cr-eval/src`): three programs.
  - `accuracy`: the platform's libm (through std) and `cr` against MPFR (`rug`), 200,000
    random arguments per row.
  - `speed`: ns per call, the platform's libm against `cr`. Independent calls through a
    function pointer, summed into an accumulator, 2,000,000 arguments per row, median of 5
    passes. The `glibc_ns` column is whatever libm std links: Apple's libm on macOS.
  - `speed --acos-args <file>`: the same loop for `acos` and `acos_rn`, on the arguments in
    `file`. Each line holds three tab-separated columns: the unit, the call-site bucket, and
    the argument's bits as 16 hex digits. A first line that doesn't parse is taken as a
    header. It prints one row per bucket, in sorted order, then `all`. Each row's arguments
    are taken in file order, repeated until a pass makes at least 2,000,000 calls.
  - `interval`: a 3-vector dot product of point intervals, building the intervals included,
    best of 7 passes. It times plain `f64`, solidcore's `Ivl` and `inari::Interval`. `src/ivl.rs`
    is solidcore's `Ivl`, copied from `crates/solidcore/src/blend/fillet_section/interval.rs`.
- `scripts/`: building, the untimed runs and the timed runs. Each takes `S`, a work directory
  outside the tree, from the environment; `env.sh` lists the other settings.
  - `build-eval.sh`: the eval programs, for both inari commits, in three builds each, into
    `$S/bins/<main|ca6>-<build>/`:
    - `gen-default`: the default target with `force-generic`, what solidcore's CI would build
      (its portable backend has no FMA on this target);
    - `native`: `-Ctarget-cpu=native`, inari's default (`fma`) backend;
    - `gen-native`: `-Ctarget-cpu=native` with `force-generic`.
  - `build-goldens-variants.sh`: the goldens binary in four variants: `baseline`
    (unmodified), `cr-count` (inari main, counting), `cr-main` and `cr-ca6`. All four use the
    default target; the `cr` ones use `force-generic`.
  - `check-run.sh <variant>`: one untimed full goldens run. It writes every case's complete
    output with `--bless --goldens-dir <copy of expected>`; for `cr-count`, it writes the call
    counts instead.
  - `timing.sh [micro|whole|heavy|all]` and `timed-run.sh`: the timed runs. `timed-run.sh`
    runs `timing.sh` with a sampler that logs the load and the number of other build
    processes every 10 s.
- `results/`: added when the runs are done.

## Building the eval programs elsewhere

In `cr-eval/` and `cr-eval-ca6/`, `cargo build --release` (with `--features force-generic`
for the portable backend). The toolchain solidcore pins (1.99.1) applies inside the tree.
The first build compiles GMP and MPFR for `rug`. Then, with the binaries in
`target/release/`:
- `speed > speed-<commit>-<build>.tsv`
- `speed --acos-args acos-args.tsv > speed-acos-args-<commit>-<build>.tsv`
- `interval > interval-<commit>-<build>.txt`

## Notes

- Run the goldens binary from the tree's root: some units read `upstream/TestFiles/...`
  relative to the current directory (`upstream/` is solidcore's pinned clone, see
  `scripts/upstream-pin.env`).
- The `goldens` profile builds every dependency, inari included, with `overflow-checks` and
  `debug-assertions`. The eval programs use a plain release profile (`lto = "fat"`,
  `codegen-units = 1`).
- glibc's `exp`, `log` and `pow` are versioned `@GLIBC_2.29`. The shim's unversioned
  definitions still win at static link time (checked with `nm`).
