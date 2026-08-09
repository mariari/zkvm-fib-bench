# Results — fib(N) and a bounds check: ZKFOL vs general zkVMs (RISC Zero / SP1)

**Machine:** AMD Ryzen 7 5700X (8c/16t), AVX2, **CPU** proving, 62 GiB RAM, **no swap configured**.
**Versions:** RISC Zero 3.0.5, SP1 6.3.1.
**zkVM program:** `fib(n) mod 7919` (u32) — the *exact* program zkbenchmarks.com uses, a
**naive linear loop**, NOT fast-doubling. Output is a 4-digit residue, not the real number.
The `+fastdbl` and `+bounds` guest algorithms added here are described under
[Guest algorithms](#guest-algorithms).
**ZKFOL:** proves the **exact** arbitrary-precision `fib(n)`; `zinc-plus` backend
(`PnttConfig7340033`, 2^20 codewords).

> ⚠️ zkVM prove times are **hardware- and contention-sensitive**. Always quote the exact CPU
> and "uncontended". Every number below was taken with the box otherwise idle, cells run
> strictly one at a time.

## Full sweep — `./bench_all.sh` (3 reps per cell, median)

| system | mode | n | prove (s) | verify (ms) | proof | peak RSS | cycles |
|---|---|--:|--:|--:|--:|--:|--:|
| risc0 | composite | 1,000 | 7.341 | 12.442 | 221.2 KB | 618 MB | 65,536 |
| risc0 | composite | 10,000 | 29.653 | 14.323 | 256.0 KB | 2.30 GB | 262,144 |
| risc0 | succinct | 1,000 | 18.283 | 12.354 | 223.3 KB | 1.40 GB | 65,536 |
| risc0 | succinct | 10,000 | 40.673 | 12.517 | 223.3 KB | 2.30 GB | 262,144 |
| sp1 | core | 1,000 | 13.439 | 77.618 | 2.78 MB | 9.42 GB | 19,138 |
| sp1 | core | 10,000 | 14.144 | 77.393 | 2.78 MB | 9.55 GB | 145,138 |
| sp1 | compressed | 1,000 | 49.118 | 33.863 | 1.27 MB | 17.13 GB | 19,138 |
| sp1 | compressed | 10,000 | 50.391 | 35.235 | 1.27 MB | 17.07 GB | 145,138 |
| risc0 | composite+fastdbl | 10,000 | 3.687 | 11.846 | 209.6 KB | 312 MB | 32,768 |
| risc0 | succinct+fastdbl | 1,000 | 14.699 | 12.512 | 223.3 KB | 1.39 GB | 32,768 |
| risc0 | succinct+fastdbl | 10,000 | 14.726 | 12.400 | 223.3 KB | 1.39 GB | 32,768 |
| sp1 | core+fastdbl | 1,000 | 13.308 | 75.179 | 2.78 MB | 9.37 GB | 6,216 |
| sp1 | core+fastdbl | 10,000 | 13.319 | 74.280 | 2.78 MB | 9.39 GB | 6,213 |
| sp1 | compressed+fastdbl | 10,000 | 49.244 | 32.889 | 1.27 MB | 17.00 GB | 6,213 |

`risc0 composite+fastdbl` is not a `bench_all.sh` cell; it was run separately with identical
methodology. `sp1 compressed+fastdbl n=1000` and `risc0 composite+fastdbl n=1000` remain
unmeasured.

## Prove / verify, n=10,000 — each system's default route

| route | prove (s) | verify (ms) | proof | Δ prover mem |
|---|--:|--:|--:|--:|
| **ZKFOL** doubled mod | **0.005** | 1.4 | 49.8 KB | < 10 MB |
| **ZKFOL** registers mod | **1.06** | 31.2 | 866 KB | < 10 MB |
| **ZKFOL** registers (full, exact 2090-digit) | **3.65** | 820.6 | 15.5 MB | 1.20 GB |
| risc0 succinct | 40.67 | 12.5 | 223.3 KB | 2.30 GB |
| sp1 compressed | 50.39 | 35.2 | 1.27 MB | 17.07 GB |

## Prove / verify, n=10,000 — algorithm held fixed (fast doubling)

| route | prove | verify (ms) | proof | Δ prover mem |
|---|--:|--:|--:|--:|
| **ZKFOL** doubled mod | **4.82 ms** | 1.4 | 49.8 KB | < 10 MB |
| risc0 composite + fastdbl | 3.69 s | 11.8 | 209.6 KB | 312 MB |
| risc0 succinct + fastdbl | 14.73 s | 12.4 | 223.3 KB | 1.39 GB |
| sp1 core + fastdbl | 13.32 s | 74.3 | 2.78 MB | 9.39 GB |
| sp1 compressed + fastdbl | 49.24 s | 32.9 | 1.27 MB | 17.00 GB |

## The floor — a bounds check, `x ∈ [10,100]`, x=42

The smallest claim worth making: one range assertion, no recurrence. Single runs, not medians.

| system / mode | prove | verify (ms) | proof | peak prover mem | cycles |
|---|--:|--:|--:|--:|--:|
| **ZKFOL** bounds check | **1.53 ms** | 0.54 | 28.0 KB | ~3 MB | — |
| risc0 composite+bounds | 3.70 s | 11.5 | 209.6 KB | 311 MB | 32,768 (3,876 user) |
| risc0 succinct+bounds | 14.68 s | 12.3 | 223.2 KB | 1.39 GB | 32,768 (3,876 user) |
| sp1 core+bounds | 13.23 s | 78.2 | 2.78 MB | 9.36 GB | 4,882 |
| sp1 compressed+bounds | 49.20 s | 32.6 | 1.27 MB | 17.01 GB | 4,882 |

Prove ratio vs ZKFOL: **2,420× / 9,600× / 8,650× / 32,160×**.

### The floor is real, and it is not a function of the claim

Three independent measurements land on the same cost:

1. **risc0 pads to 32,768 cycles regardless.** The bounds check uses 3,876 *user* cycles and
   still reports `total_cycles: 32768` — the same total as `+fastdbl` and the same as a
   separately measured `n=1` run (`composite n=1`: prove 3.686 s, 209,586 B, 318,792 kB,
   32,768 cycles, against bounds' 3.702 s, 209,570 B, 318,172 kB). Three different claims,
   one number.
2. **SP1 barely responds to cycle count.** 4,882 cycles (bounds) costs 13.23 s core /
   49.20 s compressed; 145,138 cycles (n=10,000 linear) costs 14.14 s / 50.39 s. A **30×
   cycle reduction buys ~0.9 s.**
3. **Cutting real work 23× buys 0.8 s.** sp1 core at 6,213 cycles (fastdbl) vs 145,138
   (linear): 13.32 s vs 14.14 s.

A bounds check cannot beat this floor because there is nothing left to remove. The floor is
the cost of recursively compressing *any* execution into a constant-size proof.

## Key findings

1. **A general zkVM pays a fixed proof-compression floor.** SP1's is flat (~13 s core,
   ~49 s compressed) across every claim measured — n=1,000, n=10,000, fast doubling, and a
   single range assertion. risc0's floor is lower but scales with cycles once the claim
   exceeds the 32,768-cycle pad. ZKFOL has no such floor: it scales from ~0 with the actual
   claim. This is the structural reason it wins at small and medium sizes.
2. **The fairest fib point is n=1,000, both linear.** Algorithm held fixed, ZKFOL is faster
   *while doing exact bignum vs a mod-7919 residue* — so the gap is not an artifact of
   linear-vs-log; it survives with the algorithm matched.
3. **Memory is SP1's real cost.** SP1 compressed peaks at ~17 GB, 7.4× risc0 succinct's
   2.3 GB and ~5,800× ZKFOL's bounds-check footprint. On a 62 GiB box **with no swap**, that
   fits but has no soft landing: exceeding RAM is an OOM kill, not degradation. Run cells
   strictly serially.
4. **ZKFOL's edge = specialized representation (no VM tax, native bignum) × algorithmic
   freedom (doubling).** The zkVM benchmark exploits neither.

## Honest boundaries (do NOT overclaim)

- **Expressiveness, not speed, is the real limit of "faster regardless."** A zkVM runs
  *anything* (SHA, an EVM, arbitrary control flow). For claims ZKFOL can't encode, it is not
  in the race. That is the only place "regardless" genuinely fails.
- **Verify and proof size are usually the zkVM's column.** At n=10,000 ZKFOL verify is
  820 ms against risc0's 12.5 ms — the zkVM wins outright. The bounds check is the exception,
  where ZKFOL wins verify too (0.54 ms vs 11.5 ms). Claim the **prove** and **memory** wins
  generally; claim verify only at small claim sizes, and say which.
- **A Groth16 wrap remains the zkVM's trump card for on-chain use** — 521-byte proof, ~3 ms
  verify. Not re-measured in this sweep (needs Docker; see below).
- **ZKFOL-linear is ~O(n²) bit-work** (exact bignum) and climbs toward the zkVM's flat floor
  around n≈10–20k — which is exactly why the doubling rewrite exists.
- **The bounds row compares floors, not identical tasks.** It answers "what is the minimum a
  zkVM charges for *any* claim" against "what ZKFOL charges for this one" — not a like-for-like
  program comparison.

## Guest algorithms

Selected by suffixing the prover mode (e.g. `succinct+fastdbl`, `core+bounds`):

| suffix | guest | journal |
|---|---|---|
| *(none)* | linear recurrence, n iterations | `(n, F(n) mod 7919, F(n+1) mod 7919)` |
| `+fastdbl` | fast doubling, ~log2(n) iterations | identical to linear — directly comparable |
| `+bounds` | assert `10 <= x <= 100`, no recurrence | `(x)` |

## Methodology

- **Reps.** Sweep cells are the median of 3 reps. The bounds cells and `risc0 composite n=1`
  are single runs; their agreement with the 3-rep figures (49.20 vs 49.24, 13.23 vs 13.32,
  14.68 vs 14.73) suggests few-percent noise.
- **Peak RSS.** `/usr/bin/time -v` when GNU time is installed, otherwise an inline
  `python3 os.wait4()` wrapper reporting the same `ru_maxrss`. Both provers are
  single-process/multi-threaded, so `ru_maxrss` covers the whole prover.
- **ZKFOL numbers** come from `Examples.EBench` in the zkfol repo under `MIX_ENV=test`,
  single unwarmed runs (reps=1) — a methodology asymmetry against the zkVM medians. Its
  `Δ prover mem` is peak minus post-GC baseline and brackets emit-prove-verify only, so
  witness solving is not counted; the zkVM figures are absolute peak RSS for the whole process.
- **Docker is not required for STARK modes.** `bench_all.sh` builds the risc0 guest natively
  and gates groth16 behind `WRAP=1`. The native build succeeded on host rustc 1.97.0 with the
  rzup guest toolchain — the `borsh-derive` panic the README warns about did not occur.
  Docker is still required for the two groth16 cells, which were not run.
- **⚠️ Stale-binary trap.** `bench_all.sh` builds only when a binary is *absent*, so an
  existing binary from an earlier commit is silently benchmarked instead of the current guest.
  Rebuild explicitly (`cargo build --release` in both projects) after changing any guest, or
  the `+fastdbl` / `+bounds` numbers will describe the wrong program.

## A genuinely apples-to-apples comparison would

align **algorithm** and **output semantics**: an *exact fast-doubling bignum* guest in the
zkVM (same task as `e_doubling`), then compare. That isolates the pure "VM tax". The
`+fastdbl` guest aligns the algorithm but still computes a mod-7919 residue, not the exact
2090-digit integer, so it closes half the gap only.
