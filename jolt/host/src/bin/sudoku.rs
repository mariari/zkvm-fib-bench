use jolt_sdk::serialize_and_print_size;
use std::time::Instant;

fn valid_grid() -> [[u32; 9]; 9] {
    std::array::from_fn(|row| {
        std::array::from_fn(|column| {
            ((3 * (row % 3) + row / 3 + column) % 9 + 1) as u32
        })
    })
}

fn main() {
    let n = std::env::args()
        .nth(1)
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(9);
    assert_eq!(n, 9, "the Jolt Sudoku benchmark is fixed at 9x9");
    let grid = valid_grid();

    let mut program = guest::compile_sudoku("/tmp/jolt-guest-targets");
    let shared = guest::preprocess_shared_sudoku(&mut program).unwrap();
    let prover_preprocessing = guest::preprocess_prover_sudoku(shared.clone());
    let verifier_setup = prover_preprocessing.generators.to_verifier_setup();
    let verifier_preprocessing =
        guest::preprocess_verifier_sudoku(shared, verifier_setup, None);

    let prove_sudoku = guest::build_prover_sudoku(program, prover_preprocessing);
    let verify_sudoku = guest::build_verifier_sudoku(verifier_preprocessing);

    let started = Instant::now();
    let (output, proof, io_device) = prove_sudoku(n, grid.clone());
    let prove_s = started.elapsed().as_secs_f64();
    let cycles = proof.trace_length;

    let proof_path = format!("/tmp/jolt-sudoku-{n}.proof");
    serialize_and_print_size("Proof", &proof_path, &proof).unwrap();
    let proof_bytes = std::fs::metadata(&proof_path).unwrap().len();

    let started = Instant::now();
    let valid = verify_sudoku(n, grid, output, io_device.panic, proof);
    let verify_ms = started.elapsed().as_secs_f64() * 1000.0;
    assert!(valid && output, "Jolt verification failed");

    println!(
        "BENCH jolt bench=sudoku n={n} mode=stark prove_s={prove_s:.3} verify_ms={verify_ms:.3} proof_bytes={proof_bytes} cycles={cycles}"
    );
}
