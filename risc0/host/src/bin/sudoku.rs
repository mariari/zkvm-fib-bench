// RISC Zero sudoku-validity benchmark host, generic over grid size n.
//
// Mirrors the fib host (host/src/main.rs): same ProverOpts modes, same separate
// prove/verify timing, same BENCH line. The guest proves that a completed n x n
// grid is a valid sudoku via power sums (see methods/guest/src/bin/sudoku.rs).
//
// Usage: sudoku [n] [succinct|composite|groth16]   (defaults: n=9, succinct)
//        n must be a perfect square (4, 9, 16, ...). The host generates a known-
//        valid grid of that size and writes (n, grid) to the guest the way the
//        fib host writes its argument. Both are committed, so the proof attests
//        "this public n x n grid is a valid sudoku".
use methods::{SUDOKU_ELF, SUDOKU_ID};
use risc0_zkvm::{default_prover, ExecutorEnv, ProverOpts};
use std::time::Instant;

/// A known-valid solved sudoku of size n (n a perfect square), row-major, values
/// 1..n. The canonical base pattern value(r,c) = (b*(r%b) + r/b + c) mod n is a
/// valid sudoku for every n = b*b, so one formula serves all sizes.
fn valid_grid(n: usize) -> Vec<u32> {
    let b = (1..=n).find(|b| b * b >= n).unwrap();
    assert_eq!(b * b, n, "n must be a perfect square");
    let mut g = vec![0u32; n * n];
    for r in 0..n {
        for c in 0..n {
            g[n * r + c] = ((b * (r % b) + r / b + c) % n) as u32 + 1;
        }
    }
    g
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    let args: Vec<String> = std::env::args().collect();
    let n: u32 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(9);
    let mode = args
        .get(2)
        .map(|s| s.to_lowercase())
        .unwrap_or_else(|| "succinct".to_string());

    let opts = match mode.as_str() {
        "composite" => ProverOpts::composite(),
        "succinct" => ProverOpts::succinct(),
        "groth16" => ProverOpts::groth16(),
        other => {
            eprintln!("unknown mode '{other}', falling back to succinct");
            ProverOpts::succinct()
        }
    };

    let grid: Vec<u32> = valid_grid(n as usize);
    let env = ExecutorEnv::builder()
        .write(&n)
        .unwrap()
        .write(&grid)
        .unwrap()
        .build()
        .unwrap();
    let prover = default_prover();

    // ---- prove ----
    let t = Instant::now();
    let prove_info = prover.prove_with_opts(env, SUDOKU_ELF, &opts).unwrap();
    let prove_s = t.elapsed().as_secs_f64();

    let receipt = prove_info.receipt;
    let proof_bytes = bincode::serialize(&receipt).map(|v| v.len()).unwrap_or(0);

    // ---- verify (the part zkbenchmarks.com does not measure) ----
    let t = Instant::now();
    receipt.verify(SUDOKU_ID).unwrap();
    let verify_ms = t.elapsed().as_secs_f64() * 1000.0;

    // sanity-check output: the guest commits (n, grid) it proved valid
    let (n_out, grid_out): (u32, Vec<u32>) = receipt.journal.decode().unwrap_or_default();
    let ok = n_out == n && grid_out == grid;

    println!(
        "BENCH risc0 bench=sudoku n={n} mode={mode} prove_s={prove_s:.3} verify_ms={verify_ms:.3} proof_bytes={proof_bytes} grid_ok={ok} stats={:?}",
        prove_info.stats
    );
}
