# Results: zkFOL vs RISC Zero vs SP1

Three claims, each proved on every system, on one machine.

| claim | why this row exists |
|---|---|
| **fib(10,000) mod 7919, fast doubling on every system** | the fair fight: same claim, same algorithm, only the proof system differs |
| **bounds check 10 ≤ x ≤ 100** | the floor: the smallest claim worth proving, so the fixed cost of a proof |
| **fib(10,000) mod 7919, each system's default** | what you get running the published benchmark unchanged, plus zkFOL's exact full-integer rows so the column it loses is visible |

**Machine:** AMD Ryzen 7 5700X (8c/16t), Linux, CPU proving only, one prover at a time.
**Versions:** zkFOL 0.4.0 (zinc-plus `451ba17`), RISC Zero 3.0.5, SP1 6.3.1. Measured August 2026.
**Numbers:** zkVM cells are the median of 3 runs of `bench_all.sh`; zkFOL rows are one
`Examples.EBench` call each (the map it returns is the row).

The one-line version: at the same claim and the same algorithm, zkFOL proves in
**4.8 ms** where RISC Zero takes **3.7 s** (composite) to **14.7 s** (succinct) and SP1
**13.3 s** (core) to **49 s** (compressed); the proof is 50 KB against 210 KB to 2.8 MB.

## 1. Same claim, same algorithm

Everyone proves fib(10,000) mod 7919 by fast doubling, ~14 rounds instead of 10,000
steps. The zkVM guests use a hand-written `fast_doubling` (below). zkFOL is handed the
naive two-call recurrence and `Zkfol.Doubling` rewrites it to the log-depth kernel.

![prover time and peak RSS, fib(10,000) mod 7919, fast doubling](prove_fib10000.svg)

| system | prove | verify | proof | peak RSS |
|---|--:|--:|--:|--:|
| **zkFOL doubled mod** | **4.82 ms** | **1.4 ms** | **49.8 KB** | 376 MB |
| RISC Zero composite + fastdbl | 3.69 s | 11.8 ms | 209.6 KB | 312 MB |
| RISC Zero succinct + fastdbl | 14.73 s | 12.4 ms | 223.3 KB | 1.39 GB |
| SP1 core + fastdbl | 13.32 s | 74.3 ms | 2.78 MB | 9.39 GB |
| SP1 compressed + fastdbl | 49.24 s | 32.9 ms | 1.27 MB | 17.00 GB |

Prove, zkVM over zkFOL: 766× / 3,060× / 2,760× / 10,200×. Peak RSS: 0.8× / 3.7× / 25× / 45×.

## 2. The floor: a bounds check

Prove that a committed x lies in [10, 100]. Nothing to compute, so this is the price of
a proof at all. RISC Zero charges the same 32,768 cycles here as for fastdbl and for
n = 1; the zkVM columns are unchanged from section 1 because the work never left the floor.

![prover time and peak RSS, bounds check](prove_bounds.svg)

| system | prove | verify | proof | peak RSS |
|---|--:|--:|--:|--:|
| **zkFOL bounds** | **1.53 ms** | **0.54 ms** | **28.0 KB** | 274 MB |
| RISC Zero composite + bounds | 3.70 s | 11.5 ms | 209.6 KB | 311 MB |
| RISC Zero succinct + bounds | 14.68 s | 12.3 ms | 223.2 KB | 1.39 GB |
| SP1 core + bounds | 13.23 s | 78.2 ms | 2.78 MB | 9.36 GB |
| SP1 compressed + bounds | 49.20 s | 32.6 ms | 1.27 MB | 17.01 GB |

Prove, zkVM over zkFOL: 2,420× / 9,600× / 8,650× / 32,200×. Peak RSS: 1.1× / 5.1× / 34× / 62×.

## 3. Each system's default route

The zkVMs run the zkbenchmarks.com program unchanged: a linear loop, n additions mod
7919, in their headline STARK mode. zkFOL runs the same linear register recurrence
(`regsm`, below) and, for scale, the same two routes on the exact 2,090-digit integer.

| system | what is proved | prove | verify | proof | peak RSS |
|---|---|--:|--:|--:|--:|
| zkFOL registers mod | fib(10,000) mod 7919, linear | 1.06 s | 31.2 ms | 866 KB | 562 MB |
| zkFOL doubled mod | fib(10,000) mod 7919, doubling | 4.82 ms | 1.4 ms | 49.8 KB | 376 MB |
| zkFOL registers | exact fib(10,000), linear | 3.65 s | 820.6 ms | 15.5 MB | 1.60 GB |
| zkFOL doubled | exact fib(10,000), doubling | 8.36 ms | 3.1 ms | 664 KB | 311 MB |
| RISC Zero succinct | fib(10,000) mod 7919, linear | 40.67 s | 12.5 ms | 223.3 KB | 2.30 GB |
| SP1 compressed | fib(10,000) mod 7919, linear | 50.39 s | 35.2 ms | 1.27 MB | 17.07 GB |

Read the loss column too: on the linear route zkFOL's verify (31 ms, 820 ms) and proof
(866 KB, 15.5 MB) grow with the trace, while the zkVM's are constant. The doubling
rewrite is what removes that; without it the exact-integer linear route is the slowest
zkFOL row in every column but prove.

## What each side wrote

The claim in section 1, as the zkVM guest computes it and as zkFOL states it.

<table><tr><th>RISC Zero / SP1 guest (hand-written doubling)</th><th>zkFOL source (the rewrite finds the doubling)</th></tr>
<tr><td>

```rust
fn fast_doubling(n: u32) -> (u32, u32) {
    let (mut a, mut b): (u64, u64) = (0, 1);
    for i in (0..32).rev() {
        let c = (a * ((2 * b + M - a) % M)) % M;
        let d = (a * a + b * b) % M;
        if (n >> i) & 1 == 0 {
            a = c;
            b = d;
        } else {
            a = d;
            b = (c + d) % M;
        }
    }
    (a as u32, b as u32)
}
```

</td><td>

```elixir
fibm(1, 1)
fibm(2, 1)

fibm(x, v) do
  x > 2
  fibm(x - 1, v1)
  fibm(x - 2, v2)
  v = mod(v1 + v2, 7919)
end
```

`Doubling.rewrite(fibm, 10_000)`

</td></tr></table>

The floor claim (section 2):

<table><tr><th>guest</th><th>zkFOL</th></tr>
<tr><td>

```rust
assert!((10..=100).contains(&n));
```

</td><td>

```elixir
bounded(x) do
  x > 9
  x < 101
end
```

</td></tr></table>

The default route (section 3), the linear loop both sides run:

<table><tr><th>guest</th><th>zkFOL</th></tr>
<tr><td>

```rust
let (mut a, mut b) = (0u32, 1u32);
for _ in 0..n {
    let mut c = a + b;
    c %= 7919;
    a = b;
    b = c;
}
```

</td><td>

```elixir
regsm(1, 1, 1, 0)

regsm(x, a, b, q) do
  x > 1
  regsm(x - 1, a1, b1, q1)
  a1 + b1 = q * 7919 + a
  a < 7919
  a + 1 > 0
  q + 1 > 0
  b = a1
end
```

</td></tr></table>

## How memory is measured

Peak RSS is the whole process, for everyone: `ru_maxrss` of the zkVM host binary, `VmHWM`
of the Erlang VM running zkFOL. zkFOL's number is therefore mostly the BEAM at rest
(230 to 380 MB depending on what ran before); the prover's own increment over that
baseline is within measurement noise on every row except exact fib(10,000) on the linear
route, which adds 1.2 GB. So the honest reading of the memory column is: at the floor,
zkFOL and RISC Zero composite are the same size, and everything else is larger.

## Caveats that ship with the tables

- **Prove is the claim, not verify or proof size in general.** zkFOL wins those only
  where the trace is short (doubling, bounds). On a long linear trace the zkVM's constant
  receipt wins (section 3).
- **CPU only, one box.** zkVM prove times are hardware- and contention-sensitive (RISC
  Zero fib(1000) measured 33 s contended vs 18 s uncontended on this machine). SP1's ~1 s
  setup is excluded from its prove column.
- **Groth16 wraps are not measured** (Docker); vendor figures are unverified here. Wrapped,
  RISC Zero's receipt is 521 bytes with ~3 ms verify, at several times the prove time.
- **Security levels are not normalised.** zkFOL's field on this lineage is a fixed
  secp256k1 projecting prime, so its proofs are honest-prover-only; the zkVMs target
  production soundness. zkFOL's guard slack is oracle-checked, not field-enforced.
- **Out of bounds is unprovable on both sides**: zkFOL refuses (`no_answer`), the guest
  panics and no receipt is produced.
- **Expressiveness is the real boundary.** A zkVM runs anything. For a claim zkFOL cannot
  state, it is not in the race.

## Reproduce

zkVM cells: `./bench_all.sh` runs every (system, mode, n) sequentially and prints one
`CELL` line each, median of 3, with peak RSS. Rows above are the cells
`<system> <mode>+fastdbl 10000`, `<system> <mode>+bounds 42`, and `<system> <mode> 10000`.

zkFOL rows, one call each, in the zkfol repo at 0.4.0:

| row | call |
|---|---|
| doubled mod | `Examples.EBench.measured_doubled_fibonacci_mod(10_000)` |
| bounds | `Examples.EBench.measured_bounds_check()` |
| registers mod | `Examples.EBench.measured_registers_fibonacci_mod(10_000)` |
| registers | `Examples.EBench.measured_registers_fibonacci(10_000)` |
| doubled | `Examples.EBench.measured_doubled_fibonacci(10_000)` |

```sh
MIX_ENV=test mix run -e 'Examples.EBench.measured_doubled_fibonacci_mod(10_000) |> IO.inspect()'
```

The figures are drawn from the tables by `./chart.py`.
