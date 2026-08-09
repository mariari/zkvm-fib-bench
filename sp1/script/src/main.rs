// SP1 fib benchmark host.
//
// Mirrors the zkbenchmarks.com SP1 harness (proves `fib(n) mod 7919` with
// .compressed() / .groth16()), but ADDS what the site omits: it times prove
// and verify separately and reports cycle count + proof sizes.
//
// Usage: cargo run --release -- <n> [core|compressed|groth16|plonk][+fastdbl|+bounds]
//        (defaults: n=10000, compressed). The `+fastdbl` suffix switches the
//        guest from the linear recurrence to fast doubling (same journal).
//        The `+bounds` suffix runs the bounds check instead, in which case the
//        first argument is not a loop count but the value x (default 42).
use sp1_sdk::prelude::*;
use sp1_sdk::ProverClient;
use std::time::Instant;

/// The ELF we want to execute inside the zkVM.
const ELF: Elf = include_elf!("fibonacci-program");

#[tokio::main]
async fn main() {
    sp1_sdk::utils::setup_logger();

    let args: Vec<String> = std::env::args().collect();
    let mode = args
        .get(2)
        .map(|s| s.to_lowercase())
        .unwrap_or_else(|| "compressed".to_string());

    // `<prover>+fastdbl` and `<prover>+bounds` select the guest algorithm.
    let algo: u32 = if mode.ends_with("+bounds") {
        2
    } else if mode.ends_with("+fastdbl") {
        1
    } else {
        0
    };
    let prover_mode = mode.trim_end_matches("+fastdbl").trim_end_matches("+bounds");

    // The bounds check has no loop count, so it reuses the first argument as x.
    let n: u32 = args
        .get(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(if algo == 2 { 42 } else { 10_000 });

    let mut stdin = SP1Stdin::new();
    stdin.write(&n);
    stdin.write(&algo);

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
    let proof = match prover_mode {
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
    let tmp = std::env::temp_dir().join(format!("sp1_fib_{mode}.bin"));
    let proof_bytes = match proof.save(&tmp) {
        Ok(()) => std::fs::metadata(&tmp).map(|m| m.len()).unwrap_or(0),
        Err(_) => 0,
    };
    // on-chain proof size is only meaningful for the SNARK modes
    let onchain_bytes: u64 = if prover_mode == "groth16" || prover_mode == "plonk" {
        proof.bytes().len() as u64
    } else {
        0
    };

    // ---- verify (the part zkbenchmarks.com does not measure) ----
    let t = Instant::now();
    client.verify(&proof, vk, None).expect("verification failed");
    let verify_ms = t.elapsed().as_secs_f64() * 1000.0;

    println!(
        "BENCH sp1 mode={mode} n={n} cycles={cycles} setup_s={setup_s:.3} prove_s={prove_s:.3} verify_ms={verify_ms:.3} proof_bytes={proof_bytes} onchain_bytes={onchain_bytes}"
    );
}
