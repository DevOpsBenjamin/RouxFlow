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

    // Parse IDX_PRINT from env: start printing cubes from this move index
    // Usage: IDX_PRINT=20 cargo test --test gyro_analyzer_test -- --nocapture
    let idx_print: usize = std::env::var("IDX_PRINT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    let parsed = rouxflow_ai::analyze_solve(&telemetry, idx_print);
    let json = serde_json::to_string_pretty(&parsed).unwrap();
    std::fs::write("solve1.json", json).unwrap();
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

    let idx_print: usize = std::env::var("IDX_PRINT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    let parsed = rouxflow_ai::analyze_solve(&telemetry, idx_print);
    let json = serde_json::to_string_pretty(&parsed).unwrap();
    std::fs::write("solve2.json", json).unwrap();
}
