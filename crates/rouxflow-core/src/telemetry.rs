use serde::{Serialize, Deserialize};
use crate::move_interpreter::MoveKind;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GyroSample {
    pub t: f64,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RawMove {
    pub n: String,
    pub t: f64,
    pub k: MoveKind,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SolveTelemetry {
    #[serde(default)]
    pub scramble: String,
    pub scramble_gyro: Vec<GyroSample>,
    pub solve_gyro: Vec<GyroSample>,
    pub solve_moves: Vec<RawMove>,
    pub scramble_start_t: f64,
    pub solve_start_t: f64,
    pub solve_end_t: f64,
}
