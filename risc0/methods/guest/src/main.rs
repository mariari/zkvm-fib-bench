use risc0_zkvm::guest::env;

// Fibonacci benchmark guest.
// Mirrors the SP1 official fibonacci example exactly so the two zkVMs run the
// identical computation: iterate n times, tracking (a, b) mod 7919.
//
// Reads two words: `n`, then `algo` (0 = linear, 1 = fast doubling).  Both
// algorithms commit the same journal (n, F(n) mod 7919, F(n+1) mod 7919), so
// they are directly comparable -- only the number of guest cycles differs.

const M: u64 = 7919;

/// Linear recurrence: n additions mod 7919. The zkbenchmarks.com program.
fn linear(n: u32) -> (u32, u32) {
    let mut a: u32 = 0;
    let mut b: u32 = 1;
    for _ in 0..n {
        let mut c = a + b;
        c %= 7919; // modulus to prevent overflow, same as SP1's example
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

fn main() {
    let n: u32 = env::read();
    let algo: u32 = env::read();

    // Commit the input n to the journal (public output).
    env::commit(&n);

    let (a, b) = if algo == 0 { linear(n) } else { fast_doubling(n) };

    // Commit the results.
    env::commit(&a);
    env::commit(&b);
}
