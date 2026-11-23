#!/usr/bin/env bash
# usage: build-wasm-bench.sh [config...]   (default: all five)
# Builds cr-wasm/ (cr-eval's speed program without MPFR, at fushimi $V010) into
# $S/bins/wasm-<config>/:
#   native       the default x86-64 target: wspeed, to compare its hashes and times
#   wasip1       wasm32-wasip1: wspeed.wasm, and wspeed.cwasm precompiled for wasmtime
#   wasip1-simd  the same with +simd128
#   unknown      wasm32-unknown-unknown, as SolidCraft's web app is built (no extra target
#                features): cr_wasm.wasm, driven by scripts/wasm-drive.mjs
#   unknown-simd the same with +simd128
# std's math on each target: glibc natively, wasi-libc (musl's C code) on wasip1,
# compiler-builtins' copy of the libm crate on unknown-unknown.
set -eu
. "$(dirname "$0")/env.sh"
crate=$E/cr-wasm
[ $# -eq 0 ] && set -- native wasip1 wasip1-simd unknown unknown-simd
for cfg in "$@"; do
  t=$S/t/wasm-$cfg
  case $cfg in
    native) target=x86_64-unknown-linux-gnu; flags=""; what="--bin wspeed" ;;
    wasip1) target=wasm32-wasip1; flags=""; what="--bin wspeed" ;;
    wasip1-simd) target=wasm32-wasip1; flags="-Ctarget-feature=+simd128"; what="--bin wspeed" ;;
    unknown) target=wasm32-unknown-unknown; flags=""; what="--lib" ;;
    unknown-simd) target=wasm32-unknown-unknown; flags="-Ctarget-feature=+simd128"; what="--lib" ;;
    *) echo "unknown config $cfg"; exit 1 ;;
  esac
  echo "=== wasm-$cfg"
  (cd "$crate" && RUSTFLAGS="$flags" CARGO_BUILD_BUILD_DIR=$t/build nice -n 10 taskset -c "$CPUS" \
    cargo +1.99.1 build --release --locked --target "$target" --target-dir "$t" $what)
  o=$S/bins/wasm-$cfg
  mkdir -p "$o"
  case $cfg in
    native) cp "$t/$target/release/wspeed" "$o/" ;;
    wasip1*) cp "$t/$target/release/wspeed.wasm" "$o/"
             nice -n 10 "$WASMTIME" compile -o "$o/wspeed.cwasm" "$o/wspeed.wasm" ;;
    unknown*) cp "$t/$target/release/cr_wasm.wasm" "$o/" ;;
  esac
done
echo "build-wasm-bench done"
