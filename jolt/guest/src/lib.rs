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


fn check_sudoku<const N: usize, const B: usize>(grid: [[u32; N]; N]) -> bool {
    for row in 0..N {
        let mut seen = [false; N];
        for column in 0..N {
            let value = grid[row][column];
            assert!(value >= 1 && value <= N as u32, "cell out of range");
            let slot = value as usize - 1;
            assert!(!seen[slot], "duplicate in row");
            seen[slot] = true;
        }
    }
    for column in 0..N {
        let mut seen = [false; N];
        for row in 0..N {
            let value = grid[row][column];
            assert!(value >= 1 && value <= N as u32, "cell out of range");
            let slot = value as usize - 1;
            assert!(!seen[slot], "duplicate in column");
            seen[slot] = true;
        }
    }
    for box_row in 0..B {
        for box_column in 0..B {
            let mut seen = [false; N];
            for row in 0..B {
                for column in 0..B {
                    let value = grid[box_row * B + row][box_column * B + column];
                    assert!(value >= 1 && value <= N as u32, "cell out of range");
                    let slot = value as usize - 1;
                    assert!(!seen[slot], "duplicate in box");
                    seen[slot] = true;
                }
            }
        }
    }
    true
}

/// Exact-size public-grid guests matching the repository's 4x4, 9x9, 16x16 cases.
#[jolt::provable(heap_size = 32768, max_trace_length = 524288)]
pub fn sudoku4(n: u32, grid: [[u32; 4]; 4]) -> bool {
    assert!(n == 4, "wrong Sudoku size");
    check_sudoku::<4, 2>(grid)
}

#[jolt::provable(heap_size = 32768, max_trace_length = 2097152)]
pub fn sudoku9(n: u32, grid: [[u32; 9]; 9]) -> bool {
    assert!(n == 9, "wrong Sudoku size");
    check_sudoku::<9, 3>(grid)
}

#[jolt::provable(stack_size = 65536, heap_size = 32768, max_trace_length = 8388608)]
pub fn sudoku16(n: u32, grid: [[u32; 16]; 16]) -> bool {
    assert!(n == 16, "wrong Sudoku size");
    check_sudoku::<16, 4>(grid)
}
