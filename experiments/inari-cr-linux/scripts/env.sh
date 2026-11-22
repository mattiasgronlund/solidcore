# Sourced by the other scripts.
#   E     this experiment's directory; W the solidcore tree that holds it.
#   S     a work directory outside the tree, for builds, binaries and raw outputs (required).
#   CPUS  the CPUs builds and full goldens runs may use, as a taskset list. The Linux host
#         keeps CPUs 11 and 23 for another job; elsewhere, set it to every CPU.
#   PIN   the CPU single-threaded timed runs are pinned to.
#   D     the solidcore of tangency-labs/solidcraft#15405 at ebd45f35d5 (`math::det` on the
#         `libm` crate), with patches/det-cr-ebd45f3.patch applied: the det-* variants.
E=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
W=$(cd "$E/../.." && pwd)
S=${S:?set S to a work directory outside the tree}
CPUS=${CPUS:-0-10,12-22}
PIN=${PIN:-5}
D=${D:-$HOME/git/tangency-labs/solidcraft-det-ebd45f3/solidcore}
INARI_MAIN=6a9fa7aad2a3ed0551e01239dac94ae149773d1c
INARI_CA6=ca6faf9c109ad62a7e53afc598d59f74af92adeb
# fushimi (inari renamed) e9e5ae57aff3a6cfe8ca6b8934d63eee2fee0715, written as every
# dependency line on it gives it.
NEW_COMMIT=e9e5ae5
export CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS:-2}
mkdir -p "$S"

# The solidcore tree a goldens variant is built in, run from, and checked against.
tree() {
  case $1 in
    det-*) echo "$D" ;;
    *) echo "$W" ;;
  esac
}

# Writes <binary>.libm-imports: the math functions the binary still imports from the C libm.
libm_imports() {
  local fns='sin|cos|tan|asin|acos|atan|sinh|cosh|tanh|asinh|acosh|atanh|exp|exp2|expm1|log|log1p|log2|log10|cbrt|atan2|pow|hypot|sincos'
  nm -D --undefined-only "$1" | awk '{print $2}' | sed 's/@.*//' \
    | { grep -xE "$fns" || true; } | sort > "$1.libm-imports"
  echo "$(basename "$1") imports $(wc -l < "$1.libm-imports") math functions from libm"
}
