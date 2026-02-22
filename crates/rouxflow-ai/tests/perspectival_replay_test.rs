use rouxflow_bitboard::move_indices::Move;
use rouxflow_bitboard::BitCube;
use rouxflow_core::cube::facelet::Color;
use rouxflow_core::cube::Orientation;
use rouxflow_core::telemetry::{DebugTrace, SimpleSolveEvent};

mod test_data;

fn run_debug_replay(solve_name: &str, telemetry: rouxflow_core::telemetry::SolveTelemetry) {
    println!("\n=== DEBUG PERSPECTIVAL REPLAY: {} ===", solve_name);

    let mut debug_trace = DebugTrace::default();
    let parsed = rouxflow_ai::gyro_analyzer::analyze_solve(&telemetry, Some(&mut debug_trace));
    let clean = parsed.to_clean();

    let trace_filename = format!("{}_trace.json", solve_name.replace(" ", "").to_lowercase());
    let trace_json = serde_json::to_string_pretty(&debug_trace).unwrap();
    std::fs::write(&trace_filename, trace_json).unwrap();
    println!("Wrote debug trace to {}", trace_filename);

    let mut cube = BitCube::new_solved();

    println!("1. Applying Scramble (body frame): {}", telemetry.scramble);
    for m_str in telemetry.scramble.split_whitespace() {
        cube.apply_move(m_str);
    }

    println!(
        "2. Initial Solver Perspective: {:?}",
        clean.initial_orientation
    );
    let bitcube_home = Orientation {
        top: Color::White,
        front: Color::Green,
    };
    let initial_rot =
        rouxflow_ai::gyro_analyzer::math::detect_rotation(bitcube_home, clean.initial_orientation);
    println!("   Alignment Rotations: {}", initial_rot);
    for rot_part in initial_rot.split_whitespace() {
        cube.apply_move(rot_part);
    }

    for event in clean.timeline.iter() {
        match event {
            SimpleSolveEvent::Move { m, .. } => {
                cube.apply_move_enum(*m);
            }
            SimpleSolveEvent::Rotation { axis, .. } => {
                cube.apply_move_enum(Move::Rotate(*axis));
            }
        }
    }

    let is_solved = cube.is_solved();
    println!("4. Result: Solved = {}", is_solved);
    if !is_solved {
        println!("   Final Cube State:\n{}", cube.to_html_string());
    }

    assert!(is_solved, "{} failed perspectival replay!", solve_name);
}

#[test]
fn test_solve_1_perspectival_replay() {
    run_debug_replay("SOLVE 1", test_data::solve_1());
}

#[test]
fn test_solve_2_perspectival_replay() {
    run_debug_replay("SOLVE 2", test_data::solve_2());
}
