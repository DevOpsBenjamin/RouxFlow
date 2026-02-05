use serde::{Serialize, Deserialize};
use crate::cube::Quaternion;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
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

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum FlowState {
    Idle,
    Scrambling,
    Ready,
    Solving,
    Finished,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
pub enum CoreAction {
    SaveSolve(Solve),
    DemoteSession(String), // Session ID
    FlowStateChanged(FlowState),
    NotifyReady,
    Pickup,
    Putdown,
    Move(String), // Move notation
    Error(String),
}

pub struct ScrambleValidator {
    pub scramble: Vec<String>,
    pub current_index: usize,
    last_move_time: f64,
    pub is_invalid: bool,
    mistakes: Vec<String>,
}

impl ScrambleValidator {
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

pub struct SessionManager {
    active_session: Option<Session>,
    scramble_validator: Option<ScrambleValidator>,

    // Motion fields
    last_orientation: Option<Quaternion>,
    is_stable: bool,
    stable_since: f64,

    // Flow management
    flow_state: FlowState,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            active_session: None,
            scramble_validator: None,
            last_orientation: None,
            is_stable: true,
            stable_since: 0.0,
            flow_state: FlowState::Idle,
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
        self.save_solve_internal(solve)
    }

    fn save_solve_internal(&mut self, solve: Solve) -> String {
        if let Some(session) = &mut self.active_session {
            // WCA Integrity: 1h limit
            if let SessionType::WCA = session.session_type {
                if let Some(first_at) = session.first_solve_at {
                    let one_hour_ms = 3600 * 1000;
                    if solve.date > first_at + one_hour_ms {
                        return serde_json::to_string(&CoreAction::DemoteSession(session.id.clone())).unwrap();
                    }
                } else {
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

    pub fn create_session(&mut self, name: String, session_type: SessionType) -> String {
        let session = Session {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            session_type,
            solves: Vec::new(),
            first_solve_at: None,
        };
        self.active_session = Some(session.clone());
        serde_json::to_string(&session).unwrap_or_default()
    }

    pub fn start_scramble(&mut self, scramble: &str) -> String {
        self.scramble_validator = Some(ScrambleValidator::new(scramble));
        self.flow_state = FlowState::Scrambling;
        serde_json::to_string(&CoreAction::FlowStateChanged(self.flow_state)).unwrap_or_default()
    }

    pub fn reset_flow(&mut self) -> String {
        self.flow_state = FlowState::Idle;
        self.scramble_validator = None;
        serde_json::to_string(&CoreAction::FlowStateChanged(self.flow_state)).unwrap_or_default()
    }

    pub fn get_active_session_json(&self) -> String {
        serde_json::to_string(&self.active_session).unwrap_or_default()
    }

    pub fn handle_scramble_move(&mut self, move_str: &str, timestamp: f64) -> String {
        if let Some(v) = &mut self.scramble_validator {
            let moved = v.handle_move(move_str, timestamp);

            if v.is_ready() && self.flow_state != FlowState::Ready {
                self.flow_state = FlowState::Ready;
                return serde_json::to_string(&CoreAction::FlowStateChanged(self.flow_state)).unwrap_or_default();
            }

            if moved {
                return serde_json::to_string(&CoreAction::Move(move_str.to_string())).unwrap_or_default();
            }
        }
        "".into()
    }

    pub fn record_solve(&mut self, time_ms: u32, moves_json: &str) -> String {
        let moves: Vec<String> = serde_json::from_str(moves_json).unwrap_or_default();
        let solve = Solve {
            id: uuid::Uuid::new_v4().to_string(),
            time: time_ms,
            moves,
            date: chrono::Utc::now().timestamp_millis(),
            is_valid: true,
        };

        self.flow_state = FlowState::Finished;
        self.save_solve_internal(solve)
    }

    pub fn set_solving(&mut self) -> String {
        self.flow_state = FlowState::Solving;
        serde_json::to_string(&CoreAction::FlowStateChanged(self.flow_state)).unwrap_or_default()
    }

    pub fn get_flow_state(&self) -> String {
        serde_json::to_string(&self.flow_state).unwrap_or_default()
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
