// Jolt fib benchmark host.
//
// Runs the same program risc0/ and sp1/ run (`fib(n) mod 7919`, the exact
// zkbenchmarks.com program) and reports what the other two hosts report:
// prove and verify timed separately, plus proof size.
//
// Usage: cargo run --release -- <n> [stark][+fastdbl|+bounds]
//        (defaults: n=10000, stark). Jolt has a single proving mode -- there
//        is no analogue of risc0's succinct/composite or SP1's core/
//        compressed -- so `stark` is the only prover mode, and the suffixes
//        select the guest algorithm exactly as they do for the other two.
//        The `+bounds` suffix runs the bounds check instead, in which case the
//        first argument is not a loop count but the value x (default 42).
use jolt_sdk::serialize_and_print_size;
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = args
        .get(2)
        .map(|s| s.to_lowercase())
        .unwrap_or_else(|| "stark".to_string());

    // `<prover>+fastdbl` and `<prover>+bounds` select the guest algorithm.
    let algo: u32 = if mode.ends_with("+bounds") {
        2
    } else if mode.ends_with("+fastdbl") {
        1
    } else {
        0
    };

    // The bounds check has no loop count, so it reuses the first argument as x.
    let n: u32 = args
        .get(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(if algo == 2 { 42 } else { 10_000 });

    // ---- compile the guest ELF (outside every timed section) ----
    let mut program = guest::compile_fib("/tmp/jolt-guest-targets");

    // ---- setup (preprocessing, the analogue of SP1's proving/verifying keys) ----
    let t = Instant::now();
    let shared = guest::preprocess_shared_fib(&mut program).unwrap();
    let prover_preprocessing = guest::preprocess_prover_fib(shared.clone());
    let verifier_setup = prover_preprocessing.generators.to_verifier_setup();
    let verifier_preprocessing = guest::preprocess_verifier_fib(shared, verifier_setup, None);
    let setup_s = t.elapsed().as_secs_f64();

    let prove_fib = guest::build_prover_fib(program, prover_preprocessing);
    let verify_fib = guest::build_verifier_fib(verifier_preprocessing);

    // ---- prove ----
    let t = Instant::now();
    let (output, proof, io_device) = prove_fib(n, algo);
    let prove_s = t.elapsed().as_secs_f64();

    // ---- proof size ----
    let tmp = format!("/tmp/jolt_fib_{mode}.bin");
    let _ = serialize_and_print_size("Proof", &tmp, &proof);
    let proof_bytes = std::fs::metadata(&tmp).map(|m| m.len()).unwrap_or(0);

    // ---- verify (the part zkbenchmarks.com does not measure) ----
    let t = Instant::now();
    let ok = verify_fib(n, algo, output, io_device.panic, proof);
    let verify_ms = t.elapsed().as_secs_f64() * 1000.0;
    assert!(ok, "verification failed");

    // cycles=0: risc0 and SP1 hand their host a cycle count as a by-product of
    // execution; Jolt's is reachable only through `guest::analyze_fib`, whose
    // summary API this host does not read yet. Left explicit rather than
    // silently omitted -- bench_all.sh will carry the 0 through to the CELL line.
    println!(
        "BENCH jolt mode={mode} n={n} cycles=0 setup_s={setup_s:.3} prove_s={prove_s:.3} verify_ms={verify_ms:.3} proof_bytes={proof_bytes} onchain_bytes=0"
    );
}
