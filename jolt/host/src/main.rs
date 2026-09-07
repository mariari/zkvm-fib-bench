use jolt_sdk::serialize_and_print_size;
use std::time::Instant;

fn main() {
    let n = std::env::args()
        .nth(1)
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(10_000);

    let mut program = guest::compile_fib("/tmp/jolt-guest-targets");
    let shared = guest::preprocess_shared_fib(&mut program).unwrap();
    let prover_preprocessing = guest::preprocess_prover_fib(shared.clone());
    let verifier_setup = prover_preprocessing.generators.to_verifier_setup();
    let verifier_preprocessing =
        guest::preprocess_verifier_fib(shared, verifier_setup, None);

    let prove_fib = guest::build_prover_fib(program, prover_preprocessing);
    let verify_fib = guest::build_verifier_fib(verifier_preprocessing);

    let started = Instant::now();
    let (output, proof, io_device) = prove_fib(n);
    let prove_s = started.elapsed().as_secs_f64();
    let cycles = proof.trace_length;

    let proof_path = format!("/tmp/jolt-fib-{n}.proof");
    serialize_and_print_size("Proof", &proof_path, &proof).unwrap();
    let proof_bytes = std::fs::metadata(&proof_path).unwrap().len();

    let started = Instant::now();
    let valid = verify_fib(n, output, io_device.panic, proof);
    let verify_ms = started.elapsed().as_secs_f64() * 1000.0;
    assert!(valid, "Jolt verification failed");

    println!(
        "BENCH jolt mode=stark n={n} prove_s={prove_s:.3} verify_ms={verify_ms:.3} proof_bytes={proof_bytes} cycles={cycles}"
    );
}
