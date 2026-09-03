//! Jolt guest: fib(n) mod 7919 -- the exact program used by zkbenchmarks.com.
//!
//! Line for line the same arithmetic as `sp1/program/src/main.rs` and the
//! risc0 guest, so the three VMs prove the same claim. Takes two words: `n`,
//! then `algo` (0 = linear, 1 = fast doubling, 2 = bounds check). Algorithms
//! 0 and 1 return the same pair, so they are directly comparable -- only the
//! number of guest cycles differs.
//!
//! Jolt has no `commit`: a `#[jolt::provable]` function's arguments and return
//! value ARE the public I/O, so returning `(F(n) mod 7919, F(n+1) mod 7919)`
//! with `n` as an argument gives the same journal the other two commit.
//! Algorithm 2 is the exception in shape as well as claim -- it reads `n` as
//! the value x, enforces 10 <= x <= 100, and its return carries nothing
//! further, so it returns `(x, 0)` where the others return the pair.
//!
//! `max_trace_length` is a compile-time constant, unlike risc0 and SP1 which
//! size the trace at run time. It must exceed the linear program's cycle count
//! at the largest n in the sweep; 2^21 covers n=100000 with room. See the note
//! in the README: if Jolt sizes the prover to this bound rather than to the
//! actual trace, the n=1000 and n=10000 rows are not comparable and this wants
//! tuning per n.
#![cfg_attr(feature = "guest", no_std)]

const M: u64 = 7919;

#[jolt::provable(heap_size = 32768, max_trace_length = 2097152)]
fn fib(n: u32, algo: u32) -> (u32, u32) {
    // Bounds check: the smallest real claim, a single range assertion on x.
    // A violation panics, which makes the guest fail and no proof is produced.
    if algo == 2 {
        assert!(n >= 10 && n <= 100, "x out of bounds");
        return (n, 0);
    }

    if algo == 0 {
        linear(n)
    } else {
        fast_doubling(n)
    }
}

/// Linear recurrence: n additions mod 7919. The zkbenchmarks.com program.
fn linear(n: u32) -> (u32, u32) {
    let mut a: u32 = 0;
    let mut b: u32 = 1;
    for _ in 0..n {
        let mut c = a + b;
        c %= 7919; // modulus to prevent overflow
        a = b;
        b = c;
    }
    (a, b)
}

/// Fast doubling: ~log2(n) iterations of
///   F(2k)   = F(k) * (2*F(k+1) - F(k))
///   F(2k+1) = F(k)^2 + F(k+1)^2
/// all mod 7919. Operands stay < 7919, so the products fit in u64 with room.
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
