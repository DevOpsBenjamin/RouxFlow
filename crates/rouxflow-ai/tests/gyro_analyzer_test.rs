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

    // Parse PRINT_OUTPUT from env: whether to print cube states/logging
    // Usage: PRINT_OUTPUT=1 cargo test --test gyro_analyzer_test -- --nocapture
    let print_output: bool = std::env::var("PRINT_OUTPUT")
        .map(|v| v != "0" && v != "false")
        .unwrap_or(true);

    let parsed = rouxflow_ai::analyze_solve(&telemetry, print_output);
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

    let print_output: bool = std::env::var("PRINT_OUTPUT")
        .map(|v| v != "0" && v != "false")
        .unwrap_or(true);

    let parsed = rouxflow_ai::analyze_solve(&telemetry, print_output);
    let json = serde_json::to_string_pretty(&parsed).unwrap();
    std::fs::write("solve2.json", json).unwrap();
}
