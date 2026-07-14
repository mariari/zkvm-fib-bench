# Results — fib(N): ZKFOL vs general zkVMs (RISC Zero / SP1)

**Machine:** AMD Ryzen 7 5700X (8c/16t), AVX2, **CPU** proving.
**Versions:** RISC Zero 3.0.5, SP1 6.3.1 (current releases, July 2026).
**zkVM program:** `fib(n) mod 7919` (u32), the *exact* program zkbenchmarks.com uses —
a **naive linear loop**, NOT fast-doubling. Output is a 4-digit residue, not the real number.
**ZKFOL:** proves the **exact** arbitrary-precision `fib(n)`; `zinc-plus` backend.

> ⚠️ zkVM prove times are **hardware- and contention-sensitive**. risc0 is embarrassingly
> parallel: fib(1000) measured 33.3 s under CPU contention vs **18.05 s uncontended** on the
> same box. Always quote the exact CPU and "uncontended". Numbers below are uncontended.

## Prove / verify (CPU, uncontended)

| task | system | algorithm | arithmetic | prove | verify | proof size |
|---|---|---|---|---|---|---|
| **fib(1,000)** | **ZKFOL** (int768) | linear | exact, 209-digit | **0.195 s** | 70.8 ms | — |
| | risc0 succinct | linear | mod-u32 | 18.05 s | 12.6 ms | 218 KB |
| | SP1 compressed | linear | mod-u32 | 47.19 s | 35.5 ms | 1.24 MB |
| **fib(10,000)** | **ZKFOL** (int7040) | **doubling** (log) | exact, 2090-digit | **0.686 s** | 16.5 ms | — |
| | risc0 succinct | linear | mod-u32 | 42.6 s | 12.3 ms | 218 KB |
| | SP1 compressed | linear | mod-u32 | 53.3 s | 35.0 ms | 1.24 MB |

Prove speedup, ZKFOL vs zkVM: **93× / 242×** (fib 1k, both linear) · **62× / 78×** (fib 10k).

## SNARK-wrapped (STARK → Groth16), fib(10,000)

| system | prove | verify | proof size |
|---|---|---|---|
| risc0 groth16 | 196.5 s | 3.2 ms | **521 bytes** |
| SP1 groth16 | *(pending — `./run_fib.sh 10000 snark`)* | | |

The wrap shrinks risc0's proof 218 KB → 521 B and verify 12.3 → 3.2 ms, at ~4.6× the prove time.

## Key findings

1. **A general zkVM pays a fixed proof-compression floor.** At n=1,000 the useful work is
   ~14k cycles, yet prove is 18–47 s — almost entirely the cost of recursively compressing
   execution into a constant-size STARK. SP1's floor is flat (47→53 s across n); risc0's is
   smaller but scales with cycles (18→42 s). ZKFOL has no such floor — it scales from ~0 with
   the actual claim. This is the structural reason it wins at small/medium sizes.
2. **The fairest point is fib(1,000), both linear.** Algorithm held fixed, ZKFOL is 93–242×
   faster *while doing exact bignum vs a mod-7919 residue*. So the gap is not an artifact of
   linear-vs-log — it survives with the algorithm matched.
3. **The mod barely helps the zkVM at this size** — it's floor-dominated, so exact-bignum
   *addition* would change ~14k cycles to ~100k, still a rounding error against 47 s. The mod
   only matters at large n where cycle count becomes the bottleneck.
4. **ZKFOL's edge = specialized representation (no VM tax, native bignum) × algorithmic
   freedom (doubling).** The zkVM benchmark exploits neither.

## Honest boundaries (do NOT overclaim)

- **Expressiveness, not speed, is the real limit of "faster regardless."** A zkVM runs
  *anything* (SHA, an EVM, arbitrary control flow). For claims ZKFOL can't encode, it's not
  in the race. That is the only place "regardless" genuinely fails.
- **Verify / proof size is the zkVM's column.** ZKFOL verify scales with the witness
  (70.8 ms @ num_vars=10; 16.5 ms @ 4) and can be *slower* than the zkVM's constant ~13 ms.
  Their Groth16 wrap gives a 521-byte proof with 3 ms verify — better for on-chain. Claim the
  **prove** win, not the verify/size win.
- **ZKFOL-linear is ~O(n²) bit-work** (exact bignum) and climbs toward the zkVM's flat floor
  around n≈10–20k — which is exactly why the doubling rewrite exists.

## A genuinely apples-to-apples comparison would

align **algorithm** and **output semantics**: e.g. an *exact fast-doubling bignum* guest in
the zkVM (same task as `e_doubling`), then compare. That isolates the pure "VM tax" (still
favors ZKFOL, but a smaller factor than 93×). See TODO in `run_fib.sh` / the exact-bignum guest.
