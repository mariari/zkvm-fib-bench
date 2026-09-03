#!/usr/bin/env bash
#
# Reproduce fib(N) prove/verify numbers locally on RISC Zero 3.0.5, SP1 6.3.1
# and Jolt.
#
# All three zkVMs run the IDENTICAL program (the exact one zkbenchmarks.com uses):
#     let (mut a, mut b) = (0u32, 1u32);
#     for _ in 0..n { let mut c = a + b; c %= 7919; a = b; b = c; }
#     commit(n); commit(a); commit(b);
#
# Unlike zkbenchmarks.com (which records only one end-to-end prove time), this
# also times VERIFY separately and reports proof size + guest cycles.
#
# Prereqs (see README.md):
#   - Rust (rustup)
#   - RISC Zero:  curl -L https://risczero.com/install | bash && rzup install
#   - SP1:        curl -L https://sp1up.succinct.xyz | bash && sp1up
#   - Jolt:       nothing to install -- jolt/rust-toolchain.toml pins the
#                 channel and the RISC-V targets, and rustup fetches them on
#                 the first cargo invocation in that directory
#   - Docker running (only for the `snark` phase)
#
# Usage:
#   ./run_fib.sh [N] [stark|snark|both]
#   ./run_fib.sh                 # fib(10000), STARK  (no Docker needed)
#   ./run_fib.sh 10000 snark     # Groth16 SNARK for both  (needs Docker)
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
JOLT_BIN="$ROOT/jolt/target/release/host"

# --- build on demand ---
if [[ ! -x "$RISC0_BIN" ]]; then
    echo ">> building RISC Zero (guest compiles in Docker) ..."
    ( cd "$ROOT/risc0" && RISC0_USE_DOCKER=1 cargo build --release )
fi
if [[ ! -x "$SP1_BIN" ]]; then
    echo ">> building SP1 (target-cpu=native for AVX2/512) ..."
    ( cd "$ROOT/sp1/script" && RUSTFLAGS="-C target-cpu=native" cargo build --release )
fi
# No `cargo +<version>` here: jolt/rust-toolchain.toml pins the channel *and*
# the RISC-V targets, and an explicit toolchain overrides the file and loses them.
if [[ ! -x "$JOLT_BIN" ]]; then
    echo ">> building Jolt (guest toolchain from jolt/rust-toolchain.toml) ..."
    ( cd "$ROOT/jolt" && cargo build --release )
fi

risc0() { ( cd "$ROOT/risc0"     && ./target/release/host             "$N" "$1" ); }
sp1()   { ( cd "$ROOT/sp1/script" && ./target/release/fibonacci-script "$N" "$1" ); }
jolt()  { ( cd "$ROOT/jolt"       && ./target/release/host             "$N" "$1" ); }

echo "=========================================================="
echo " fib($N)  |  RISC Zero 3.0.5  vs  SP1 6.3.1  vs  Jolt  |  CPU"
echo "=========================================================="
case "$PHASE" in
    stark) risc0 succinct; sp1 compressed; jolt stark ;;
    snark) echo "(Groth16 needs Docker running; Jolt has no SNARK wrap)"; risc0 groth16; sp1 groth16 ;;
    both)  risc0 succinct; sp1 compressed; jolt stark; echo "--- SNARK (Docker) ---"; risc0 groth16; sp1 groth16 ;;
    *)     echo "usage: ./run_fib.sh <N> [stark|snark|both]"; exit 1 ;;
esac
