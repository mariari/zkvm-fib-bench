# zkvm-fib-bench

A tiny, reproducible **fib(N) benchmark for RISC Zero, SP1 and Jolt** on current
releases, that reports **prove *and* verify** time (plus proof size and guest cycles).

All three zkVMs run the **identical** program — the exact one behind
[zkbenchmarks.com](https://zkbenchmarks.com) (source:
[yetanotherco/zkvm_benchmarks](https://github.com/yetanotherco/zkvm_benchmarks)):

```rust
let (mut a, mut b) = (0u32, 1u32);
for _ in 0..n { let mut c = a + b; c %= 7919; a = b; b = c; }  // fib(n) mod 7919
commit(n); commit(a); commit(b);
```

The public site records only one end-to-end *prove* time; this harness additionally
times **verify** separately. Versions: **RISC Zero 3.0.5**, **SP1 6.3.1**, **Jolt**
pinned by git rev (`aec34d1`) — it has no crates.io release.

## Prerequisites

```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# RISC Zero toolchain (r0vm + guest compiler)
curl -L https://risczero.com/install | bash && rzup install

# SP1 toolchain (cargo-prove + succinct toolchain)
curl -L https://sp1up.succinct.xyz | bash && sp1up

# Jolt needs no installer: jolt/rust-toolchain.toml pins channel 1.95 and the
# riscv32imac/riscv64imac targets, which rustup fetches on first cargo use.

# Docker — only needed for the Groth16 (SNARK) phase, and for the RISC Zero
# guest build on very new host toolchains (see note below).
```

## Run

```bash
./run_fib.sh                 # fib(10000), STARK: risc0 succinct + sp1 compressed + jolt
./run_fib.sh 10000 snark     # Groth16 SNARK for risc0 + sp1 (needs Docker running)
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

Modes: risc0 `succinct | composite | groth16`; SP1 `core | compressed | groth16 | plonk`;
Jolt `stark` — it has a single proving mode and no SNARK wrap.
Append a guest-algorithm suffix to any mode:

| suffix | guest | journal |
|---|---|---|
| *(none)* | linear recurrence, n iterations | `(n, F(n) mod 7919, F(n+1) mod 7919)` |
| `+fastdbl` | fast doubling, ~log2(n) iterations | identical to linear — directly comparable |
| `+bounds` | assert `10 <= x <= 100`, no recurrence | `(x)` |

> ⚠️ **Rebuild after changing a guest.** Both scripts build only when the binary is *absent*,
> so a binary left from an earlier commit is silently benchmarked instead of your current
> guest. Run `cargo build --release` in `risc0/` and `sp1/script/` explicitly after any guest
> change.

## Layout

| Path | What |
|---|---|
| `risc0/` | RISC Zero project (guest `methods/guest`, timing host `host/src/main.rs`) — pinned to `risc0-zkvm = 3.0.5` |
| `sp1/`   | SP1 project (`program/` guest, `script/` timing host) — pinned to `sp1 = 6.3.1` (standalone; no monorepo needed) |
| `jolt/`  | Jolt project (`guest/` guest, `src/main.rs` timing host) — `jolt-sdk` pinned by git rev (standalone; no monorepo needed) |
| `run_fib.sh` | one-shot builder/runner |
| `bench_all.sh` | full (system, mode, n) sweep: 3 reps, median, peak RSS per cell |
| `RESULTS.md` | measured numbers + analysis (incl. why this is/isn't a fair comparison vs a specialized prover) |

## Note: RISC Zero guest builds in Docker

On a bleeding-edge host Rust, the risc0 guest's `borsh-derive` can hit a `proc_macro_crate`
panic. `run_fib.sh` therefore builds the guest with `RISC0_USE_DOCKER=1` (pinned toolchain,
reproducible ELF). If your host toolchain is older you can drop that and build natively.

`bench_all.sh` builds **natively** and needs no Docker. That path is verified on host
rustc 1.97.0 with the rzup guest toolchain — the panic above did not occur — so Docker is
required only for the groth16 cells (`WRAP=1`), where risc0's stark-to-snark and SP1's gnark
FFI both shell out to it.

## Note: Jolt's trace bound is a compile-time constant

risc0 and SP1 size the trace at run time. Jolt does not: `max_trace_length` is an
argument to the `#[jolt::provable]` attribute in `jolt/guest/src/lib.rs`, fixed at
compile time for every n. It is set to 2^21, which covers the linear program up to
n=100000; raise it before sweeping further, and rebuild (see the rebuild warning
above — the scripts only build when the binary is absent).

This is worth an eye during review: if Jolt sizes the prover to that declared bound
rather than to the actual trace, then every Jolt row costs the same regardless of n,
and the n=1000 and n=10000 cells are not comparable to each other or to the other two
VMs. One run at each n settles it — if the prove times are flat, the constant needs
tuning per n rather than one bound for the whole sweep.

## Results

See [`RESULTS.md`](RESULTS.md): three claims, each proved on zkFOL, RISC Zero and SP1 on
one CPU, with the source each side wrote and the figures. Headline, fib(10,000) mod
7919 by fast doubling on every system (AMD Ryzen 7 5700X):

|               |   zkFOL | RISC Zero composite | RISC Zero succinct | SP1 core | SP1 compressed |
|---------------|--------:|--------------------:|-------------------:|---------:|---------------:|
| prove         | 4.82 ms |              3.69 s |            14.73 s |  13.32 s |        49.24 s |
| verify        |  1.4 ms |             11.8 ms |            12.4 ms |  74.3 ms |        32.9 ms |
| proof         | 49.8 KB |            209.6 KB |           223.3 KB |  2.78 MB |        1.27 MB |
| prover memory | < 10 MB |              312 MB |            1.39 GB |  9.39 GB |       17.00 GB |

> zkVM prove times are hardware- and contention-sensitive (risc0 scales with cores;
> risc0 fib(1000) measured 33 s contended vs 18 s uncontended on the same box). Quote the
> exact CPU and "uncontended". Note SP1 compressed peaks near 17 GB RSS — on a box without
> swap, exceeding RAM is an OOM kill rather than a slowdown, so run cells serially.

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



