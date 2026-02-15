use rouxflow_core::telemetry::SolveTelemetry;

pub fn solve_1() -> SolveTelemetry {
    let json = include_str!("solve_1.json");
    serde_json::from_str(json).expect("Failed to parse solve_2.json")
}

pub fn solve_2() -> SolveTelemetry {
    let json = include_str!("solve_2.json");
    serde_json::from_str(json).expect("Failed to parse solve_2.json")
}
