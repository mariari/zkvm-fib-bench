// RISC Zero fib benchmark host.
//
// Mirrors the zkbenchmarks.com risc0 harness (proves `fib(n) mod 7919` with
// ProverOpts::succinct()), but ADDS what the site omits: it times prove and
// verify separately and reports proof size + guest cycle stats.
//
// Usage: host <n> [succinct|composite|groth16][+fastdbl]
//        (defaults: n=10000, succinct). The `+fastdbl` suffix switches the guest
//        from the linear recurrence to fast doubling (same journal, ~log n work).
use methods::{METHOD_ELF, METHOD_ID};
use risc0_zkvm::{default_prover, ExecutorEnv, ProverOpts};
use std::time::Instant;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    let args: Vec<String> = std::env::args().collect();
    let n: u32 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(10_000);
    let mode = args
        .get(2)
        .map(|s| s.to_lowercase())
        .unwrap_or_else(|| "succinct".to_string());

    // `<prover>+fastdbl` selects the fast-doubling guest algorithm.
    let algo: u32 = mode.ends_with("+fastdbl") as u32;
    let prover_mode = mode.trim_end_matches("+fastdbl");

    let opts = match prover_mode {
        "composite" => ProverOpts::composite(),
        "succinct" => ProverOpts::succinct(),
        "groth16" => ProverOpts::groth16(),
        other => {
            eprintln!("unknown mode '{other}', falling back to succinct");
            ProverOpts::succinct()
        }
    };

    let env = ExecutorEnv::builder()
        .write(&n)
        .unwrap()
        .write(&algo)
        .unwrap()
        .build()
        .unwrap();
    let prover = default_prover();

    // ---- prove ----
    let t = Instant::now();
    let prove_info = prover.prove_with_opts(env, METHOD_ELF, &opts).unwrap();
    let prove_s = t.elapsed().as_secs_f64();

    let receipt = prove_info.receipt;
    let proof_bytes = bincode::serialize(&receipt).map(|v| v.len()).unwrap_or(0);

    // ---- verify (the part zkbenchmarks.com does not measure) ----
    let t = Instant::now();
    receipt.verify(METHOD_ID).unwrap();
    let verify_ms = t.elapsed().as_secs_f64() * 1000.0;

    // sanity-check outputs (guest commits n, a, b)
    let (n_out, a, b): (u32, u32, u32) = receipt.journal.decode().unwrap_or((0, 0, 0));

    println!(
        "BENCH risc0 mode={mode} n={n} prove_s={prove_s:.3} verify_ms={verify_ms:.3} proof_bytes={proof_bytes} out=(n={n_out},a={a},b={b}) stats={:?}",
        prove_info.stats
    );
}
