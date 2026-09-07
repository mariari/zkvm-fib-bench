use jolt_sdk::serialize_and_print_size;
use std::time::Instant;

fn valid_grid<const N: usize, const B: usize>() -> [[u32; N]; N] {
    std::array::from_fn(|row| {
        std::array::from_fn(|column| {
            ((B * (row % B) + row / B + column) % N + 1) as u32
        })
    })
}

macro_rules! run_case {
    ($n:expr, $grid:expr, $compile:ident, $shared:ident, $prover:ident, $verifier:ident, $build_prover:ident, $build_verifier:ident) => {{
        let grid = $grid;
        let mut program = guest::$compile("/tmp/jolt-guest-targets");
        let shared = guest::$shared(&mut program).unwrap();
        let prover_preprocessing = guest::$prover(shared.clone());
        let verifier_setup = prover_preprocessing.generators.to_verifier_setup();
        let verifier_preprocessing = guest::$verifier(shared, verifier_setup, None);
        let prove = guest::$build_prover(program, prover_preprocessing);
        let verify = guest::$build_verifier(verifier_preprocessing);

        let started = Instant::now();
        let (output, proof, io_device) = prove($n, grid);
        let prove_s = started.elapsed().as_secs_f64();
        let cycles = proof.trace_length;
        let path = format!("/tmp/jolt-sudoku-{}.proof", $n);
        serialize_and_print_size("Proof", &path, &proof).unwrap();
        let proof_bytes = std::fs::metadata(&path).unwrap().len();

        let started = Instant::now();
        let valid = verify($n, grid, output, io_device.panic, proof);
        let verify_ms = started.elapsed().as_secs_f64() * 1000.0;
        assert!(valid && output, "Jolt verification failed");
        println!("BENCH jolt bench=sudoku n={} mode=stark prove_s={prove_s:.3} verify_ms={verify_ms:.3} proof_bytes={proof_bytes} cycles={cycles}", $n);
    }};
}

fn main() {
    let n = std::env::args().nth(1).and_then(|v| v.parse().ok()).unwrap_or(9);
    match n {
        4 => run_case!(4, valid_grid::<4, 2>(), compile_sudoku4, preprocess_shared_sudoku4, preprocess_prover_sudoku4, preprocess_verifier_sudoku4, build_prover_sudoku4, build_verifier_sudoku4),
        9 => run_case!(9, valid_grid::<9, 3>(), compile_sudoku9, preprocess_shared_sudoku9, preprocess_prover_sudoku9, preprocess_verifier_sudoku9, build_prover_sudoku9, build_verifier_sudoku9),
        16 => run_case!(16, valid_grid::<16, 4>(), compile_sudoku16, preprocess_shared_sudoku16, preprocess_prover_sudoku16, preprocess_verifier_sudoku16, build_prover_sudoku16, build_verifier_sudoku16),
        _ => panic!("supported Sudoku sizes are 4, 9, and 16"),
    }
}
