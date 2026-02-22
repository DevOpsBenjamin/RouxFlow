use rouxflow_core::telemetry::DebugTrace;
mod test_data;

#[test]
fn test_analyze_solve_1_debug() {
    let telemetry = test_data::solve_1();

    // Verify data loaded correctly
    assert!(
        !telemetry.scramble_gyro.is_empty(),
        "scramble gyro should not be empty"
    );
    assert!(
        !telemetry.solve_gyro.is_empty(),
        "solve gyro should not be empty"
    );
    assert!(
        !telemetry.solve_moves.is_empty(),
        "solve moves should not be empty"
    );

    let mut debug_trace = DebugTrace::default();
    let parsed = rouxflow_ai::analyze_solve(&telemetry, Some(&mut debug_trace));

    let trace_json = serde_json::to_string_pretty(&debug_trace).unwrap();
    std::fs::write("solve1_trace.json", trace_json).unwrap();

    let json = serde_json::to_string_pretty(&parsed).unwrap();
    std::fs::write("solve1.json", json).unwrap();

    let clean = parsed.to_clean();
    let clean_json = serde_json::to_string_pretty(&clean).unwrap();
    std::fs::write("solve1_clean.json", clean_json).unwrap();
}

#[test]
fn test_analyze_solve_2_debug() {
    let telemetry = test_data::solve_2();

    assert!(
        !telemetry.scramble_gyro.is_empty(),
        "scramble gyro should not be empty"
    );
    assert!(
        !telemetry.solve_gyro.is_empty(),
        "solve gyro should not be empty"
    );
    assert!(
        !telemetry.solve_moves.is_empty(),
        "solve moves should not be empty"
    );
    assert!(
        !telemetry.scramble.is_empty(),
        "scramble should not be empty"
    );

    let mut debug_trace = DebugTrace::default();
    let parsed = rouxflow_ai::analyze_solve(&telemetry, Some(&mut debug_trace));

    let trace_json = serde_json::to_string_pretty(&debug_trace).unwrap();
    std::fs::write("solve2_trace.json", trace_json).unwrap();

    let json = serde_json::to_string_pretty(&parsed).unwrap();
    std::fs::write("solve2.json", json).unwrap();

    let clean = parsed.to_clean();
    let clean_json = serde_json::to_string_pretty(&clean).unwrap();
    std::fs::write("solve2_clean.json", clean_json).unwrap();
}
