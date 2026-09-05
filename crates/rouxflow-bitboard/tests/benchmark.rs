use rouxflow_bitboard::{BitCube, FaceMove};
use std::time::Instant;

#[test]
fn benchmark_moves() {
    let niters = 10_000_000;
    let mut cube = BitCube::new_solved();

    println!(
        "\n--- RouxFlow Bitboard Benchmark ({} iterations) ---",
        niters
    );

    // 1. Face Moves (R)
    let start = Instant::now();
    for _ in 0..niters {
        cube.face_r();
    }
    let duration = start.elapsed();
    println!(
        "Face Move (R):   {:>8.2} million moves/sec",
        niters as f64 / duration.as_secs_f64() / 1_000_000.0
    );

    // 2. Slice Moves (M)
    let start = Instant::now();
    for _ in 0..niters {
        cube.slice_m();
    }
    let duration = start.elapsed();
    println!(
        "Slice Move (M):   {:>8.2} million moves/sec",
        niters as f64 / duration.as_secs_f64() / 1_000_000.0
    );

    // 3. Wide Moves (Rw)
    let start = Instant::now();
    for _ in 0..niters {
        cube.wide_rw();
    }
    let duration = start.elapsed();
    println!(
        "Wide Move (Rw):   {:>8.2} million moves/sec",
        niters as f64 / duration.as_secs_f64() / 1_000_000.0
    );

    // 4. Rotations (Y) - already single pass
    let start = Instant::now();
    for _ in 0..niters {
        cube.rot_y();
    }
    let duration = start.elapsed();
    println!(
        "Rotation (Y):    {:>8.2} million moves/sec",
        niters as f64 / duration.as_secs_f64() / 1_000_000.0
    );

    // 5. Rotations (X) - newly optimized
    let start = Instant::now();
    for _ in 0..niters {
        cube.rot_x();
    }
    let duration = start.elapsed();
    println!(
        "Rotation (X):    {:>8.2} million moves/sec",
        niters as f64 / duration.as_secs_f64() / 1_000_000.0
    );

    // 6. Random Scramble (Mix of everything)
    // We use a small array of moves to cycle through
    let moves = [
        FaceMove::R,
        FaceMove::U,
        FaceMove::F,
        FaceMove::L,
        FaceMove::B,
        FaceMove::D,
        FaceMove::Rp,
        FaceMove::Up,
    ];
    let start = Instant::now();
    for i in 0..niters {
        cube.apply_face_move(moves[i % 8]);
    }
    let duration = start.elapsed();
    println!(
        "Mixed Scramble:  {:>8.2} million moves/sec",
        niters as f64 / duration.as_secs_f64() / 1_000_000.0
    );

    println!("--------------------------------------------------\n");
}
