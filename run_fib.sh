#!/usr/bin/env bash
#
# Reproduce fib(N) prove/verify numbers locally on RISC Zero, SP1, and Jolt.
#
# Both zkVMs run the IDENTICAL program (the exact one zkbenchmarks.com uses):
#     let (mut a, mut b) = (0u32, 1u32);
#     for _ in 0..n { let mut c = a + b; c %= 7919; a = b; b = c; }
#     commit(n); commit(a); commit(b);
#
# Unlike zkbenchmarks.com (which records only one end-to-end prove time), this
# also times VERIFY separately and reports proof size + guest cycles.
#
# Prereqs (see README.md):
#   - Rust (rustup)
#   - Jolt:        ./jolt/build.sh (builds the pinned Jolt CLI)
#
# Usage:
#   ./run_fib.sh [N] [stark|snark|both]
#   ./run_fib.sh                 # fib(10000), STARK
#   ./run_fib.sh 10000 snark     # Groth16 for RISC Zero and SP1 (needs Docker)
#   ./run_fib.sh 100000 both
#
set -euo pipefail

N="${1:-10000}"
PHASE="${2:-stark}"

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export PATH="$HOME/.risc0/bin:$HOME/.sp1/bin:$HOME/.cargo/bin:$PATH"
export SP1_PROVER=cpu

RISC0_BIN="$ROOT/risc0/target/release/host"
SP1_BIN="$ROOT/sp1/script/target/release/fibonacci-script"
JOLT_BIN="$ROOT/jolt/target/release/jolt-fib-host"
JOLT_CLI="$ROOT/jolt/.toolchain/bin/jolt"

# --- build on demand ---
if [[ ! -x "$RISC0_BIN" ]]; then
    echo ">> building RISC Zero (guest compiles in Docker) ..."
    ( cd "$ROOT/risc0" && RISC0_USE_DOCKER=1 cargo build --release )
fi
if [[ ! -x "$SP1_BIN" ]]; then
    echo ">> building SP1 (target-cpu=native for AVX2/512) ..."
    ( cd "$ROOT/sp1/script" && RUSTFLAGS="-C target-cpu=native" cargo build --release )
fi
if [[ ! -x "$JOLT_BIN" || ! -x "$JOLT_CLI" ]]; then
    echo ">> building Jolt ..."
    ( cd "$ROOT/jolt" && ./build.sh )
fi

risc0() { ( cd "$ROOT/risc0"      && ./target/release/host             "$N" "$1" ); }
sp1()   { ( cd "$ROOT/sp1/script" && ./target/release/fibonacci-script "$N" "$1" ); }
jolt()  { ( cd "$ROOT/jolt"      && JOLT_PATH="$JOLT_CLI" ./target/release/jolt-fib-host "$N" ); }

echo "=========================================================="
echo " fib($N)  |  RISC Zero 3.0.5  vs  SP1 6.3.1  vs  Jolt  |  CPU"
echo "=========================================================="
case "$PHASE" in
    stark) risc0 succinct; sp1 compressed; jolt ;;
    snark) echo "(Groth16 needs Docker; Jolt has no SNARK mode)"; risc0 groth16; sp1 groth16 ;;
    both)  risc0 succinct; sp1 compressed; jolt; echo "--- SNARK (Docker) ---"; risc0 groth16; sp1 groth16 ;;
    *)     echo "usage: ./run_fib.sh <N> [stark|snark|both]"; exit 1 ;;
esac
