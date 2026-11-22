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

## Rerun 2026-11-22

The rerun asked for in [mattiasgronlund/fushimi#2](https://github.com/mattiasgronlund/fushimi/issues/2)
(formerly inari#2: the crate is now `fushimi`). Raw results in `results-2026-11-22/`.

**Versions.**
- fushimi `e9e5ae57aff3a6cfe8ca6b8934d63eee2fee0715`, everywhere as
  `inari = { package = "fushimi", git = "https://github.com/mattiasgronlund/fushimi", rev = "e9e5ae5", default-features = false }`.
  `default-features = false` changes nothing: fushimi's default feature set is empty.
- inari main `6a9fa7a` and `ca6faf9`, as on Nov 14.
- solidcore `ed65e20` for the shim variants. For the det variants,
  [tangency-labs/solidcraft#15405](https://github.com/tangency-labs/solidcraft/pull/15405) at
  `ebd45f35d5`: `math::det` on the `libm` crate (`=0.2.16`), with its own goldens.
- rustc 1.99.1 for the shim variants and the eval programs; 1.98.0, the PR tree's pin, for the
  det variants.
- Host as on Nov 14: Ryzen 9 3900X, Linux 7.0.0-38, glibc 2.39.

**Variants.** Goldens binaries, all on the default x86-64 target:
- `cr-new`: feature `cr-libm` at fushimi `e9e5ae5`, without `force-generic`: fushimi's kernel
  picks its FMA copy at run time.
- `cr-new-gen`: the same with `force-generic` (`cr-libm-generic`, as `cr-main`).
- `cr-new-count`: counting, with `force-generic` (`cr-libm-count`).
- `det-libm`: the PR tree, unmodified (`scripts/build-det-variants.sh`).
- `det-cr`: the PR tree with `patches/det-cr-ebd45f3.patch`, feature `det-cr`: every function
  of `math::det` but `exp_m1`, `ln_1p` and `powi` calls `cr::*_rn` (fushimi `e9e5ae5`, no
  `force-generic`) in place of the `libm` crate.
- `baseline`, `cr-main`, `cr-ca6`, `cr-count`: the Nov 14 binaries.
- Only `baseline` imports math functions from the C libm (17; `goldens-*.libm-imports`).

The eval programs: `cr-eval-new/` (fushimi `e9e5ae5`; `src` is `cr-eval/src`) adds a fourth
build, `default`: the default target without `force-generic`, which main and `ca6faf9` don't
build; at `e9e5ae5` the kernel picks its FMA copy at run time. All ten builds were rebuilt:
`speed` and `accuracy` now also measure the `libm` crate (`=0.2.16`). `scripts/per-call.py`
writes `per-call.tsv`.

**When.**
- Builds and the untimed checks 03:11–04:05; the timed runs 04:06–06:21 (micro 04:06–04:08,
  whole runs 04:08–05:28, heavy units 05:28–06:21).
- The sampler (`timing/load.log`) saw 0 other builds in each of its 810 samples.

**Deviations.**
- `check-baseline.log` is truncated, with `exit=127`: `check-run.sh` was edited while the run
  was reading it. The run itself completed: all 283 units are in `check-times/baseline.tsv`,
  and its outputs are byte-identical to Nov 14's.
- The acos-args file was read from a copy (sha256 `fcd15ecf…3359`, 42,020 lines): the Nov 14
  path on the zram disk is gone.
- The Nov 14 `baseline`, `cr-main`, `cr-ca6` and `cr-count` binaries were reused, checked by
  sha256.
- The det tree's change is the patch file, not a commit: there is no solidcraft fork to push it
  to.

### The outputs, bit for bit

`identity.txt`: `diff -rq` of the blessed outputs (283 units and `known_failing.json`).

| Compared | Files that differ |
|---|---|
| `cr-new`, `cr-new-gen`, `cr-new-count`, Nov 14 `cr-main` and `cr-ca6` | 0 |
| `baseline` against Nov 14 `baseline` | 0 |
| `baseline` against `ed65e20`'s goldens | 23 |
| `cr-new` against `ed65e20`'s goldens | 75 |
| `det-libm` against #15405's goldens | 0 |
| `det-cr` against #15405's goldens | 167 |
| `ed65e20`'s goldens against #15405's | 186 |
| `det-cr` against `cr-new` | 76 |

So fushimi's FMA copy, its portable backend and inari at both commits give the same output for
every case.

### Changed cases

`changed-cases-summary.txt`: the cases that differ from their tree's goldens bit for bit.
Float-only: both records exist and differ only in numbers in `values`; other: the rest.

| Variant | Goldens | Cases | Units | Beyond 1e-9 rel. | Listed by the tool | Float-only | Other |
|---|---|---|---|---|---|---|---|
| glibc (`baseline`) | `ed65e20` | 261 | 23 | 4 | 0 | 261 | 0 |
| `cr-new` | `ed65e20` | 1,200 | 75 | 53 | 47 | 1,200 | 0 |
| `det-libm` | #15405 | 0 | 0 | 0 | 0 | 0 | 0 |
| `det-cr` | #15405 | 8,154 | 167 | 879 | 876 | 8,107 in 166 units | 47 in 10 units |

- `det-cr` moves 6.8× as many cases as the shim, in 167 units against 75.
- The libm crate misrounds more often than glibc (below), and the two trees differ:
  `det-libm`'s outputs differ from `baseline`'s in 11,695 cases of 185 units (counted from the
  blessed outputs, which are not committed). This run can't attribute `det-cr`'s count to
  either.

### Calls to libm

`cr-new-count`'s `call-counts.tsv` equals Nov 14's line for line: 780,736,757 calls,
422,703,508 of them `acos`.

### Whole runs

`--all --goldens --jobs 4`, in the order shown (`timing-summary.tsv`). Changes are against the
mean of the tree's two unmodified runs (`baseline` or `det-libm`); units fail against the tree's
goldens.

| Run | Wall | Unit times added up | Units failing |
|---|---|---|---|
| Unmodified (glibc) | 524.7 s | 1,996.8 s | 0 |
| `cr`, inari main | 534.8 s (+1.9%) | 2,035.0 s (+1.9%) | 12 |
| `cr`, inari `ca6faf9` | 537.8 s (+2.5%) | 2,046.8 s (+2.5%) | 12 |
| `cr-new` | 528.5 s (+0.7%) | 2,010.4 s (+0.7%) | 12 |
| `cr-new-gen` | 526.5 s (+0.3%) | 2,002.6 s (+0.3%) | 12 |
| Unmodified again | 524.9 s | 1,996.3 s | 0 |
| `det-libm` | 540.5 s | 2,008.7 s | 0 |
| `det-cr` | 535.7 s (−0.9%) | 1,988.8 s (−1.0%) | 71 |
| `det-libm` again | 540.4 s | 2,010.6 s | 0 |

`det-libm` against `baseline`: wall +3.0%, unit times +0.7%.

### The heavy units, each run alone

Pinned to one CPU, two rounds with the variants alternating. Seconds, round 1 / round 2, and
the change of the mean.

| Unit | Unmodified | `cr` at main | `cr` at `ca6faf9` | `cr-new` |
|---|---|---|---|---|
| `corpus.parity` | 93.72 / 93.57 | 95.11 / 94.63 (+1.3%) | 94.87 / 94.74 (+1.2%) | 94.33 / 94.88 (+1.0%) |
| `offset.offset_executive_p2` | 75.80 / 76.20 | 80.17 / 79.37 (+5.0%) | 80.26 / 79.84 (+5.3%) | 77.18 / 77.25 (+1.6%) |
| `blend.fillet_executive_p3` | 56.48 / 56.54 | 58.61 / 58.55 (+3.7%) | 58.96 / 58.86 (+4.3%) | 57.39 / 57.34 (+1.5%) |
| `blend.feature_executive` | 19.69 / 19.62 | 20.25 / 20.40 (+3.4%) | 20.35 / 20.37 (+3.6%) | 20.19 / 20.26 (+2.9%) |
| `surface.surface_p7` | 6.94 / 7.01 | 9.06 / 9.19 (+30.8%) | 9.83 / 9.85 (+41.0%) | 7.27 / 7.30 (+4.4%) |

| Unit | `det-libm` | `det-cr` | `det-libm` against unmodified |
|---|---|---|---|
| `corpus.parity` | 111.81 / 111.71 | 112.70 / 112.76 (+0.9%) | +19.3% |
| `offset.offset_executive_p2` | 75.82 / 75.99 | 76.49 / 76.77 (+1.0%) | −0.1% |
| `blend.fillet_executive_p3` | 58.78 / 58.58 | 58.96 / 58.57 (+0.2%) | +3.8% |
| `blend.feature_executive` | 20.25 / 20.25 | 19.98 / 20.10 (−1.1%) | +3.0% |
| `surface.surface_p7` | 7.26 / 7.20 | 7.62 / 7.66 (+5.7%) | +3.7% |

- `corpus.parity` is 19.3% slower on the PR tree as a whole (`det-libm` against `baseline`);
  `det-cr` adds 0.9% to that.
- `blend.fillet_executive_p3` exits 1 with every `cr` variant, `det-cr` included.

### The micro programs

`per-call.tsv`, from `timing/`. Each build alone, pinned to one CPU.

**ns per call** (`speed`). glibc and the libm crate as measured in the `default` build. At
`e9e5ae5`, `gen-native` is within 0.1 ns of `native`.

| Function | glibc | libm crate | `cr` `default` | `cr` `gen-default` | `cr` `native` | main `gen-default` | main `native` |
|---|---|---|---|---|---|---|---|
| `acos` | 16.2 | 11.2 | 23.0 | 30.1 | 24.0 | 69.3 | 50.9 |
| `asin` | 16.2 | 11.4 | 19.2 | 25.9 | 20.5 | 63.5 | 47.8 |
| `atan` | 11.5 | 10.6 | 23.2 | 33.1 | 25.2 | 33.4 | 25.2 |
| `atan2` | 23.7 | 25.4 | 37.3 | 48.4 | 40.4 | 48.3 | 39.6 |
| `sin` | 17.5 | 17.0 | 15.8 | 18.8 | 17.2 | 31.5 | 23.1 |
| `cos` | 16.3 | 16.1 | 16.7 | 18.6 | 17.8 | 31.8 | 22.8 |
| `tan` | 15.3 | 16.1 | 17.0 | 27.7 | 19.7 | 41.9 | 28.3 |
| `exp` | 5.0 | 9.7 | 8.5 | 11.1 | 8.5 | 14.4 | 8.9 |
| `ln` | 4.7 | 7.9 | 18.1 | 23.2 | 20.3 | 22.9 | 20.1 |
| `pow` | 13.9 | 45.0 | 36.9 | 48.2 | 36.9 | 50.3 | 38.3 |

- `atan` and `atan2` at `e9e5ae5` gain only in the `default` build: in `gen-default`,
  `native` and `gen-native` they are within 0.8 ns of inari main.
- `default` beats `native` for 8 of the 10 functions, by 1.0–3.1 ns; `exp` and `pow` are equal.
- In `default`, `cr` is faster than the libm crate for `sin`, `exp` and `pow`, within 5% for
  `cos` and `tan`, and 1.5–2.3× slower for the rest.

**`acos` on the macOS run's arguments** (`speed --acos-args`, 42,020 arguments; the `all` row).
glibc 9.3–9.5 ns and the libm crate 6.1–6.3 ns in every build:

| Build | `cr` at main | `cr` at `ca6faf9` | `cr` at `e9e5ae5` |
|---|---|---|---|
| `gen-default` | 59.1 ns (6.32×) | 75.7 ns (8.09×) | 23.0 ns (2.45×) |
| `native` | 44.3 ns (4.73×) | 58.2 ns (6.19×) | 16.0 ns (1.69×) |
| `gen-native` | 44.4 ns (4.74×) | 58.0 ns (6.20×) | 16.0 ns (1.70×) |
| `default` | | | 15.9 ns (1.68×) |

The ratios are to glibc. In `default`, `cr` takes 2.53× the libm crate's time.

**`interval`** (`f64` and `Ivl` from the `e9e5ae5` builds):

| Build | `f64` | solidcore `Ivl` | `Interval` at main | at `ca6faf9` | at `e9e5ae5` |
|---|---|---|---|---|---|
| `gen-default` | 0.7 ns | 22.3 ns | 84.8 ns | 85.2 ns | 93.6 ns |
| `native` | 0.7 ns | 20.1 ns | 52.4 ns | 52.3 ns | 50.6 ns |
| `gen-native` | 0.7 ns | 20.0 ns | 75.4 ns | 75.8 ns | 87.6 ns |
| `default` | 0.7 ns | 22.3 ns | | | 80.8 ns |

`inari::Interval` got slower with `force-generic` at `e9e5ae5`: +8.8 ns in `gen-default`, +12.2
ns in `gen-native`, against main.

### Accuracy

`accuracy-new-default.tsv` and `accuracy-main-gen-default.tsv` are identical: 200,000 random
arguments per row, against MPFR. `cr` misrounds none in either. The share of arguments
misrounded, and the largest error in ulps:

| Function | Range | glibc | libm crate | Max ulps glibc / libm |
|---|---|---|---|---|
| `sin` | [-7, 7] | 0.152% | 3.066% | 1 / 1 |
| `cos` | [-7, 7] | 0.122% | 3.029% | 1 / 1 |
| `sin` | [-1e4, 1e4] | 0.148% | 3.106% | 1 / 1 |
| `tan` | [-1.5, 1.5] | 0.238% | 3.741% | 1 / 1 |
| `acos` | [-1, 1] | 0.072% | 7.703% | 1 / 1 |
| `asin` | [-1, 1] | 0.141% | 5.859% | 1 / 1 |
| `atan` | [-10, 10] | 0.093% | 6.121% | 1 / 1 |
| `atan2` | [-1, 1]² | 0.095% | 17.776% | 1 / 1 |
| `exp` | [-10, 10] | 0.088% | 9.705% | 1 / 1 |
| `ln` | [1e-3, 1e3] | 0.007% | 1.991% | 1 / 1 |
| `cos` | [-1e4, 1e4] | 0.128% | 3.079% | 1 / 1 |
| `pow` | [0.01, 100] × [-10, 10] | 0.077% | 9.807% | 1 / 1 |
| `hypot` | [-1e3, 1e3]² | 0.611% | 13.882% | 1 / 1 |
| `cbrt` | [-1e3, 1e3] | 0.000% | 0.000% | 0 / 0 |
| `exp2` | [-10, 10] | 0.101% | 0.036% | 1 / 1 |
| `log2` | [1e-3, 1e3] | 0.013% | 0.104% | 1 / 1 |
| `log10` | [1e-3, 1e3] | 3.876% | 0.118% | 2 / 1 |
| `sinh` | [-5, 5] | 26.056% | 26.062% | 2 / 2 |
| `cosh` | [-5, 5] | 21.687% | 22.354% | 1 / 1 |
| `tanh` | [-5, 5] | 11.065% | 7.901% | 2 / 2 |

- The libm crate misrounds more often than glibc in 16 of the 20 rows, less often for `exp2`,
  `log10` and `tanh`; neither misrounds `cbrt`.
- glibc's `sincos` equals its `sin` and `cos` on [-7, 7].
