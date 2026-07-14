# zkvm-fib-bench

A tiny, reproducible **fib(N) benchmark for RISC Zero and SP1** on current releases,
that reports **prove *and* verify** time (plus proof size and guest cycles).

Both zkVMs run the **identical** program — the exact one behind
[zkbenchmarks.com](https://zkbenchmarks.com) (source:
[yetanotherco/zkvm_benchmarks](https://github.com/yetanotherco/zkvm_benchmarks)):

```rust
let (mut a, mut b) = (0u32, 1u32);
for _ in 0..n { let mut c = a + b; c %= 7919; a = b; b = c; }  // fib(n) mod 7919
commit(n); commit(a); commit(b);
```

The public site records only one end-to-end *prove* time; this harness additionally
times **verify** separately. Versions: **RISC Zero 3.0.5**, **SP1 6.3.1**.

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

## Run

```bash
./run_fib.sh                 # fib(10000), STARK: risc0 succinct + sp1 compressed
./run_fib.sh 10000 snark     # Groth16 SNARK for both (needs Docker running)
./run_fib.sh 100000 both     # STARK + SNARK at N=100000
```

`run_fib.sh` builds each project on first use, then prints one `BENCH …` line per VM:

```
BENCH risc0 mode=succinct   n=10000 prove_s=42.634 verify_ms=12.280 proof_bytes=223250 ...
BENCH sp1   mode=compressed n=10000 prove_s=53.328 verify_ms=34.962 proof_bytes=1272581 ...
```

Modes: risc0 `succinct | composite | groth16`; SP1 `core | compressed | groth16 | plonk`.

## Layout

| Path | What |
|---|---|
| `risc0/` | RISC Zero project (guest `methods/guest`, timing host `host/src/main.rs`) — pinned to `risc0-zkvm = 3.0.5` |
| `sp1/`   | SP1 project (`program/` guest, `script/` timing host) — pinned to `sp1 = 6.3.1` (standalone; no monorepo needed) |
| `run_fib.sh` | one-shot builder/runner |
| `RESULTS.md` | measured numbers + analysis (incl. why this is/isn't a fair comparison vs a specialized prover) |

## Note: RISC Zero guest builds in Docker

On a bleeding-edge host Rust, the risc0 guest's `borsh-derive` hits a `proc_macro_crate`
panic. `run_fib.sh` therefore builds the guest with `RISC0_USE_DOCKER=1` (pinned toolchain,
reproducible ELF). If your host toolchain is older you can drop that and build natively.

## Results

See [`RESULTS.md`](RESULTS.md). Headline (fib 10,000, CPU, AMD Ryzen 7 5700X):

| | RISC Zero (succinct STARK) | SP1 (compressed STARK) |
|---|---|---|
| prove | 42.6 s | 53.3 s |
| verify | 12.3 ms | 35.0 ms |
| proof size | ~218 KB | ~1.24 MB |

Groth16 wrap (risc0): 196.5 s prove, 3.2 ms verify, **521-byte** proof.

> zkVM prove times are hardware- and contention-sensitive (risc0 scales with cores;
> risc0 fib(1000) measured 33 s contended vs 18 s uncontended on the same box). Quote the
> exact CPU and "uncontended".

## Credits

This is an independent, minimal harness — **not a fork**. The reused pieces are all
permissively licensed:

- The `fib(n) mod 7919` program and the choice of N values follow the **zkbenchmarks.com**
  harness by Yet Another Company —
  [yetanotherco/zkvm_benchmarks](https://github.com/yetanotherco/zkvm_benchmarks) (MIT).
- Per-VM project scaffolds come from [SP1](https://github.com/succinctlabs/sp1)
  (Succinct Labs, MIT/Apache-2.0) and [RISC Zero](https://github.com/risc0/risc0)
  (MIT/Apache-2.0).

This repo's own contribution is the instrumented prove/verify timing hosts, the runner,
and the analysis in `RESULTS.md`. Licensed MIT (see `LICENSE`).
