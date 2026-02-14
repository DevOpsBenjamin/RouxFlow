mod test_data;

#[test]
fn test_analyze_solve_1_debug() {
    let telemetry = test_data::solve_1();

    // Verify data loaded correctly
    assert!(!telemetry.scramble_gyro.is_empty(), "scramble gyro should not be empty");
    assert!(!telemetry.solve_gyro.is_empty(), "solve gyro should not be empty");
    assert!(!telemetry.solve_moves.is_empty(), "solve moves should not be empty");
    assert_eq!(telemetry.scramble, test_data::SOLVE_1_SCRAMBLE);

    // Run analysis — prints debug output, verify it doesn't panic
    rouxflow_ai::analyze_solve(&telemetry);
}
