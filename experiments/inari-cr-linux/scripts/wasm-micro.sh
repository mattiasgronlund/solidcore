#!/usr/bin/env bash
# usage: wasm-micro.sh hash|speed [run...]
# cr-wasm (build-wasm-bench.sh) on every target and engine. A run is <config>-<engine>:
#   native                      wspeed, the default x86-64 target
#   wasip1-wasmtime, wasip1-simd-wasmtime
#                               wspeed.cwasm under wasmtime
#   wasip1-node, wasip1-simd-node
#                               wspeed.wasm under node's WASI (V8)
#   unknown-node, unknown-simd-node
#                               cr_wasm.wasm driven by wasm-drive.mjs under node (V8), as
#                               SolidCraft's web build runs in Chrome
# hash:  not timed: `--hash` and `--acos-hash $ACOS_ARGS` of each run into $S/results/wasm-hash/;
#        cr's hashes must be equal in every run.
# speed: timed, only on a quiet host: each run alone, pinned to CPU $PIN, `speed` and
#        `--acos-args $ACOS_ARGS`, into $S/results/timing/wasm-micro/.
set -u
. "$(dirname "$0")/env.sh"
ACOS_ARGS=${ACOS_ARGS:?set ACOS_ARGS to the acos-args.tsv}
A=$(dirname "$(readlink -f "$ACOS_ARGS")")
mode=$1; shift
[ $# -eq 0 ] && set -- native wasip1-wasmtime wasip1-simd-wasmtime wasip1-node wasip1-simd-node \
  unknown-node unknown-simd-node
# run <run> <wspeed args...>: the program of <run> with wspeed's arguments.
run() {
  local r=$1; shift
  local cfg=${r%-*} eng=${r##*-}
  [ "$r" = native ] && { "$S/bins/wasm-native/wspeed" "$@"; return; }
  case $cfg-$eng in
    wasip1*-wasmtime) "$WASMTIME" run --allow-precompiled --dir "$A" "$S/bins/wasm-$cfg/wspeed.cwasm" "$@" ;;
    wasip1*-node) "$NODE" "$E/scripts/wasi-run.mjs" --dir "$A" "$S/bins/wasm-$cfg/wspeed.wasm" "$@" ;;
    unknown*-node)
      local m
      case ${1-} in
        --hash) m=hash ;;
        --acos-args) m=acos-args ;;
        --acos-hash) m=acos-hash ;;
        *) m=speed ;;
      esac
      [ "$m" = speed ] || shift
      "$NODE" "$E/scripts/wasm-drive.mjs" "$S/bins/wasm-$cfg/cr_wasm.wasm" "$m" "$@" ;;
    *) echo "unknown run $r"; return 2 ;;
  esac
}
case $mode in
  hash)
    o=$S/results/wasm-hash
    mkdir -p "$o"
    # This shell and what it starts: nice, on $CPUS.
    renice -n 10 -p $$ > /dev/null
    taskset -cp "$CPUS" $$ > /dev/null
    for r in "$@"; do
      echo "=== $r"
      run "$r" --hash > "$o/hash-$r.tsv" || echo "$r --hash failed"
      run "$r" --acos-hash "$ACOS_ARGS" > "$o/acos-hash-$r.tsv" || echo "$r --acos-hash failed"
    done
    ;;
  speed)
    R=$S/results/timing
    o=$R/wasm-micro
    mkdir -p "$o"
    log() { echo "$(date +%T) $*" | tee -a "$R/timing.log"; }
    # This shell and what it starts: on CPU $PIN alone.
    taskset -cp "$PIN" $$ > /dev/null
    for r in "$@"; do
      log "wasm micro speed $r"
      run "$r" > "$o/speed-$r.tsv" || log "$r speed failed"
      log "wasm micro acos-args $r"
      run "$r" --acos-args "$ACOS_ARGS" > "$o/speed-acos-args-$r.tsv" || log "$r acos-args failed"
    done
    log "wasm micro done"
    ;;
  *) echo "usage: wasm-micro.sh hash|speed [run...]"; exit 1 ;;
esac
