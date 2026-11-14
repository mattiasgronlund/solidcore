#!/bin/bash
# Builds every goldens variant of the ed65e20 experiment into bin/variants/<name>.
set -e
cd "$(dirname "$0")/wt-cr2"
export CARGO_BUILD_JOBS=2
mkdir -p bin/variants
rustc --version
b() { # name, features
  echo "== $1 ($2) $(date +%T)"
  nice -n 10 cargo build --profile goldens -p goldens ${2:+--features $2} 2>&1 | grep -E "^(error|warning: unused)|Finished" -A6
  cp bin/goldens/goldens bin/variants/$1
}
b base ""
b cr-main cr
b acos-args acos-args
b ivl-count ivl-count
b ivl-inari ivl-inari
b ivl-generic ivl-inari-generic
cp crates/goldens/Cargo.toml ../goldens-Cargo.toml.main; cp Cargo.lock ../Cargo.lock.main
sed -i '' 's/rev = "6a9fa7aad2a3ed0551e01239dac94ae149773d1c", optional = true }/rev = "ca6faf9c109ad62a7e53afc598d59f74af92adeb", optional = true }/' crates/goldens/Cargo.toml
b cr-ca6 cr
cp Cargo.lock ../Cargo.lock.ca6
cp ../goldens-Cargo.toml.main crates/goldens/Cargo.toml; cp ../Cargo.lock.main Cargo.lock
echo "== done $(date +%T)"; ls -la bin/variants
