#!/usr/bin/env bash
# usage: build-goldens-variants.sh [variant...]   (default: all seven, in this order)
# Builds the goldens binary of $W in these variants, into $S/bins/goldens-<variant>:
#   baseline      unmodified: every libm call goes to glibc
#   cr-count      every libm call through inari::cr at inari main, counting the calls
#   cr-main       the same without counting
#   cr-ca6        the same at inari ca6faf9
#   cr-new        the same at fushimi $NEW_COMMIT, without force-generic: on the default
#                 target, its kernel picks its FMA copy at run time
#   cr-new-gen    fushimi $NEW_COMMIT with force-generic, as cr-main
#   cr-new-count  fushimi $NEW_COMMIT, counting (force-generic, as cr-count)
# All use the default x86-64 target. inari's main and ca6faf9 only build there with
# force-generic, so cr-main, cr-ca6 and the count variants use it.
# For cr-ca6 and the cr-new variants the shim's inari dependency line is replaced: its rev for
# ca6faf9, the whole line for fushimi (the package and URL change too). On exit the shim's
# Cargo.toml and $W/Cargo.lock are put back as they were. Beside each binary,
# <binary>.libm-imports lists the math functions it still imports from libm (none, for the cr
# variants).
set -eu
. "$(dirname "$0")/env.sh"
shim=$E/cr-libm-shim/Cargo.toml
grep -q "rev = \"$INARI_MAIN\"" "$shim" || { echo "the shim is not at inari main"; exit 1; }
[ "$(grep -c '^inari = {' "$shim")" -eq 1 ] || { echo "the shim has no single inari line"; exit 1; }
cp "$shim" "$S/shim-Cargo.toml.saved"
cp "$W/Cargo.lock" "$S/Cargo.lock.saved"
restore() {
  cp "$S/shim-Cargo.toml.saved" "$shim"
  cp "$S/Cargo.lock.saved" "$W/Cargo.lock"
}
trap restore EXIT
# use_dep <line>: the saved shim manifest with its inari dependency line replaced by <line>.
use_dep() {
  awk -v line="$1" '/^inari = \{/ { print line; next } { print }' "$S/shim-Cargo.toml.saved" \
    > "$S/shim-Cargo.toml.next"
  cp "$S/shim-Cargo.toml.next" "$shim"
  grep '^inari = {' "$shim"
}
inari_at() { echo "inari = { git = \"https://github.com/mattiasgronlund/inari\", rev = \"$1\", default-features = false }"; }
fushimi_new="inari = { package = \"fushimi\", git = \"https://github.com/mattiasgronlund/fushimi\", rev = \"$NEW_COMMIT\", default-features = false }"
mkdir -p "$S/bins"
b() { # variant, features (empty for none)
  echo "=== goldens-$1"
  (cd "$W" && nice -n 10 taskset -c "$CPUS" cargo build --profile goldens -p goldens ${2:+--features "$2"})
  cp "$W/bin/goldens/goldens" "$S/bins/goldens-$1"
  libm_imports "$S/bins/goldens-$1"
}
[ $# -eq 0 ] && set -- baseline cr-count cr-main cr-ca6 cr-new cr-new-gen cr-new-count
for v in "$@"; do
  case $v in
    baseline) b baseline "" ;;
    cr-count) b cr-count cr-libm-count ;;
    cr-main) b cr-main cr-libm-generic ;;
    cr-ca6) use_dep "$(inari_at "$INARI_CA6")"; b cr-ca6 cr-libm-generic; restore ;;
    cr-new) use_dep "$fushimi_new"; b cr-new cr-libm; restore ;;
    cr-new-gen) use_dep "$fushimi_new"; b cr-new-gen cr-libm-generic; restore ;;
    cr-new-count) use_dep "$fushimi_new"; b cr-new-count cr-libm-count; restore ;;
    *) echo "unknown variant $v"; exit 1 ;;
  esac
done
echo "build-goldens-variants $* done"
