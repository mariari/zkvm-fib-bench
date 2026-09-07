#![cfg_attr(feature = "guest", no_std)]

const MODULUS: u32 = 7919;

/// The same linear recurrence used by the RISC Zero and SP1 guests.
#[jolt::provable(heap_size = 32768, max_trace_length = 2097152)]
pub fn fib(n: u32) -> (u32, u32) {
    let (mut a, mut b) = (0u32, 1u32);

    for _ in 0..n {
        let c = (a + b) % MODULUS;
        a = b;
        b = c;
    }

    (a, b)
}
