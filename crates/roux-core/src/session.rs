use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::cube::{CubeMove, Quaternion};

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
    pub date: u64,
    pub is_valid: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Session {
    pub id: String,
    pub name: String,
    pub session_type: SessionType,
    pub solves: Vec<Solve>,
}

#[wasm_bindgen]
pub struct ScrambleValidator {
    scramble: Vec<String>,
    current_index: usize,
    last_move_time: f64,
    is_invalid: bool,
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
        }
    }

    pub fn handle_move(&mut self, move_str: &str, timestamp: f64) -> bool {
        if self.is_invalid || self.current_index >= self.scramble.len() {
            return false;
        }

        if move_str == self.scramble[self.current_index] {
            if self.last_move_time > 0.0 && timestamp - self.last_move_time > 2.0 {
                self.is_invalid = true;
                return false;
            }
            self.current_index += 1;
            self.last_move_time = timestamp;
            return true;
        }
        false
    }

    pub fn is_ready(&self) -> bool {
        self.current_index >= self.scramble.len() && !self.is_invalid
    }
}

#[wasm_bindgen]
pub struct SessionManager {
    sessions: HashMap<String, Session>,
    active_session_id: Option<String>,
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
            sessions: HashMap::new(),
            active_session_id: None,
            scramble_validator: None,
            last_orientation: None,
            is_stable: true,
            stable_since: 0.0,
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
                return "{\"type\":\"pickup\"}".into();
            } else if !moving && !self.is_stable {
                if self.stable_since == 0.0 {
                    self.stable_since = timestamp;
                } else if timestamp - self.stable_since > 500.0 { 
                    self.is_stable = true;
                    self.stable_since = 0.0;
                    return "{\"type\":\"putdown\"}".into();
                }
            } else if moving {
                self.stable_since = 0.0;
            }
        }
        self.last_orientation = Some(q);
        "".into()
    }

    pub fn create_session(&mut self, id: String, name: String, session_type: SessionType) {
        let session = Session {
            id: id.clone(),
            name,
            session_type,
            solves: Vec::new(),
        };
        self.sessions.insert(id.clone(), session);
        self.active_session_id = Some(id);
    }

    pub fn add_solve(&mut self, solve_json: &str) -> Result<(), String> {
        let solve: Solve = serde_json::from_str(solve_json).map_err(|e| e.to_string())?;
        if let Some(active_id) = &self.active_session_id {
            if let Some(session) = self.sessions.get_mut(active_id) {
                if let SessionType::WCA = session.session_type {
                    if session.solves.len() >= 5 {
                        return Err("WCA Session limited to 5 solves".into());
                    }
                }
                session.solves.push(solve);
                return Ok(());
            }
        }
        Err("No active session".into())
    }

    pub fn start_scramble(&mut self, scramble: &str) {
        self.scramble_validator = Some(ScrambleValidator::new(scramble));
    }

    pub fn switch_session(&mut self, id: String) {
        if self.sessions.contains_key(&id) {
            self.active_session_id = Some(id);
        }
    }

    pub fn get_sessions_json(&self) -> String {
        let list: Vec<&Session> = self.sessions.values().collect();
        serde_json::to_string(&list).unwrap_or_default()
    }

    pub fn scramble_validator(&mut self) -> Option<ScrambleValidator> {
        // This is tricky in wasm_bindgen (ownership). 
        // Better to expose methods ON SessionManager that delegate to validator.
        None 
    }

    pub fn get_active_session_json(&self) -> String {
        if let Some(id) = &self.active_session_id {
            if let Some(session) = self.sessions.get(id) {
                return serde_json::to_string(session).unwrap_or_default();
            }
        }
        "".into()
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

    pub fn get_scramble_index(&self) -> usize {
        self.scramble_validator.as_ref().map(|v| v.current_index).unwrap_or(0)
    }

    pub fn get_scramble_len(&self) -> usize {
        self.scramble_validator.as_ref().map(|v| v.scramble.len()).unwrap_or(0)
    }
}
