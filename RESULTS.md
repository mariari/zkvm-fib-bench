# Benchmarks: zkFOL vs RISC0 vs SP1

We benchmark the same computations on zkFOL, RISC Zero and SP1:

1. **fib(10,000) mod 7919.** Fibonacci run to 10,000, the exact same algorithm on every
   system.
2. **bounds check 10 ≤ x ≤ 100.** Just the cost of constraining the given argument.
3. **fib(10,000).** What just running Fibonacci looks like for your average use case.

Each section shows the code each side wrote, complete with imports, then its numbers. The
zkFOL definitions the benchmark ran are collected in [`zkfol/definitions.ex`](zkfol/definitions.ex).

**Machine:** AMD Ryzen 7 5700X (8c/16t), Linux, CPU proving only, one prover at a time.
**Versions:** zkFOL 0.4.0 (zinc-plus `451ba17`), RISC Zero 3.0.5, SP1 6.3.1. Measured August 2026.

## Glossary

- **Linear.** Compute fib by walking the recurrence n times: `a, b = b, a + b`. Ten
  thousand steps for n = 10,000.
- **Fast doubling.** Jump instead of walk. From the pair (F(k), F(k+1)) two identities
  give the pair at 2k directly, F(2k) = F(k)(2F(k+1) − F(k)) and F(2k+1) = F(k)² + F(k+1)²,
  so the index doubles each round and n = 10,000 takes 14 rounds instead of 10,000 steps.
  Both algorithms prove the same claim; they differ only in how much work the proof has to
  cover.
- **RISC Zero composite / succinct.** Composite is the raw proof, one STARK per execution
  segment; succinct recursively folds those into one constant-size STARK.
- **SP1 core / compressed.** The same pair for SP1: core is the raw shard proofs,
  compressed recursively folds them into one.

## 1. fib(10,000) mod 7919

Everyone proves fib(10,000) mod 7919 by fast doubling, 14 rounds instead of 10,000 steps.
The zkVM guests use a hand-written `fast_doubling` (source below). zkFOL is handed
`defrel fibm`, the naive two-call recurrence, and `Zkfol.Doubling` rewrites it to the
doubling form.

![prover time and prover memory, fib(10,000) mod 7919, fast doubling](prove_fib10000.svg)

<table><tr><th><a href="risc0/methods/guest/src/main.rs#L32-L46">RISC Zero guest</a> (hand-written doubling)</th><th><a href="zkfol/definitions.ex#L20-L28">zkFOL</a> (the compiler finds the doubling)</th></tr>
<tr><td>

```rust
use risc0_zkvm::guest::env;

const M: u64 = 7919;

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

fn main() {
    let n: u32 = env::read();
    env::commit(&n);
    let (a, b) = fast_doubling(n);
    env::commit(&a);
    env::commit(&b);
}
```

</td><td>

```elixir
defmodule Fibonacci do
  use Zkfol.Lang

  defrel fibm(1, 1)
  defrel fibm(2, 1)

  defrel fibm(x, v) do
    x > 2
    fibm(x - 1, v1)
    fibm(x - 2, v2)
    v = mod(v1 + v2, 7919)
  end
end
```

</td></tr></table>

| system                                                                   |       prove |     verify |       proof | prover memory |
|--------------------------------------------------------------------------|------------:|-----------:|------------:|--------------:|
| **[zkFOL `fibm`, doubled](zkfol/definitions.ex#L20-L28)**                | **4.82 ms** | **1.4 ms** | **49.8 KB** |   **< 10 MB** |
| [RISC Zero composite + fastdbl](risc0/methods/guest/src/main.rs#L32-L46) |      3.69 s |    11.8 ms |    209.6 KB |        312 MB |
| [RISC Zero succinct + fastdbl](risc0/methods/guest/src/main.rs#L32-L46)  |     14.73 s |    12.4 ms |    223.3 KB |       1.39 GB |
| [SP1 core + fastdbl](sp1/program/src/main.rs#L30-L44)                    |     13.32 s |    74.3 ms |     2.78 MB |       9.39 GB |
| [SP1 compressed + fastdbl](sp1/program/src/main.rs#L30-L44)              |     49.24 s |    32.9 ms |     1.27 MB |      17.00 GB |

Prove, zkVM over zkFOL: 766× / 3,060× / 2,760× / 10,200×. Memory: over 30× / 130× / 900× / 1,700×.

## 2. The floor: a bounds check

Prove that a committed x lies in [10, 100]. Nothing to compute, so this is the price of
a proof at all. RISC Zero charges the same 32,768 cycles here as for fast doubling and for
n = 1; the zkVM columns are unchanged from section 1 because the work never left the floor.

![prover time and prover memory, bounds check](prove_bounds.svg)

<table><tr><th><a href="risc0/methods/guest/src/main.rs#L59">RISC Zero guest</a></th><th><a href="zkfol/definitions.ex#L51-L54">zkFOL</a></th></tr>
<tr><td>

```rust
use risc0_zkvm::guest::env;

fn main() {
    let x: u32 = env::read();
    env::commit(&x);
    assert!((10..=100).contains(&x), "x out of bounds: {}", x);
}
```

</td><td>

```elixir
defmodule Bounds do
  use Zkfol.Lang

  defrel bounded(x) do
    x > 9
    x < 101
  end
end
```

</td></tr></table>

| system                                                              |       prove |      verify |       proof | prover memory |
|---------------------------------------------------------------------|------------:|------------:|------------:|--------------:|
| **[zkFOL `bounded`](zkfol/definitions.ex#L51-L54)**                 | **1.53 ms** | **0.54 ms** | **28.0 KB** |     **~3 MB** |
| [RISC Zero composite + bounds](risc0/methods/guest/src/main.rs#L59) |      3.70 s |     11.5 ms |    209.6 KB |        311 MB |
| [RISC Zero succinct + bounds](risc0/methods/guest/src/main.rs#L59)  |     14.68 s |     12.3 ms |    223.2 KB |       1.39 GB |
| [SP1 core + bounds](sp1/program/src/main.rs#L56)                    |     13.23 s |     78.2 ms |     2.78 MB |       9.36 GB |
| [SP1 compressed + bounds](sp1/program/src/main.rs#L56)              |     49.20 s |     32.6 ms |     1.27 MB |      17.01 GB |

Prove, zkVM over zkFOL: 2,420× / 9,600× / 8,650× / 32,200×. Memory: 100× / 460× / 3,100× / 5,700×.

## 3. Each system's default route

The zkVMs run the zkbenchmarks.com program unchanged: the linear loop, in their headline
proof mode. zkFOL runs `defrel regsm`, the same linear loop as a relation, for a like-for-like
row, and `defrel fib`, the way a user would write it, on the exact 2,090-digit integer.

![prover time and prover memory, fib(10,000) on each system's default route](prove_default.svg)

<table><tr><th><a href="risc0/methods/guest/src/main.rs#L16-L26">RISC Zero guest</a></th><th><a href="zkfol/definitions.ex">zkFOL</a></th></tr>
<tr><td>

```rust
use risc0_zkvm::guest::env;

fn main() {
    let n: u32 = env::read();
    env::commit(&n);
    let (mut a, mut b) = (0u32, 1u32);
    for _ in 0..n {
        let mut c = a + b;
        c %= 7919;
        a = b;
        b = c;
    }
    env::commit(&a);
    env::commit(&b);
}
```

</td><td>

```elixir
defmodule Fibonacci do
  use Zkfol.Lang

  defrel regsm(1, 1, 1)

  defrel regsm(x, a, b) do
    x > 1
    regsm(x - 1, b, b1)
    a = mod(b + b1, 7919)
  end

  defrel fib(1, 1)
  defrel fib(2, 1)

  defrel fib(x, v) do
    x > 2
    fib(x - 1, v1)
    fib(x - 2, v2)
    v = v1 + v2
  end
end
```

</td></tr></table>

| system                                                        | what is proved               |   prove |  verify |    proof | prover memory |
|---------------------------------------------------------------|------------------------------|--------:|--------:|---------:|--------------:|
| [zkFOL `regsm`](zkfol/definitions.ex#L42-L48)                 | fib(10,000) mod 7919, linear |  1.06 s | 31.2 ms |   866 KB |       < 10 MB |
| [zkFOL `fib`, doubled](zkfol/definitions.ex#L9-L17)           | exact fib(10,000), doubling  | 8.36 ms |  3.1 ms |   664 KB |       < 10 MB |
| [RISC Zero succinct](risc0/methods/guest/src/main.rs#L16-L26) | fib(10,000) mod 7919, linear | 40.67 s | 12.5 ms | 223.3 KB |       2.30 GB |
| [SP1 compressed](sp1/program/src/main.rs#L14-L24)             | fib(10,000) mod 7919, linear | 50.39 s | 35.2 ms |  1.27 MB |      17.07 GB |

Read the loss column too: on `regsm` zkFOL's verify (31 ms) and proof (866 KB) grow with
the number of steps, while the zkVM's stay constant. `fib` has neither problem because
the compiler rewrites it to doubling, which is why that is the route a user gets by
default.

## How memory is measured

The zkVM column is the peak RSS of the host process (`ru_maxrss`), which is the prover.
zkFOL's prover runs inside the Erlang VM as a NIF, and the VM idles at 230 to 380 MB
before any proof, so its column is what the prover itself added: the VM's high-water
mark after the peak counter is reset at entry, less its resident size at that moment.
That increment is under the ~10 MB measurement noise on every row but exact fib(10,000)
on the linear route, which adds 1.23 GB. Whole-process peaks, for reference: 274 to
562 MB on the small rows, 1.6 GB on that one.

## Caveats that ship with the tables

- **CPU only, one box.** zkVM prove times are hardware- and contention-sensitive (RISC
  Zero fib(1000) measured 33 s contended vs 18 s uncontended on this machine). SP1's ~1 s
  setup is excluded from its prove column.
- **Groth16 wraps are not measured** (Docker); vendor figures are unverified here. Wrapped,
  RISC Zero's receipt is 521 bytes with ~3 ms verify, at several times the prove time.
- **Out of bounds is unprovable on both sides**: zkFOL refuses (`no_answer`), the guest
  panics and no receipt is produced.

## Reproduce

zkVM cells: `./bench_all.sh` runs every (system, mode, n) sequentially and prints one
`CELL` line each, median of 3, with peak RSS. Rows above are the cells
`<system> <mode>+fastdbl 10000`, `<system> <mode>+bounds 42`, and `<system> <mode> 10000`.

zkFOL rows, one call each, in the zkfol repo at 0.4.0. The definition column links the
relation in [`zkfol/definitions.ex`](zkfol/definitions.ex); the call column is the measuring
example in `Examples.EBench`.

| row             | definition                                                                             | call                                       |
|-----------------|----------------------------------------------------------------------------------------|--------------------------------------------|
| `fibm`, doubled | [`fibm`](zkfol/definitions.ex#L20-L28), rewritten by `Zkfol.Doubling`                  | `measured_doubled_fibonacci_mod(10_000)`   |
| `bounded`       | [`bounded`](zkfol/definitions.ex#L51-L54)                                              | `measured_bounds_check()`                  |
| `regsm`         | [`regsm`](zkfol/definitions.ex#L42-L48)                                                | `measured_registers_fibonacci_mod(10_000)` |
| `regs`          | [`regs`](zkfol/definitions.ex#L32-L38), the exact integer                              | `measured_registers_fibonacci(10_000)`     |
| `fib`, doubled  | [`fib`](zkfol/definitions.ex#L9-L17), the exact integer, rewritten by `Zkfol.Doubling` | `measured_doubled_fibonacci(10_000)`       |

The definitions in `zkfol/definitions.ex` are for reading; they run inside the zkfol
application, from its shell:

```sh
iex -S mix
```

```elixir
Examples.EBench.measured_doubled_fibonacci_mod(10_000)
```

The figures are drawn from the tables by `elixir chart.exs`.









