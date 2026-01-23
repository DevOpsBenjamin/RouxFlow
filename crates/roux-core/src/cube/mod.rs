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
    pub fn is_solved(&self) -> bool {
        if self.stickers.len() < 20 { return false; }
        self.stickers[0] == 0x01 && self.stickers[1] == 0x01
    }

    #[wasm_bindgen(getter)]
    pub fn stickers(&self) -> Vec<u8> {
        self.stickers.clone()
    }
}

pub mod moyu;

pub trait CubeProtocol {
    fn name(&self) -> &str;
    fn has_gyro(&self) -> bool;
    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, String>;
    fn decode_move(&self, decrypted_data: &[u8]) -> Result<CubeMove, String>;
    fn decode_state(&self, decrypted_data: &[u8]) -> Result<CubeState, String>;
    fn decode_orientation(&self, decrypted_data: &[u8]) -> Result<Option<Quaternion>, String>;
}
