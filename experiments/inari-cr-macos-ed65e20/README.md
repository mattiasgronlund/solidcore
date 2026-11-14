# inari in solidcore: the macOS run at `ed65e20`

This is the macOS run at the commits
[the issue fixed for all further measurements](https://github.com/mattiasgronlund/inari/issues/2#issuecomment-6865592034),
for [mattiasgronlund/inari#2](https://github.com/mattiasgronlund/inari/issues/2). It is an
experiment, not a proposal: nothing here is meant for solidcore's `main`. The earlier run, at
`955f8c2ca`, stays on branch `exp/inari-cr-macos` as a record.

## Versions

| | |
|---|---|
| solidcore | `ed65e20dac563a0688a1eab9ee25fdd1fc02d98e`, this branch's base |
| inari | main `6a9fa7aad2a3ed0551e01239dac94ae149773d1c`, and `ca6faf9c109ad62a7e53afc598d59f74af92adeb` |
| rustc | 1.99.1 (58cf8f9c3 2026-10-13), the toolchain `rust-toolchain.toml` pins |
| Machine | Apple M-series (arm64), 10 cores, macOS (Darwin 25.6) |
| `RUSTFLAGS` | none |
| Goldens | the `goldens` profile, `--all --jobs 4`, the pinned upstream cloned into `upstream/` as CI does |

## The variants

The experiment is two commits on `ed65e20`. Cargo features choose what each build of the
goldens binary contains. `scripts/build-variants.sh` built every variant into
`bin/variants/<name>`.

| Variant | Features | What it is |
|---|---|---|
| `base` | none | solidcore unmodified (Apple's libm) |
| `cr-main` | `cr` | every libm call of the binary through `inari::cr::*_rn` (`crates/goldens/src/cr_interpose.rs`), inari main |
| `cr-ca6` | `cr` | the same, built with the inari rev in `crates/goldens/Cargo.toml` set to `ca6faf9` |
| `acos-args` | `acos-args` | `cr-main`, also recording the argument of every 10,000th `acos` call |
| `ivl-count` | `ivl-count` | `base`, counting `Ivl`'s operations |
| `ivl-inari` | `ivl-inari` | `Ivl`'s `+ - * /`, `sqrt` and the `*_ulp` forms computed with `inari::Interval` (aarch64 backend) |
| `ivl-generic` | `ivl-inari-generic` | the same with inari's `force-generic` |
| `ivl-inari-count` | `ivl-count,ivl-inari` | `ivl-inari`, counting |

- **The shim** is the earlier run's, behind a feature. The workspace lints are dropped from
  `crates/goldens/Cargo.toml` for its `unsafe`.
- **`Ivl` through inari** (`crates/solidcore/src/blend/fillet_section/interval.rs`, module
  `experiment`):
  - each operation converts both operands to `inari::Interval`, operates, and converts back;
  - `Ivl`'s own rules are kept: a divisor holding 0 gives the whole line, a NaN end gives the
    whole line, and the `*_ulp` forms give the whole line for a non-finite operand;
  - `Ivl::checked`, which widens results computed in plain `f64` elsewhere, is unchanged.

## How the runs were made

All runs were on 2026-11-13 and 2026-11-14, one after another, with nothing else of the
experiment running. `results/timed-phases.log` and `results/units-load.log` record the load
average before and after each phase.

1. **Untimed runs** (`scripts/run-phase1.sh`): `ivl-count` and `acos-args` over all units.
2. **Whole runs, in this order** (`scripts/run-phase2.sh`): `base`, `cr-main`, `cr-ca6`, `base`
   again.
3. **Complete outputs** (`scripts/run-phase3.sh`, `run-phase4.sh`): every variant run with
   `--goldens --bless --goldens-dir <copy of expected>`.
4. **Units alone** (`scripts/run-units.sh`): two rounds, the three variants alternating within
   each round. Wall time is bash's `time`.
   - The first attempt, inside phases 2 and 4, recorded no times: its `/usr/bin/time` report
     went to `/dev/null` with the program's output.
   - The rerun waited for macOS's `installd`, which had been busy for two hours, to go idle.
5. **`ivl-inari-count`**: the seven units that use `Ivl`, untimed.

The scripts name the scratch directory they ran in (`../r-ed65e20` from the worktree).

## `results/`

| File | What it is |
|---|---|
| `goldens-{base-1,cr-main,cr-ca6,base-2}.log` | The four whole runs |
| `unit-times.tsv` | Each unit's `rust=` time in those four runs |
| `heavy-units.tsv` | Five heavy units alone: round, unit, variant, wall seconds |
| `ivl-units.tsv` | The same for the five units with the most `Ivl` operations, `base` against `ivl-inari` and `ivl-generic` |
| `ivl-counts.tsv` | `Ivl` operations per unit and kind, for `ivl-count` and `ivl-inari-count` |
| `bless-<variant>.log` | The runs that wrote the complete outputs |
| `changed-cases-cr-main.jsonl` | Every case whose `cr-main` output differs bit for bit from the committed golden (blessed on Linux x86-64), with the `golden`, `apple_libm` (`base`) and `cr` records |
| `changed-cases-apple-libm.jsonl` | The same for `base` |
| `changed-cases-ivl-inari.jsonl` | Every case where `ivl-inari` differs from `base`, bit for bit. Here `golden` and `apple_libm` both hold `base`'s record and `cr` holds `ivl-inari`'s |
| `acos-args.tsv` | The `acos` arguments: unit, call-site bucket, the argument's bits as 16 hex digits |
| `acos-args-summary.tsv` | Per bucket: count, share, share negative, and a histogram of `1 − |x|` by decade (percent of the bucket) |
| `call-counts.tsv` | libm calls per function in the `acos-args` run |
| `goldens-ivl-count.log`, `goldens-acos-args.log` | The untimed runs |
| `timed-phases.log`, `units-load.log` | The run scripts' logs with load averages |

`cr-ca6`'s complete outputs are identical to `cr-main`'s, and `ivl-generic`'s to `ivl-inari`'s,
so neither has a file of its own. The complete outputs themselves (about 140 MB per variant)
are not committed.

**Buckets in `acos-args.tsv`.** A bucket is assigned from the first solidcore frame of the
call's stack, below the `vector3d::acos` wrapper:

| Bucket | Frame |
|---|---|
| `net` | `nurbs_srf::compute_net_constants` |
| `polar_box` | `PolarBox::compute_polar_coord` |
| `curve_cache` | `curve_cache::compute_constants` |
| `angle_between` | `Vec3::angle_between` |
| `other` | any other frame |

The arguments are calls 9,999, 19,999, and so on of each process.

## The Linux micro programs on macOS (item 3)

On 2026-11-14, the programs in `experiments/inari-cr-linux/cr-eval/` (inari main) and
`cr-eval-ca6/` (inari `ca6faf9`) of branch `exp/inari-cr-linux-ed65e20` (`683ba4814`) ran
on this machine (Apple M1 Pro, macOS 26.6.2), with rustc 1.99.1.

- **Builds:** `cargo build --release --locked --bin speed --bin interval`, in two variants:
  `aarch64` (inari's default backend) and `generic` (`--features force-generic`).
- **Not built:** `accuracy`. It fails to link here: the GMP and MPFR that `gmp-mpfr-sys`
  builds for `rug` are built for macOS 26.0, and the link targets 11.0. Item 3 doesn't use it.
- **Runs:** `scripts/run-micro.sh`. Each program runs alone, one after another: `speed`,
  `speed --acos-args results/acos-args.tsv` and `interval`, for each build. macOS has no
  `taskset`, so nothing is pinned. The whole set ran twice, in `results/micro/round-1/`
  and `round-2/`. `timing.log` in each records the load average before and after.
  - A browser tab used about 0.6 of one of the ten cores throughout.
- **Files:** `speed-<commit>-<build>.tsv`, `speed-acos-args-<commit>-<build>.tsv` and
  `interval-<commit>-<build>.txt`, with commit `main` or `ca6` and build `aarch64` or
  `generic`. In `speed`, the `glibc_ns` column is Apple's libm.
