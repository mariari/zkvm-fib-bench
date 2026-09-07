#!/usr/bin/env bash
#
# Reproduce the sudoku-validity prove/verify numbers locally on RISC Zero, SP1,
# and Jolt (Jolt's first version is fixed at 9x9).
#
# All systems run the same public-grid claim: prove that a completed n x n grid
# is a valid sudoku, i.e. each of the 3n groups -- n rows, n columns, and n b x b
# boxes -- is a permutation of {1,...,n}.
# The check is by power sums: for k = 1..n,
#     sum_{cell in group} cell^k  ==  sum_{v=1}^{n} v^k
# which pins each group's multiset to exactly {1..n} (distinctness AND range in
# one shot). Arithmetic is mod the prime 2^31-1 > n, mirroring zkFOL's prime
# field and dodging u64 overflow (16^16 = 2^64). The host generates a known-valid
# grid of size n and commits (n, grid); the proof attests it is a valid sudoku.
#
# Prereqs are identical to run_fib.sh (see README.md).
#
# Usage:
#   ./run_sudoku.sh [N] [stark|snark|both]
#   ./run_sudoku.sh                  # 9x9, STARK  (no Docker needed)
#   ./run_sudoku.sh 4                # 4x4, STARK
#   ./run_sudoku.sh 16 snark         # 16x16, Groth16 SNARK for both (needs Docker)
#
set -euo pipefail

N="${1:-9}"        # grid size (perfect square: 4, 9, 16, ...)
PHASE="${2:-stark}"

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export PATH="$HOME/.risc0/bin:$HOME/.sp1/bin:$HOME/.cargo/bin:$PATH"
export SP1_PROVER=cpu

RISC0_BIN="$ROOT/risc0/target/release/sudoku"
SP1_BIN="$ROOT/sp1/script/target/release/sudoku"
JOLT_BIN="$ROOT/jolt/target/release/sudoku"
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
if [[ "$N" == 9 && ( ! -x "$JOLT_BIN" || ! -x "$JOLT_CLI" ) ]]; then
    echo ">> building Jolt ..."
    ( cd "$ROOT/jolt" && ./build.sh )
fi

risc0() { ( cd "$ROOT/risc0"     && ./target/release/sudoku "$N" "$1" ); }
sp1()   { ( cd "$ROOT/sp1/script" && ./target/release/sudoku "$N" "$1" ); }
jolt()  { ( cd "$ROOT/jolt" && JOLT_PATH="$JOLT_CLI" ./target/release/sudoku 9 ); }

echo "=========================================================="
echo " sudoku(${N}x${N})  |  RISC Zero 3.0.5  vs  SP1 6.3.1  vs  Jolt (9x9)  |  CPU"
echo "=========================================================="
case "$PHASE" in
    stark)
        risc0 succinct; sp1 compressed
        [[ "$N" == 9 ]] && jolt
        ;;
    snark) echo "(Groth16 needs Docker; Jolt has no SNARK mode)"; risc0 groth16; sp1 groth16 ;;
    both)  risc0 succinct; sp1 compressed; [[ "$N" == 9 ]] && jolt; echo "--- SNARK (Docker) ---"; risc0 groth16; sp1 groth16 ;;
    *)     echo "usage: ./run_sudoku.sh <N> [stark|snark|both]"; exit 1 ;;
esac
