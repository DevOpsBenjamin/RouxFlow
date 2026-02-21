use crate::move_interpreter::MoveKind;
use serde::{Deserialize, Serialize};

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

use crate::cube::Orientation;
use rouxflow_bitboard::move_indices::{Move, Rotation};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ParsedSolve {
    pub solve_duration_ms: f64,
    pub is_solved: bool,
    pub steps_reached: Vec<String>, // e.g., ["FB", "SB", "CMLL", "LSE"]
    pub initial_orientation: Orientation,
    pub timeline: Vec<SolveEvent>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum SolveEvent {
    Move {
        t: f64,
        original: Vec<String>,
        body_move: Move,
        relative_move: Move,
        state_after: CubeStateFlags,
    },
    Rotation {
        t: f64,
        axis: Rotation,
        from_orientation: Orientation,
        to_orientation: Orientation,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct CubeStateFlags {
    pub is_fb: bool,
    pub is_sb: bool,
    pub is_cmll: bool,
    pub is_lse_ul_ur: bool,
    pub bad_edges_count: usize,
}
