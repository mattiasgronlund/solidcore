#!/usr/bin/env bash
# usage: build-det-variants.sh [variant...]   (default: det-libm det-cr)
# Builds the goldens binary of the PR tree $D (solidcore's `math::det` on the `libm` crate,
# =0.2.16) into $S/bins/goldens-<variant>:
#   det-libm  the tree as the PR has it: `math::det` through the libm crate
#   det-cr    feature det-cr: `math::det` through fushimi's (inari's) `cr::*_rn` at
#             $NEW_COMMIT, default features and no force-generic, so its kernel picks its FMA
#             copy at run time; `exp_m1`, `ln_1p` and `powi` stay as they are
#   det-cr-010
#             the same at fushimi $V010 from crates.io
#   det-libm-wasm, det-cr-010-wasm
#             det-libm and det-cr-010 for wasm32-wasip1, linked with the 8 MiB stack solidcore's
#             README asks wasm consumers for, into $S/bins/goldens-<variant>.wasm, and
#             precompiled for wasmtime into goldens-<variant>.cwasm. On wasm32, fushimi has no
#             FMA copy: its kernel's fast paths use Dekker's product.
# $D carries patches/det-cr-ebd45f3.patch for det-cr, patches/det-cr-ebd45f3-fushimi-0.1.0.patch
# for the 010 variants (det-libm builds from either), and the wasm ones also
# patches/goldens-wasm32-ebd45f3.patch (the goldens harness's one constant that overflows a
# 32-bit usize; 64-bit builds are unchanged by it). cargo runs in $D, so the tree's own
# toolchain pin (1.98.0) applies. Beside each native binary, <binary>.libm-imports lists the
# math functions it still imports from the C libm (std's own calls outside `math::det`).
set -eu
. "$(dirname "$0")/env.sh"
need_dep() { # what D's solidcore must depend on, as a fixed string of its line
  grep -qF "$1" "$D/crates/solidcore/Cargo.toml" \
    || { echo "$D's solidcore has no '$1' in its fushimi line"; exit 1; }
}
mkdir -p "$S/bins"
b() { # variant, features (empty for none)
  echo "=== goldens-$1"
  (cd "$D" && nice -n 10 taskset -c "$CPUS" cargo build --locked --profile goldens -p goldens ${2:+--features "$2"})
  cp "$D/bin/goldens/goldens" "$S/bins/goldens-$1"
  libm_imports "$S/bins/goldens-$1"
}
bw() { # variant, features (empty for none)
  echo "=== goldens-$1 (wasm32-wasip1)"
  (cd "$D" && RUSTFLAGS="-C link-arg=-zstack-size=8388608" nice -n 10 taskset -c "$CPUS" \
    cargo build --locked --profile goldens -p goldens --target wasm32-wasip1 ${2:+--features "$2"})
  cp "$D/bin/wasm32-wasip1/goldens/goldens.wasm" "$S/bins/goldens-$1.wasm"
  nice -n 10 "$WASMTIME" compile -o "$S/bins/goldens-$1.cwasm" "$S/bins/goldens-$1.wasm"
}
[ $# -eq 0 ] && set -- det-libm det-cr
for v in "$@"; do
  case $v in
    det-libm) b det-libm "" ;;
    det-cr) need_dep "rev = \"$NEW_COMMIT\""; b det-cr det-cr ;;
    det-cr-010) need_dep "version = \"=$V010\""; b det-cr-010 det-cr ;;
    det-libm-wasm) bw det-libm-wasm "" ;;
    det-cr-010-wasm) need_dep "version = \"=$V010\""; bw det-cr-010-wasm det-cr ;;
    *) echo "unknown variant $v"; exit 1 ;;
  esac
done
echo "build-det-variants $* done"
