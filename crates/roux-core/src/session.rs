use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use crate::cube::Quaternion;

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum SessionType {
    Free,
    WCA,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Solve {
    pub id: String,
    pub time: u32,
    pub moves: Vec<String>,
    pub date: i64,
    pub is_valid: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Session {
    pub id: String,
    pub name: String,
    pub session_type: SessionType,
    pub solves: Vec<Solve>,
    pub first_solve_at: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
pub enum CoreAction {
    SaveSolve(Solve),
    DemoteSession(String), // Session ID
    NotifyReady,
    Pickup,
    Putdown,
    Move(String), // Move notation
    Error(String),
}

#[wasm_bindgen]
pub struct ScrambleValidator {
    scramble: Vec<String>,
    current_index: usize,
    last_move_time: f64,
    is_invalid: bool,
    mistakes: Vec<String>,
}

#[wasm_bindgen]
impl ScrambleValidator {
    #[wasm_bindgen(constructor)]
    pub fn new(scramble_str: &str) -> Self {
        Self {
            scramble: scramble_str.split_whitespace().map(|s| s.to_string()).collect(),
            current_index: 0,
            last_move_time: 0.0,
            is_invalid: false,
            mistakes: Vec::new(),
        }
    }

    fn get_inverse(m: &str) -> String {
        if m.ends_with("'") {
            m[..m.len()-1].to_string()
        } else if m.ends_with("2") {
            m.to_string()
        } else {
            format!("{}'", m)
        }
    }

    pub fn handle_move(&mut self, move_str: &str, timestamp: f64) -> bool {
        if self.is_invalid {
            return false;
        }

        // 1. Check if it's the expected move and we have no pending mistakes
        if self.mistakes.is_empty() && self.current_index < self.scramble.len() && move_str == self.scramble[self.current_index] {
            // Check speed - only if not the first move
            if self.last_move_time > 0.0 && timestamp - self.last_move_time > 5.0 {
                // User was too slow during scramble (relaxed to 5s for now)
                self.is_invalid = true;
                return false;
            }
            self.current_index += 1;
            self.last_move_time = timestamp;
            return true;
        }

        // 2. Check if it's an undo of a mistake
        if !self.mistakes.is_empty() {
            if move_str == Self::get_inverse(&self.mistakes.last().unwrap()) {
                self.mistakes.pop();
                return true;
            }
        }

        // 3. Check if it's an undo of the previous correct move
        if self.mistakes.is_empty() && self.current_index > 0 {
            if move_str == Self::get_inverse(&self.scramble[self.current_index - 1]) {
                self.current_index -= 1;
                return true;
            }
        }

        // 4. Otherwise, it's a mistake
        self.mistakes.push(move_str.to_string());
        
        // If too many mistakes (tolerance = 1), mark invalid
        if self.mistakes.len() > 1 {
            self.is_invalid = true;
        }

        true // Still processing, but might be invalid
    }

    pub fn is_ready(&self) -> bool {
        self.current_index >= self.scramble.len() && !self.is_invalid && self.mistakes.is_empty()
    }
}

#[wasm_bindgen]
pub struct SessionManager {
    active_session: Option<Session>,
    scramble_validator: Option<ScrambleValidator>,
    
    // Motion fields
    last_orientation: Option<Quaternion>,
    is_stable: bool,
    stable_since: f64,
}

#[wasm_bindgen]
impl SessionManager {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            active_session: None,
            scramble_validator: None,
            last_orientation: None,
            is_stable: true,
            stable_since: 0.0,
        }
    }

    pub fn set_active_session(&mut self, session_json: &str) {
        if let Ok(session) = serde_json::from_str(session_json) {
            self.active_session = Some(session);
        }
    }

    pub fn add_solve(&mut self, solve_json: &str) -> String {
        let solve: Solve = match serde_json::from_str(solve_json) {
            Ok(s) => s,
            Err(e) => return serde_json::to_string(&CoreAction::Error(e.to_string())).unwrap(),
        };

        if let Some(session) = &mut self.active_session {
            // WCA Integrity: 1h limit
            if let SessionType::WCA = session.session_type {
                if let Some(first_at) = session.first_solve_at {
                    let one_hour_ms = 3600 * 1000;
                    if solve.date > first_at + one_hour_ms {
                        // Demote session
                        return serde_json::to_string(&CoreAction::DemoteSession(session.id.clone())).unwrap();
                    }
                } else {
                    // This is the first solve
                    session.first_solve_at = Some(solve.date);
                }

                if session.solves.len() >= 5 {
                    return serde_json::to_string(&CoreAction::Error("WCA Session full".into())).unwrap();
                }
            }

            serde_json::to_string(&CoreAction::SaveSolve(solve)).unwrap()
        } else {
            serde_json::to_string(&CoreAction::Error("No active session".into())).unwrap()
        }
    }

    pub fn process_orientation(&mut self, x: f32, y: f32, z: f32, w: f32, timestamp: f64) -> String {
        let q = Quaternion { x, y, z, w };
        if let Some(last) = self.last_orientation {
            let delta = (q.x - last.x).powi(2) + (q.y - last.y).powi(2) + (q.z - last.z).powi(2) + (q.w - last.w).powi(2);
            let threshold = 0.0001; 
            let moving = delta > threshold;

            if moving && self.is_stable {
                self.is_stable = false;
                return serde_json::to_string(&CoreAction::Pickup).unwrap_or_default();
            } else if !moving && !self.is_stable {
                if self.stable_since == 0.0 {
                    self.stable_since = timestamp;
                } else if timestamp - self.stable_since > 500.0 { 
                    self.is_stable = true;
                    self.stable_since = 0.0;
                    return serde_json::to_string(&CoreAction::Putdown).unwrap_or_default();
                }
            } else if moving {
                self.stable_since = 0.0;
            }
        }
        self.last_orientation = Some(q);
        "".into()
    }

    pub fn create_session(&mut self, id: String, name: String, session_type: SessionType) -> String {
        let session = Session {
            id: id.clone(),
            name,
            session_type,
            solves: Vec::new(),
            first_solve_at: None,
        };
        self.active_session = Some(session.clone());
        serde_json::to_string(&session).unwrap_or_default()
    }

    pub fn start_scramble(&mut self, scramble: &str) {
        self.scramble_validator = Some(ScrambleValidator::new(scramble));
    }

    pub fn get_active_session_json(&self) -> String {
        serde_json::to_string(&self.active_session).unwrap_or_default()
    }

    pub fn handle_scramble_move(&mut self, move_str: &str, timestamp: f64) -> bool {
        if let Some(v) = &mut self.scramble_validator {
            return v.handle_move(move_str, timestamp);
        }
        false
    }

    pub fn is_scramble_ready(&self) -> bool {
        self.scramble_validator.as_ref().map(|v| v.is_ready()).unwrap_or(false)
    }

    pub fn is_scramble_invalid(&self) -> bool {
        self.scramble_validator.as_ref().map(|v| v.is_invalid).unwrap_or(false)
    }

    pub fn get_scramble_index(&self) -> usize {
        self.scramble_validator.as_ref().map(|v| v.current_index).unwrap_or(0)
    }

    pub fn get_scramble_len(&self) -> usize {
        self.scramble_validator.as_ref().map(|v| v.scramble.len()).unwrap_or(0)
    }
}
