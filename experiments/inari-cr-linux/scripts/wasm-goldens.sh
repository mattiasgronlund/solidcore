#!/usr/bin/env bash
# The det variants' goldens on wasm32-wasip1 (build-det-variants.sh det-libm-wasm
# det-cr-010-wasm), one unit per process, since the harness's --jobs spawns child processes
# and wasip1 cannot.
#   wasm-goldens.sh check <variant> [engine]  not timed: every unit blessed into a fresh copy of
#                                             $D's goldens, $S/out/<variant>-<engine>, under
#                                             nice, 4 units at once
#   wasm-goldens.sh whole <variant> [engine]  timed: every unit --goldens, 4 at once on $CPUS,
#                                             into $S/results/timing/wasm-whole-<variant>-<engine>-<n>/
#   wasm-goldens.sh heavy <variant>...        timed: the heavy units pinned to CPU $PIN, two
#                                             rounds, the variants interleaved; engine $ENGINE
# Engines: wasmtime (Cranelift; the precompiled .cwasm, default) and node (V8, as in Chrome;
# the .wasm, compiled at each start).
# Units: every unit of `--list` but corpus.parity, which runs its sides in child processes.
# Each run's per-unit table, units.tsv, has the unit, its exit code and its wall seconds.
# Preopens: the harness finds its goldens and fixtures through the absolute path of the tree
# it was built in (CARGO_MANIFEST_DIR), so $D is preopened under its own path, and $D's
# `upstream` symlink, which points out of $D, through a preopen of its target at $D/upstream.
# Some units read upstream/TestFiles relative to the tree's root, so $D is also preopened as
# `.`, and the symlink's target as `upstream`.
# Stacks: $WASM_STACK bytes of host stack for wasm frames (wasmtime -W max-wasm-stack; node
# --stack-size, in KiB). The binaries' linear-memory stack is 8 MiB (build-det-variants.sh).
set -u
. "$(dirname "$0")/env.sh"
self=$(readlink -f "$0")
R=$S/results/timing
WASM_STACK=${WASM_STACK:-4194304}
UP=$(readlink -f "$D/upstream")
HEAVY_UNITS="offset.offset_executive_p2 blend.fillet_executive_p3 blend.feature_executive surface.surface_p7"

# engine variant [--dir HOST]... -- [harness args...]: runs the harness once.
engine() {
  local eng=$1 v=$2; shift 2
  local dirs=(--dir "$D" --dir "$UP::$D/upstream" --dir "$D::." --dir "$UP::upstream")
  while [ "$1" != -- ]; do dirs+=("$1" "$2"); shift 2; done
  shift
  case $eng in
    wasmtime) "$WASMTIME" run --allow-precompiled -W "max-wasm-stack=$WASM_STACK" "${dirs[@]}" \
                --env GOLDENS_MAX_LINES=1000000 "$S/bins/goldens-$v.cwasm" "$@" ;;
    node) "$NODE" --stack-size=$((WASM_STACK / 1024)) "$E/scripts/wasi-run.mjs" "${dirs[@]}" \
            --env GOLDENS_MAX_LINES=1000000 "$S/bins/goldens-$v.wasm" "$@" ;;
    *) echo "unknown engine $eng"; return 2 ;;
  esac
}

# one engine variant unit logdir [--dir HOST]... -- [harness args...]: one unit, timed;
# prints its units.tsv line.
one() {
  local eng=$1 v=$2 u=$3 logdir=$4; shift 4
  local start end code
  start=$(date +%s.%N)
  engine "$eng" "$v" "$@" --unit "$u" > "$logdir/$u.log" 2>&1
  code=$?
  end=$(date +%s.%N)
  printf '%s\t%s\t%s\n' "$u" "$code" "$(echo "$end - $start" | bc)"
}

units() { # the units, in --list order, from the variant's native twin
  local native=$S/bins/goldens-${1%-wasm}
  (cd "$D" && "$native" --list) | cut -f1 | grep -vx corpus.parity
}

mkdir -p "$R"
log() { echo "$(date +%T) $*" | tee -a "$R/timing.log"; }
snap() { log "load: $(cut -d' ' -f1-3 /proc/loadavg)  governor: $(cat /sys/devices/system/cpu/cpufreq/policy$PIN/scaling_governor)"; }

cd "$D"
mode=$1; shift
case $mode in
  one) one "$@" ;;
  check)
    v=$1; eng=${2:-wasmtime}
    out=$S/out/$v-$eng
    [ -e "$out" ] && { echo "$out exists; move it away first"; exit 1; }
    mkdir -p "$S/out" "$S/results/check-$v-$eng"
    cp -r "$D/crates/goldens/expected" "$out"
    start=$(date +%s.%N)
    units "$v" | nice -n 10 taskset -c "$CPUS" xargs -P 4 -I{} bash "$self" one "$eng" "$v" {} \
      "$S/results/check-$v-$eng" --dir "$out" -- --goldens --bless --goldens-dir "$out" \
      > "$S/results/check-$v-$eng/units.tsv"
    end=$(date +%s.%N)
    echo "check $v $eng wall=$(echo "$end - $start" | bc)s, units not exiting 0:"
    awk -F'\t' '$2 != 0' "$S/results/check-$v-$eng/units.tsv"
    ;;
  whole)
    v=$1; eng=${2:-wasmtime}
    n=$(ls "$R" | grep -c "^wasm-whole-$v-$eng-[0-9]*$"); n=$((n + 1))
    d=$R/wasm-whole-$v-$eng-$n
    mkdir -p "$d"
    snap
    log "wasm whole $v $eng round $n"
    start=$(date +%s.%N)
    units "$v" | taskset -c "$CPUS" xargs -P 4 -I{} bash "$self" one "$eng" "$v" {} "$d" -- --goldens \
      > "$d/units.tsv"
    end=$(date +%s.%N)
    log "wasm whole $v $eng round $n wall=$(echo "$end - $start" | bc) units_not_0=$(awk -F'\t' '$2 != 0' "$d/units.tsv" | wc -l)"
    ;;
  heavy)
    eng=${ENGINE:-wasmtime}
    d=$R/wasm-heavy-$eng
    mkdir -p "$d"
    snap
    for round in 1 2; do
      for u in $HEAVY_UNITS; do
        for v in "$@"; do
          mkdir -p "$d/$v-$round"
          line=$(taskset -c "$PIN" bash "$self" one "$eng" "$v" "$u" "$d/$v-$round" -- --goldens)
          echo "$line" >> "$d/$v-$round/units.tsv"
          log "wasm heavy $eng $u $v round $round exit=$(echo "$line" | cut -f2) wall=$(echo "$line" | cut -f3)"
        done
      done
    done
    snap
    ;;
  *) echo "usage: wasm-goldens.sh check|whole <variant> [engine] | heavy <variant>..."; exit 1 ;;
esac
