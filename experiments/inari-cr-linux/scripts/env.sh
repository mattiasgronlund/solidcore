# Sourced by the other scripts.
#   E     this experiment's directory; W the solidcore tree that holds it.
#   S     a work directory outside the tree, for builds, binaries and raw outputs (required).
#   CPUS  the CPUs builds and full goldens runs may use, as a taskset list. The Linux host
#         keeps CPUs 11 and 23 for another job; elsewhere, set it to every CPU.
#   PIN   the CPU single-threaded timed runs are pinned to.
E=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
W=$(cd "$E/../.." && pwd)
S=${S:?set S to a work directory outside the tree}
CPUS=${CPUS:-0-10,12-22}
PIN=${PIN:-5}
INARI_MAIN=6a9fa7aad2a3ed0551e01239dac94ae149773d1c
INARI_CA6=ca6faf9c109ad62a7e53afc598d59f74af92adeb
export CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS:-2}
mkdir -p "$S"
