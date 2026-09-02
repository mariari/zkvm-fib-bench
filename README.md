# zkvm-fib-bench

We benchmark simple functions written in various ZK programming systems

## Prerequisites

```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# RISC Zero toolchain (r0vm + guest compiler)
curl -L https://risczero.com/install | bash && rzup install

# SP1 toolchain (cargo-prove + succinct toolchain)
curl -L https://sp1up.succinct.xyz | bash && sp1up

# Docker — only needed for the Groth16 (SNARK) phase, and for the RISC Zero
# guest build on very new host toolchains (see note below).
```

## Fib

A tiny, reproducible **fib(N) benchmark for RISC Zero and SP1** on current releases,
that reports **prove *and* verify** time (plus proof size and guest cycles).

Both zkVMs run the **identical** program, the one behind
[zkbenchmarks.com](https://zkbenchmarks.com) (source:
[yetanotherco/zkvm_benchmarks](https://github.com/yetanotherco/zkvm_benchmarks)):

```rust
let (mut a, mut b) = (0u32, 1u32);
for _ in 0..n { let mut c = a + b; c %= 7919; a = b; b = c; }  // fib(n) mod 7919
commit(n); commit(a); commit(b);
```

The public site records only one end-to-end *prove* time; this harness additionally
times **verify** separately. Versions: **RISC Zero 3.0.5**, **SP1 6.3.1**.


### Run

```bash
./run_fib.sh                 # fib(10000), STARK: risc0 succinct + sp1 compressed
./run_fib.sh 10000 snark     # Groth16 SNARK for both (needs Docker running)
./run_fib.sh 100000 both     # STARK + SNARK at N=100000

./bench_all.sh               # full (system, mode, n) sweep, 3 reps, median + peak RSS
./bench_all.sh 1             # same sweep, 1 rep
WRAP=1 ./bench_all.sh        # additionally run the groth16 cells (needs Docker)
```

`run_fib.sh` builds each project on first use, then prints one `BENCH …` line per VM:

```
BENCH risc0 mode=succinct   n=10000 prove_s=42.634 verify_ms=12.280 proof_bytes=223250 ...
BENCH sp1   mode=compressed n=10000 prove_s=53.328 verify_ms=34.962 proof_bytes=1272581 ...
```

`bench_all.sh` runs every cell sequentially (never two provers at once — compressed peaks at
~17 GB) and prints one `CELL …` line per cell, adding `peak_rss_kb`.

Modes: risc0 `succinct | composite | groth16`; SP1 `core | compressed | groth16 | plonk`.
Append a guest-algorithm suffix to any mode:

| suffix     | guest                                  | journal                                    |
|------------|----------------------------------------|--------------------------------------------|
| *(none)*   | linear recurrence, n iterations        | `(n, F(n) mod 7919, F(n+1) mod 7919)`      |
| `+fastdbl` | fast doubling, ~log2(n) iterations     | identical to linear — directly comparable |
| `+bounds`  | assert `10 <= x <= 100`, no recurrence | `(x)`                                      |

> ⚠️ **Rebuild after changing a guest.** Both scripts build only when the binary is *absent*,
> so a binary left from an earlier commit is silently benchmarked instead of your current
> guest. Run `cargo build --release` in `risc0/` and `sp1/script/` explicitly after any guest
> change.

## Sudoku benchmark

A second workload proves that a completed **n×n grid is a valid sudoku** (n a
perfect square, box side b = √n): each of the **3n groups** — n rows, n columns,
n non-overlapping b×b boxes — is a permutation of {1,…,n}. Each group is checked
the **direct, idiomatic** way you would in plain Rust: walk its n cells with a
`seen` array, asserting every value is in `1..=n` and appears exactly once.
Plain integer/boolean ops — no modular arithmetic, no overflow concerns. The host
generates a known-valid grid of the requested size and commits `(n, grid)`, so the
proof attests "this public n×n grid is a valid sudoku".

This is the fair, apples-to-apples framing: each system proves "valid sudoku" in
its own natural idiom — zkFOL via its power-sums / permutation arithmetisation, the
zkVMs via this distinctness check (what a zkVM developer would actually write).

Same structure and BENCH line as fib; `n` is the grid size (default 9):

```bash
./run_sudoku.sh            # 9x9, STARK: risc0 succinct + sp1 compressed
./run_sudoku.sh 4          # 4x4
./run_sudoku.sh 16 both    # 16x16, STARK + Groth16 SNARK (needs Docker)
```

Direct binary calls mirror fib (first arg is `n`, second the prover mode):

```bash
risc0/target/release/sudoku 9 succinct
sp1/script/target/release/sudoku 9 compressed
```

Headline at 9×9 (CPU, AMD Ryzen 7 5700X, single runs): zkFOL **33.31 ms** / 3.36 ms verify /
under 10 MB, against RISC Zero composite 7.25 s / 606 MB, RISC Zero succinct 18.04 s /
1.43 GB, SP1 core 13.69 s / 9.57 GB, SP1 compressed 50.33 s / 16.80 GB.

Note the two sides prove **different statements**: the guests `commit(&grid)`, so the grid
is public and already completed, while zkFOL unifies an answer against a seventeen-clue
puzzle and reports `public_cols: 0` — nothing of the grid is revealed. At 16×16 zkFOL is 63.22 ms against RISC Zero composite 14.73 s and SP1 compressed 50.17 s.
Across 4×4, 9×9 and 16×16 the guest cycles rise 6.9× while SP1 compressed moves under 1.4%
(49.5 → 50.2 s), and zkFOL is the only column that responds to the puzzle at all, rising 5.1×
from 9×9 to 16×16: the zkVM cost tracks its cycle pad, not the puzzle. Full tables in [`RESULTS.md`](RESULTS.md#4-sudoku-a-completed-grid-is-valid).

## Layout

| Path            | What                                                            |
|-----------------|-----------------------------------------------------------------|
| `risc0/`        | RISC Zero project                                               |
| `sp1/`          | SP1 project                                                     |
| `run_fib.sh`    | one-shot fib builder/runner                                     |
| `run_sudoku.sh` | one-shot sudoku builder/runner                                  |
| `bench_all.sh`  | full (system, mode, n) sweep: 3 reps, median, peak RSS per cell |
| `RESULTS.md`    | measured numbers + analysis                                     |

## Note: RISC Zero guest builds in Docker

On a bleeding-edge host Rust, the risc0 guest's `borsh-derive` can hit a `proc_macro_crate`
panic. `run_fib.sh` therefore builds the guest with `RISC0_USE_DOCKER=1` (pinned toolchain,
reproducible ELF). If your host toolchain is older you can drop that and build natively.

`bench_all.sh` builds **natively** and needs no Docker. That path is verified on host
rustc 1.97.0 with the rzup guest toolchain — the panic above did not occur — so Docker is
required only for the groth16 cells (`WRAP=1`), where risc0's stark-to-snark and SP1's gnark
FFI both shell out to it.

## Results

See [`RESULTS.md`](RESULTS.md)

## Credits

This is an independent, minimal harness — **not a fork**. The reused pieces are all
permissively licensed:

- The `fib(n) mod 7919` program and the choice of N values follow the
  **zkbenchmarks.com** harness by Yet Another Company
  [yetanotherco/zkvm_benchmarks](https://github.com/yetanotherco/zkvm_benchmarks)
  (MIT).
- Per-VM project scaffolds come from [SP1](https://github.com/succinctlabs/sp1)
  (Succinct Labs, MIT/Apache-2.0) and [RISC Zero](https://github.com/risc0/risc0)
  (MIT/Apache-2.0).

This repo's own contribution is the instrumented prove/verify timing hosts, the runner,
and the analysis in `RESULTS.md`. Licensed MIT (see `LICENSE`).



