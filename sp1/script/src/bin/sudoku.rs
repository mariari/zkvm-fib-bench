// SP1 sudoku-validity benchmark host, generic over grid size n.
//
// Mirrors the fib host (script/src/main.rs): same prover modes, same execute /
// setup / prove / verify timing, same BENCH line. The guest proves that a
// completed n x n grid is a valid sudoku via power sums (see program/src/bin/sudoku.rs).
//
// Usage: cargo run --release --bin sudoku -- [n] [core|compressed|groth16|plonk]
//        (defaults: n=9, compressed). n must be a perfect square (4, 9, 16, ...).
//        The host generates a known-valid grid of that size and writes (n, grid)
//        to the guest, both committed, so the proof attests "this public n x n
//        grid is a valid sudoku".
use sp1_sdk::prelude::*;
use sp1_sdk::ProverClient;
use std::time::Instant;

/// The sudoku guest ELF (program/src/bin/sudoku.rs).
const ELF: Elf = include_elf!("sudoku");

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

#[tokio::main]
async fn main() {
    sp1_sdk::utils::setup_logger();

    let args: Vec<String> = std::env::args().collect();
    let n: u32 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(9);
    let mode = args
        .get(2)
        .map(|s| s.to_lowercase())
        .unwrap_or_else(|| "compressed".to_string());

    let grid: Vec<u32> = valid_grid(n as usize);
    let mut stdin = SP1Stdin::new();
    stdin.write(&n);
    stdin.write(&grid);

    let client = ProverClient::from_env().await;

    // ---- execute (no proof) to get the cycle count ----
    let (_pv, report) = client.execute(ELF, stdin.clone()).await.unwrap();
    let cycles = report.total_instruction_count();

    // ---- setup (proving/verifying keys) ----
    let t = Instant::now();
    let pk = client.setup(ELF).await.unwrap();
    let setup_s = t.elapsed().as_secs_f64();
    let vk = pk.verifying_key();

    // ---- prove ----
    let t = Instant::now();
    let proof = match mode.as_str() {
        "core" => client.prove(&pk, stdin.clone()).core().await.unwrap(),
        "compressed" => client.prove(&pk, stdin.clone()).compressed().await.unwrap(),
        "groth16" => client.prove(&pk, stdin.clone()).groth16().await.unwrap(),
        "plonk" => client.prove(&pk, stdin.clone()).plonk().await.unwrap(),
        other => {
            eprintln!("unknown mode '{other}', falling back to compressed");
            client.prove(&pk, stdin.clone()).compressed().await.unwrap()
        }
    };
    let prove_s = t.elapsed().as_secs_f64();

    // ---- proof size ----
    let tmp = std::env::temp_dir().join(format!("sp1_sudoku_{mode}.bin"));
    let proof_bytes = match proof.save(&tmp) {
        Ok(()) => std::fs::metadata(&tmp).map(|m| m.len()).unwrap_or(0),
        Err(_) => 0,
    };
    let onchain_bytes: u64 = if mode == "groth16" || mode == "plonk" {
        proof.bytes().len() as u64
    } else {
        0
    };

    // ---- verify (the part zkbenchmarks.com does not measure) ----
    let t = Instant::now();
    client.verify(&proof, vk, None).expect("verification failed");
    let verify_ms = t.elapsed().as_secs_f64() * 1000.0;

    println!(
        "BENCH sp1 bench=sudoku n={n} mode={mode} cycles={cycles} setup_s={setup_s:.3} prove_s={prove_s:.3} verify_ms={verify_ms:.3} proof_bytes={proof_bytes} onchain_bytes={onchain_bytes}"
    );
}
