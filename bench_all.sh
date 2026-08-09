#!/usr/bin/env bash
#
# bench_all.sh -- full (system, mode, n) sweep for the fib(n) mod 7919 benchmark.
#
# Runs every cell SEQUENTIALLY (never two provers at once -- the compressed/
# recursion modes peak at ~17 GB RSS) and prints one summary line per cell:
#
#   CELL system=<risc0|sp1> mode=<...> n=<...> prove_s=<...> verify_ms=<...> \
#        proof_bytes=<...> peak_rss_kb=<...> cycles=<...>
#
# Peak RSS comes from /usr/bin/time -v when GNU time is installed, and otherwise
# from an inline python3 os.wait4() wrapper reporting the same ru_maxrss -- so the
# script is self-contained and needs no companion file.
#
# ---------------------------------------------------------------------------
# Toolchain install commands actually used (Arch Linux, 2026-08):
#
#   # Rust host toolchain: the risc0 host deps need rustc >= 1.90
#   # (ruint 1.20 wants 1.90, enum-ordinalize 4.4.2 wants 1.89).
#   rustup toolchain install 1.91.0        # or: rustup update stable
#
#   # RISC Zero (rzup -> cargo-risczero 3.0.6, r0vm 3.0.6, guest rust 1.97.0)
#   curl -L https://risczero.com/install | bash
#   export PATH="$HOME/.risc0/bin:$PATH"
#   rzup install
#
#   # SP1 (sp1up -> cargo-prove 6.3.1, `succinct` rust toolchain 1.94.0-dev)
#   curl -L https://sp1up.succinct.xyz | bash
#   export PATH="$HOME/.sp1/bin:$PATH"
#   sp1up
#
#   # Optional, for peak-RSS reporting via /usr/bin/time -v (falls back to a
#   # python3 wait4() wrapper, which reports the same ru_maxrss, if absent):
#   sudo pacman -S time                    # Debian/Ubuntu: sudo apt install time
#
#   # Docker is required ONLY for risc0's groth16 mode (stark-to-snark).
#   # Without it, `host <n> groth16` fails with "Please install docker first."
# ---------------------------------------------------------------------------
#
# Usage:  ./bench_all.sh [reps]        # default 3 reps per cell; median reported
#
set -uo pipefail

REPS="${1:-3}"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export PATH="$HOME/.risc0/bin:$HOME/.sp1/bin:$HOME/.cargo/bin:$PATH"
export SP1_PROVER=cpu

RISC0_BIN="$ROOT/risc0/target/release/host"
SP1_BIN="$ROOT/sp1/script/target/release/fibonacci-script"

# Host rustc must be >= 1.90 for the risc0 host crates; pick a toolchain that is.
CARGO_TC=""
if ! printf '1.90.0\n%s\n' "$(rustc --version | cut -d' ' -f2)" | sort -VC; then
    CARGO_TC="+1.91.0"
fi

[[ -x "$RISC0_BIN" ]] || ( cd "$ROOT/risc0" && cargo $CARGO_TC build --release )
[[ -x "$SP1_BIN"   ]] || ( cd "$ROOT/sp1/script" && \
                           RUSTFLAGS="-C target-cpu=native" cargo $CARGO_TC build --release )

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

# Peak RSS in kB. Prefers GNU time; falls back to python3 wait4() (same ru_maxrss).
if [[ -x /usr/bin/time ]]; then
    measure() { /usr/bin/time -v -o "$TMP/rss" "$@" 2>&1; }
    peak_kb() { awk '/Maximum resident set size/ {print $NF}' "$TMP/rss"; }
else
    cat > "$TMP/meas.py" <<'PY'
import os, subprocess, sys
p = subprocess.Popen(sys.argv[2:])
_, _, ru = os.wait4(p.pid, 0)
open(sys.argv[1], "w").write(str(int(ru.ru_maxrss)) + "\n")
PY
    measure() { python3 "$TMP/meas.py" "$TMP/rss" "$@" 2>&1; }
    peak_kb() { cat "$TMP/rss"; }
fi

# median of the numbers on stdin
median() { sort -g | awk '{v[NR]=$1} END {print (NR%2) ? v[(NR+1)/2] : (v[NR/2]+v[NR/2+1])/2}'; }

# cell <system> <mode> <n> <timeout_s>
cell() {
    local sys="$1" mode="$2" n="$3" to="$4" dir bin
    case "$sys" in
        risc0) dir="$ROOT/risc0";      bin="$RISC0_BIN" ;;
        sp1)   dir="$ROOT/sp1/script"; bin="$SP1_BIN"   ;;
    esac
    : > "$TMP/p" ; : > "$TMP/v" ; : > "$TMP/r"
    local bytes="?" cycles="?" out
    for _ in $(seq 1 "$REPS"); do
        out="$( cd "$dir" && measure timeout "$to" "$bin" "$n" "$mode" )"
        if ! grep -q '^BENCH' <<<"$out"; then
            echo "CELL system=$sys mode=$mode n=$n FAILED: $(grep -m1 -iE 'panic|error|docker' <<<"$out")"
            return
        fi
        out="$(grep -m1 '^BENCH' <<<"$out")"
        sed -n 's/.*prove_s=\([0-9.]*\).*/\1/p'    <<<"$out" >> "$TMP/p"
        sed -n 's/.*verify_ms=\([0-9.]*\).*/\1/p'  <<<"$out" >> "$TMP/v"
        peak_kb                                                >> "$TMP/r"
        bytes="$(sed -n 's/.*proof_bytes=\([0-9]*\).*/\1/p'  <<<"$out")"
        # sp1 prints cycles=; risc0 prints total_cycles: in its SessionStats
        cycles="$(sed -n 's/.*[^_]cycles=\([0-9]*\).*/\1/p'  <<<"$out")"
        [[ -n "$cycles" ]] || cycles="$(sed -n 's/.*total_cycles: \([0-9]*\).*/\1/p' <<<"$out")"
        sleep 2
    done
    printf 'CELL system=%s mode=%-10s n=%-6s prove_s=%s verify_ms=%s proof_bytes=%s peak_rss_kb=%s cycles=%s\n' \
        "$sys" "$mode" "$n" "$(median < "$TMP/p")" "$(median < "$TMP/v")" \
        "$bytes" "$(median < "$TMP/r")" "$cycles"
}

echo "=== fib(n) mod 7919 sweep | reps=$REPS | $(grep -m1 'model name' /proc/cpuinfo | cut -d: -f2-) ==="
cell risc0 composite  1000    900
cell risc0 composite  10000   900
cell risc0 succinct   1000   1200
cell risc0 succinct   10000  1800
cell sp1   core       1000   1200
cell sp1   core       10000  1200
cell sp1   compressed 1000   1800
cell sp1   compressed 10000  1800

# Fast-doubling guest: same journal (n, F(n) mod 7919, F(n+1) mod 7919), ~log2(n)
# iterations instead of n. This is the best-vs-best row against a log-depth prover.
cell risc0 succinct+fastdbl   1000   1200
cell risc0 succinct+fastdbl   10000  1200
cell sp1   core+fastdbl       1000   1200
cell sp1   core+fastdbl       10000  1200
cell sp1   compressed+fastdbl 10000  1800

# Bounds check: enforce 10 <= x <= 100 and commit x. Not a fib claim at all --
# this is the floor workload, the smallest thing worth proving. The n column is
# reused as x (the guest has no loop count), so 42 below is x, not an iteration
# count. Groth16 is excluded here as everywhere: it needs Docker.
cell risc0 composite+bounds  42  900
cell risc0 succinct+bounds   42  1200
cell sp1   core+bounds       42  1200
cell sp1   compressed+bounds 42  1800

# SNARK wraps. Both need Docker (risc0's stark-to-snark and sp1's gnark FFI both
# shell out to it), and sp1 additionally downloads ~9 GB of circuit artifacts.
# Off by default because they dominate the sweep's wall time: WRAP=1 ./bench_all.sh
if [[ "${WRAP:-0}" == 1 ]]; then
    cell risc0 groth16 10000 3600
    cell sp1   groth16 10000 3600
fi
