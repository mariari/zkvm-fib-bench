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


/// The same public 9x9 Sudoku claim used by the RISC Zero and SP1 guests.
#[jolt::provable(heap_size = 32768, max_trace_length = 2097152)]
pub fn sudoku(n: u32, grid: [[u32; 9]; 9]) -> bool {
    assert!(n == 9, "this benchmark is fixed at 9x9");

    for row in 0..9 {
        let mut seen = [false; 9];
        for column in 0..9 {
            let value = grid[row][column];
            assert!(value >= 1 && value <= 9, "cell out of range");
            let slot = value as usize - 1;
            assert!(!seen[slot], "duplicate in row");
            seen[slot] = true;
        }
    }
    for column in 0..9 {
        let mut seen = [false; 9];
        for row in 0..9 {
            let value = grid[row][column];
            assert!(value >= 1 && value <= 9, "cell out of range");
            let slot = value as usize - 1;
            assert!(!seen[slot], "duplicate in column");
            seen[slot] = true;
        }
    }
    for box_row in 0..3 {
        for box_column in 0..3 {
            let mut seen = [false; 9];
            for row in 0..3 {
                for column in 0..3 {
                    let value = grid[box_row * 3 + row][box_column * 3 + column];
                    assert!(value >= 1 && value <= 9, "cell out of range");
                    let slot = value as usize - 1;
                    assert!(!seen[slot], "duplicate in box");
                    seen[slot] = true;
                }
            }
        }
    }
    true
}
