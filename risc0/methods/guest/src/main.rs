use risc0_zkvm::guest::env;

// Fibonacci benchmark guest.
// Mirrors the SP1 official fibonacci example exactly so the two zkVMs run the
// identical computation: iterate n times, tracking (a, b) mod 7919.
fn main() {
    let n: u32 = env::read();

    // Commit the input n to the journal (public output).
    env::commit(&n);

    let mut a: u32 = 0;
    let mut b: u32 = 1;
    for _ in 0..n {
        let mut c = a + b;
        c %= 7919; // modulus to prevent overflow, same as SP1's example
        a = b;
        b = c;
    }

    // Commit the results.
    env::commit(&a);
    env::commit(&b);
}
