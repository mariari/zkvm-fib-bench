use risc0_zkvm::guest::env;

// Sudoku-validity benchmark guest (RISC Zero), generic over an n x n grid.
//
// Proves that a completed n x n grid (n a perfect square, box side b = sqrt(n))
// is a valid sudoku: each of the 3n groups -- n rows, n columns, n non-overlapping
// b x b boxes -- is a permutation of {1,...,n}.
//
// The check is the direct, idiomatic one you would write in plain Rust: for each
// group, walk the n cells with a `seen` array, asserting every value is in 1..=n
// and appears exactly once. Plain integer/boolean ops, no modular arithmetic and
// no overflow concerns -- cell values are small. Each system proves "valid
// sudoku" in its own natural idiom; the zkVM's is this distinctness check.

/// Box side b = sqrt(n); n must be a perfect square.
fn box_side(n: usize) -> usize {
    let mut b = 0;
    while (b + 1) * (b + 1) <= n {
        b += 1;
    }
    assert_eq!(b * b, n, "n must be a perfect square");
    b
}

/// The n cells of every group: n rows, then n columns, then n boxes.
fn groups(grid: &[u32], n: usize, b: usize) -> Vec<Vec<u32>> {
    let mut g = vec![vec![0u32; n]; 3 * n];
    for r in 0..n {
        for c in 0..n {
            let val = grid[n * r + c];
            g[r][c] = val; // row r
            g[n + c][r] = val; // column c
            let box_id = b * (r / b) + c / b;
            let pos = b * (r % b) + c % b;
            g[2 * n + box_id][pos] = val; // box box_id
        }
    }
    g
}

/// A group is a permutation of 1..=n iff every value lands in range once.
fn is_permutation(cells: &[u32], n: usize) -> bool {
    let mut seen = vec![false; n];
    for &v in cells {
        if v < 1 || v as usize > n {
            return false;
        }
        let idx = v as usize - 1;
        if seen[idx] {
            return false;
        }
        seen[idx] = true;
    }
    seen.iter().all(|&s| s)
}

fn main() {
    // Size n, then the n*n cell values row-major. The host commits both, so the
    // proof attests "this public grid of this size is a valid sudoku".
    let n: u32 = env::read();
    let grid: Vec<u32> = env::read();
    env::commit(&n);
    env::commit(&grid);

    let n = n as usize;
    assert_eq!(grid.len(), n * n, "grid must have n*n cells");
    let b = box_side(n);

    for group in groups(&grid, n, b).iter() {
        assert!(is_permutation(group, n), "group is not a permutation of 1..n");
    }
}
