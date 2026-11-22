#!/usr/bin/env bash
# usage: build-det-variants.sh [variant...]   (default: det-libm det-cr)
# Builds the goldens binary of the PR tree $D (solidcore's `math::det` on the `libm` crate,
# =0.2.16) in two variants, into $S/bins/goldens-<variant>:
#   det-libm  the tree as the PR has it: `math::det` through the libm crate
#   det-cr    feature det-cr: `math::det` through fushimi's (inari's) `cr::*_rn` at
#             $NEW_COMMIT, default features and no force-generic, so its kernel picks its FMA
#             copy at run time; `exp_m1`, `ln_1p` and `powi` stay as they are
# $D carries patches/det-cr-ebd45f3.patch. cargo runs in $D, so the tree's own toolchain pin
# (1.98.0) applies. Beside each binary, <binary>.libm-imports lists the math functions it
# still imports from the C libm (std's own calls outside `math::det`).
set -eu
. "$(dirname "$0")/env.sh"
grep -q "rev = \"$NEW_COMMIT\"" "$D/crates/solidcore/Cargo.toml" \
  || { echo "$D's solidcore does not depend on fushimi $NEW_COMMIT"; exit 1; }
mkdir -p "$S/bins"
b() { # variant, features (empty for none)
  echo "=== goldens-$1"
  (cd "$D" && nice -n 10 taskset -c "$CPUS" cargo build --locked --profile goldens -p goldens ${2:+--features "$2"})
  cp "$D/bin/goldens/goldens" "$S/bins/goldens-$1"
  libm_imports "$S/bins/goldens-$1"
}
[ $# -eq 0 ] && set -- det-libm det-cr
for v in "$@"; do
  case $v in
    det-libm) b det-libm "" ;;
    det-cr) b det-cr det-cr ;;
    *) echo "unknown variant $v"; exit 1 ;;
  esac
done
echo "build-det-variants $* done"
