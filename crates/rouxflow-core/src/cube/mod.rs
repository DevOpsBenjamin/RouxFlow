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

#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CubeState {
    pub(crate) stickers: Vec<u8>,
    pub orientation: Option<Quaternion>,
    pub motion: MotionState,
}

#[wasm_bindgen]
impl CubeState {
    /// Create a solved cube state for testing
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        CubeState {
            stickers: vec![0x01; 20], // Placeholder solved state
            orientation: Some(Quaternion { x: 0.0, y: 0.0, z: 0.0, w: 1.0 }),
            motion: MotionState::Stable,
        }
    }

    pub fn is_solved(&self) -> bool {
        if self.stickers.len() < 20 { return false; }
        self.stickers[0] == 0x01 && self.stickers[1] == 0x01
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
