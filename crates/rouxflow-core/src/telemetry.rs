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
pub struct StepDetails {
    pub fb: isize,
    pub sb: isize,
    pub cmll: isize,
    pub eo: isize,
    pub ur_lr: isize,
    pub l4e: isize,
}

impl Default for StepDetails {
    fn default() -> Self {
        Self {
            fb: -1,
            sb: -1,
            cmll: -1,
            eo: -1,
            ur_lr: -1,
            l4e: -1,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ParsedSolve {
    pub solve_duration_ms: f64,
    pub is_solved: bool,
    pub move_count: usize,
    pub tps: f64,
    pub step_details: StepDetails,
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
    },
    /// A cube rotation detected by the gyro.
    /// Does NOT count toward move_count or TPS.
    ///
    /// `is_inspection = true`  → round-trip peek (rotate to look, then rotate back).
    /// `is_inspection = false` → persistent orientation change the solver kept.
    Rotation {
        t: f64,
        axis: Rotation,
        from_orientation: Orientation,
        to_orientation: Orientation,
        /// True when this rotation is part of a within-window round-trip (inspection peek).
        is_inspection: bool,
    },
}

impl SolveEvent {
    pub fn t(&self) -> f64 {
        match self {
            SolveEvent::Move { t, .. } => *t,
            SolveEvent::Rotation { t, .. } => *t,
        }
    }
}

// ========== Clean / User-facing Output Models ==========

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum SimpleSolveEvent {
    Move {
        t: f64,
        #[serde(rename = "move")]
        m: Move,
    },
    Rotation {
        t: f64,
        axis: Rotation,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CleanParsedSolve {
    pub solve_duration_ms: f64,
    pub move_count: usize,
    pub tps: f64,
    pub step_details: StepDetails,
    pub initial_orientation: Orientation,
    pub timeline: Vec<SimpleSolveEvent>,
}

impl ParsedSolve {
    pub fn to_clean(&self) -> CleanParsedSolve {
        let timeline = self
            .timeline
            .iter()
            .map(|event| match event {
                SolveEvent::Move {
                    t, relative_move, ..
                } => SimpleSolveEvent::Move {
                    t: *t,
                    m: *relative_move,
                },
                SolveEvent::Rotation { t, axis, .. } => {
                    SimpleSolveEvent::Rotation { t: *t, axis: *axis }
                }
            })
            .collect();

        CleanParsedSolve {
            solve_duration_ms: self.solve_duration_ms,
            move_count: self.move_count,
            tps: self.tps,
            step_details: self.step_details.clone(),
            initial_orientation: self.initial_orientation,
            timeline,
        }
    }
}
