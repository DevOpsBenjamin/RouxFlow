use serde::{Serialize, Deserialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Face {
    U = 0, R = 1, F = 2, D = 3, L = 4, B = 5
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CubeMove {
    pub face: Face,
    pub amount: i8, // 1, -1, 2
}

impl CubeMove {
    pub fn notation(&self) -> String {
        let face_names = ["U", "R", "F", "D", "L", "B"];
        let amount_str = if self.amount == 1 { "" } else if self.amount == -1 { "'" } else { "2" };
        format!("{}{}", face_names[self.face as usize], amount_str)
    }

    pub fn inverse_notation(&self) -> String {
        let face_names = ["U", "R", "F", "D", "L", "B"];
        let amount_str = if self.amount == 1 { "'" } else if self.amount == -1 { "" } else { "2" };
        format!("{}{}", face_names[self.face as usize], amount_str)
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MotionState {
    Stable = 0,
    Moving = 1,
}

pub mod facelet;

#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CubeState {
    pub(crate) stickers: Vec<u8>,
    pub orientation: Option<Quaternion>,
    pub motion: MotionState,
    #[serde(skip)]
    pub(crate) logic: facelet::FaceletCube,
}

#[wasm_bindgen]
impl CubeState {
    /// Create a solved cube state
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        CubeState {
            stickers: vec![0x01; 20], 
            orientation: Some(Quaternion { x: 0.0, y: 0.0, z: 0.0, w: 1.0 }),
            motion: MotionState::Stable,
            logic: facelet::FaceletCube::new(),
        }
    }

    /// Apply a move (like "U", "R'", "F2") using the internal logic engine
    pub fn apply_move(&mut self, move_str: &str) {
        self.logic.apply_move(move_str);
    }

    pub fn dump_debug(&self) {
        self.logic.dump_debug();
    }


    /// Get all 54 facelet colors as a flat array of bytes (0-5)
    pub fn get_facelets(&self) -> Vec<u8> {
        self.logic.facelets.iter().map(|&c| c as u8).collect()
    }

    pub fn is_solved(&self) -> bool {
        // Simple check for facelet model: compare with new cube
        let solved = facelet::FaceletCube::new();
        self.logic.facelets == solved.facelets
    }

    #[wasm_bindgen(getter)]
    pub fn stickers(&self) -> Vec<u8> {
        self.stickers.clone()
    }
}

pub mod gan_v2;

pub trait CubeProtocol {
    fn name(&self) -> &str;
    fn has_gyro(&self) -> bool;
    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, String>;
    fn decode_move(&self, decrypted_data: &[u8]) -> Result<CubeMove, String>;
    fn decode_state(&self, decrypted_data: &[u8]) -> Result<CubeState, String>;
    fn decode_orientation(&self, decrypted_data: &[u8]) -> Result<Option<Quaternion>, String>;
}
