#!/usr/bin/env bash
# Builds the goldens binary in four variants, into $S/bins/goldens-<variant>:
#   baseline  unmodified: every libm call goes to glibc
#   cr-count  every libm call through inari::cr at inari main, counting the calls
#   cr-main   the same without counting
#   cr-ca6    the same at inari ca6faf9
# All four use the default x86-64 target; the cr variants use force-generic, as solidcore's
# CI would have to. For cr-ca6 the shim's rev is switched to ca6faf9; on exit the rev and
# Cargo.lock are put back. Beside each binary, <binary>.libm-imports lists the math
# functions it still imports from libm (none, for the cr variants).
set -eu
. "$(dirname "$0")/env.sh"
shim=$E/cr-libm-shim/Cargo.toml
grep -q "rev = \"$INARI_MAIN\"" "$shim" || { echo "the shim is not at inari main"; exit 1; }
cp "$W/Cargo.lock" "$S/Cargo.lock.saved"
restore() {
  sed -i "s/rev = \"$INARI_CA6\"/rev = \"$INARI_MAIN\"/" "$shim"
  cp "$S/Cargo.lock.saved" "$W/Cargo.lock"
}
trap restore EXIT
mkdir -p "$S/bins"
libm='sin|cos|tan|asin|acos|atan|sinh|cosh|tanh|asinh|acosh|atanh|exp|exp2|expm1|log|log1p|log2|log10|cbrt|atan2|pow|hypot|sincos'
b() { # variant, features (empty for none)
  echo "=== goldens-$1"
  (cd "$W" && nice -n 10 taskset -c "$CPUS" cargo build --profile goldens -p goldens ${2:+--features "$2"})
  cp "$W/bin/goldens/goldens" "$S/bins/goldens-$1"
  nm -D --undefined-only "$S/bins/goldens-$1" | awk '{print $2}' | sed 's/@.*//' \
    | grep -xE "$libm" | sort > "$S/bins/goldens-$1.libm-imports" || true
  echo "goldens-$1 imports $(wc -l < "$S/bins/goldens-$1.libm-imports") math functions from libm"
}
b baseline ""
b cr-count cr-libm-count
b cr-main cr-libm-generic
sed -i "s/rev = \"$INARI_MAIN\"/rev = \"$INARI_CA6\"/" "$shim"
b cr-ca6 cr-libm-generic
echo "build-goldens-variants done"
