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
  - `build-goldens-variants.sh [variant...]`: the goldens binary in four variants:
    `baseline` (unmodified), `cr-count` (inari main, counting), `cr-main` and `cr-ca6`. All
    four use the default target; the `cr` ones use `force-generic`.
  - `check-run.sh <variant>`: one untimed full goldens run. It writes every case's complete
    output with `--bless --goldens-dir <copy of expected>`; for `cr-count`, it writes the call
    counts instead.
  - `timing.sh [micro|whole|heavy|all]` and `timed-run.sh`: the timed runs. `timed-run.sh`
    runs `timing.sh` with a sampler that logs the load and the number of other build
    processes every 10 s.
  - `changed-cases.py`: from the blessed outputs, every case that differs from its golden
    bit for bit, in the macOS run's schema (`glibc` in place of `apple_libm`).
  - `call-counts.py`, `summarize-timing.py` and `compare-macos.py`: the tables below.
- `results/`: the raw results, described in [Results](#results-2026-11-14).

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
  `scripts/upstream-pin.env`). A new worktree has no `upstream/`: clone it at the pin first,
  or those units break in milliseconds.
- The `goldens` profile builds every dependency, inari included, with `overflow-checks` and
  `debug-assertions`. The eval programs use a plain release profile (`lto = "fat"`,
  `codegen-units = 1`).
- glibc's `exp`, `log` and `pow` are versioned `@GLIBC_2.29`. The shim's unversioned
  definitions still win at static link time (checked with `nm`).

## Results (2026-11-14)

**When.**
- The micro programs ran 12:33–12:35, and the whole runs and heavy units 12:48–13:49.
  The untimed runs followed, 13:49–14:25.
- The sampler (`results/timing/load.log`) saw no other build process during the timed
  runs. Their load averages, in `timing.log`, come from the runs themselves (`--jobs 4`).
- A first pass of the whole runs and heavy units ran without `upstream/` and was discarded.
  `timing.log` marks it with a `NOTE` line. The micro programs ran in that pass too; they
  read no upstream file and are kept.

### The outputs

- **`cr` at inari main and at `ca6faf9`:** identical complete outputs, for every case of
  every unit (`diff -r` of the blessed outputs).
- **`cr` on Linux and on macOS:** in both runs, the same 1,200 cases differ from the golden
  bit for bit, and their `cr` records are equal (`results/compare-macos.txt`; macOS:
  `exp/inari-cr-macos-ed65e20`). So `cr` gives the same output, bit for bit, on both
  platforms for all 591,934 cases of the 283 units. On Linux it ran on the default x86-64
  target with `force-generic`, on macOS with inari's aarch64 backend.
- **Against the goldens:** `cr` fails 12 units with 47 changed cases. macOS reported the same
  12 units and 47 cases. `check-cr-main.log` lists all 47, including the 25 of
  `surface.volume_p2`.
- **The platform libms:**
  - glibc's output differs from the golden bit for bit in 261 cases of 23 units. All are
    within the goldens' tolerance: the tool passes every unit.
    - 4 of them (3 in `surface.bend_volume`, 1 in `topology.brep_cutting`) are beyond 1e-9
      relative in the per-number check of `beyond_1e-9_relative`.
    - In each, the number is a sum in a long array's stored block (`blocks[i][0]` or
      `[2]`), which can cancel. The tool compares such a sum relative to the block's
      absolute sum (`crates/goldens/src/golden.rs`), and there it agrees.
  - Apple's libm differs from the golden in 9,624 cases.
  - glibc and Apple's libm give different outputs in 9,404 cases.
- `results/changed-cases-{glibc,cr,cr-ca6}.jsonl` hold the records. The `cr` and `cr-ca6`
  files are identical.

### Calls to libm

`results/call-counts.tsv`, from one full run with `cr-count`:

| Function | Calls | | Function | Calls |
|---|---|---|---|---|
| `acos` | 422,703,508 | | `pow` | 20,818,865 |
| `sincos` | 135,328,173 | | `sin` | 664,227 |
| `tan` | 63,464,664 | | `log10` | 542,172 |
| `asin` | 57,930,483 | | `hypot` | 133,142 |
| `atan2` | 56,250,367 | | the other 7 | 185,842 |
| `cos` | 22,715,314 | | **total** | **780,736,757** |

macOS counted 780,698,537 calls. Its counts per function match these to within 0.1%,
except `sin` (686,770) and `sinh` (18,833 against 11,702 here). The outputs are identical, so
these are differences in which calls reach libm on each target, not in what is computed.

### Whole runs

`--all --goldens --jobs 4`, in the order shown (`results/timing-summary.tsv`):

| Run | Wall | Unit times added up | Units failing |
|---|---|---|---|
| Unmodified (glibc) | 519.4 s | 1,984.7 s | 0 |
| `cr`, inari main | 532.0 s (+2.4%) | 2,022.3 s (+2.1%) | 12 |
| `cr`, inari `ca6faf9` | 532.9 s (+2.6%) | 2,025.8 s (+2.2%) | 12 |
| Unmodified again | 519.2 s | 1,978.6 s | 0 |

The changes are against the mean of the two unmodified runs.

### The heavy units, each run alone

Pinned to one CPU. Two rounds, with the three variants alternating within each round. Wall
times in seconds, round 1 / round 2, and the change of the mean:

| Unit | Unmodified | `cr` at main | `cr` at `ca6faf9` |
|---|---|---|---|
| `corpus.parity` | 92.0 / 91.9 | 94.2 / 94.1 (+2.4%) | 93.7 / 94.0 (+2.1%) |
| `offset.offset_executive_p2` | 75.4 / 75.3 | 78.6 / 78.3 (+4.1%) | 79.9 / 79.4 (+5.7%) |
| `blend.fillet_executive_p3` | 55.8 / 55.7 | 58.0 / 57.9 (+3.9%) | 58.0 / 58.3 (+4.2%) |
| `blend.feature_executive` | 19.6 / 19.4 | 20.0 / 20.1 (+2.9%) | 20.2 / 20.2 (+3.6%) |
| `surface.surface_p7` | 6.88 / 6.91 | 9.01 / 9.04 (+30.9%) | 9.74 / 9.71 (+41.1%) |

`blend.fillet_executive_p3` exits 1 with `cr`: 4 of the 47 changed cases are in it.

### The micro programs

`results/timing/{speed,speed-acos-args,interval}-<commit>-<build>.*`. Each build ran alone,
pinned to CPU 5.

**`acos` on the macOS run's arguments** (`speed --acos-args`, 42,020 arguments; the `all`
row; glibc 9.3 ns in every build):

| Build | `cr` at main | `cr` at `ca6faf9` |
|---|---|---|
| `gen-default` | 59.4 ns (6.35×) | 74.9 ns (8.08×) |
| `native` | 44.2 ns (4.73×) | 58.1 ns (6.21×) |
| `gen-native` | 44.5 ns (4.75×) | 58.2 ns (6.20×) |

`net`, `polar_box` and `curve_cache`, 93% of the arguments, are within 3 ns of `all` in
every build (within 1.5 ns at main). `angle_between` and `other` are 6–21 ns cheaper.

**inari main against `ca6faf9`** (`speed`). Only these four functions change by more than
2.0 ns per call. The largest other change is `sin` and `cos` in `gen-default`, 1.8–2.0 ns
faster at `ca6faf9`.

| Function | `gen-default` main / `ca6` | `native` main / `ca6` |
|---|---|---|
| `acos` | 68.7 / 87.1 ns | 51.0 / 67.3 ns |
| `asin` | 63.2 / 81.2 ns | 47.9 / 63.1 ns |
| `atan` | 33.5 / 62.3 ns | 25.2 / 52.2 ns |
| `atan2` | 48.3 / 74.6 ns | 39.8 / 63.6 ns |

**`interval`** (a 3-vector dot product, building the intervals included; the same at both
inari commits to within 0.6 ns):

| Build | `f64` | solidcore `Ivl` | `inari::Interval` |
|---|---|---|---|
| `gen-default` | 0.7 ns | 23.6 ns | 84.8 ns |
| `native` | 0.7 ns | 20.0 ns | 52.2 ns |
| `gen-native` | 0.7 ns | 20.6 ns | 76.1 ns |
