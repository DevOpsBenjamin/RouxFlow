use rouxflow_core::telemetry::SolveTelemetry;

pub const SOLVE_1_SCRAMBLE: &str = "F' D2 U L' D2 U2 F2 D F R B2 R' B' L' R' D2 F R' D' U2";

pub fn solve_1() -> SolveTelemetry {
    let json = include_str!("solve_1.json");
    let mut telemetry: SolveTelemetry =
        serde_json::from_str(json).expect("Failed to parse solve_1.json");
    telemetry.scramble = SOLVE_1_SCRAMBLE.to_string();
    telemetry
}

pub fn solve_2() -> SolveTelemetry {
    let json = include_str!("solve_2.json");
    serde_json::from_str(json).expect("Failed to parse solve_2.json")
}
