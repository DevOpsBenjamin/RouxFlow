use serde::{Serialize, Deserialize};
use crate::cube::Quaternion;

pub const DEFAULT_SESSION_ID: &str = "default";
pub const DEFAULT_SESSION_NAME: &str = "Default Session";

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
    Inspection,
    Solving,
    Summary,
}

#[derive(Serialize)]
pub struct ScrambleState {
    pub scramble: Vec<String>,
    pub index: usize,
    pub total: usize,
    pub is_ready: bool,
    pub is_invalid: bool,
    pub expected_move: Option<String>,
    pub correction_move: Option<String>,
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

    /// Get the correction move needed to undo last mistake (inverse of last mistake)
    pub fn get_correction_move(&self) -> Option<String> {
        self.mistakes.last().map(|m| Self::get_inverse(m))
    }

    /// Get the next expected scramble move, or None if a mistake is pending
    pub fn get_expected_move(&self) -> Option<&str> {
        if !self.mistakes.is_empty() || self.is_invalid {
            return None;
        }
        if self.current_index < self.scramble.len() {
            Some(&self.scramble[self.current_index])
        } else {
            None
        }
    }
}

pub struct SessionManager {
    sessions: Vec<Session>,
    active_session: Option<Session>,
    scramble_validator: Option<ScrambleValidator>,

    // Motion fields
    last_orientation: Option<Quaternion>,
    is_stable: bool,
    stable_since: f64,

    // Flow management
    flow_state: FlowState,

    // Inspection
    inspection_start: Option<f64>,
    inspection_duration: f64,

    // Chaining: next scramble for Summary → Scrambling transition
    pending_scramble: Option<String>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Vec::new(),
            active_session: None,
            scramble_validator: None,
            last_orientation: None,
            is_stable: true,
            stable_since: 0.0,
            flow_state: FlowState::Idle,
            inspection_start: None,
            inspection_duration: 15.0,
            pending_scramble: None,
        }
    }

    // ========== Session List Management ==========

    /// Bulk load sessions from storage (replaces in-memory list)
    pub fn load_sessions(&mut self, sessions: Vec<Session>) {
        self.sessions = sessions;
    }

    /// Ensure the DefaultSession exists. Returns Some(session) if a new one was
    /// created (caller should persist it), or None if it already exists.
    pub fn ensure_default_session(&mut self) -> Option<Session> {
        if self.sessions.iter().any(|s| s.id == DEFAULT_SESSION_ID) {
            return None;
        }

        let session = Session {
            id: DEFAULT_SESSION_ID.to_string(),
            name: DEFAULT_SESSION_NAME.to_string(),
            session_type: SessionType::Free,
            solves: Vec::new(),
            first_solve_at: None,
        };
        self.sessions.push(session.clone());
        Some(session)
    }

    /// Set active session by ID (looks up in sessions list)
    pub fn set_active_session_by_id(&mut self, id: &str) -> bool {
        if let Some(session) = self.sessions.iter().find(|s| s.id == id) {
            self.active_session = Some(session.clone());
            true
        } else {
            false
        }
    }

    /// Populate active session's solves from storage
    pub fn load_solves_into_active(&mut self, solves: Vec<Solve>) {
        if let Some(session) = &mut self.active_session {
            session.solves = solves.clone();
            // Also sync into sessions list
            if let Some(s) = self.sessions.iter_mut().find(|s| s.id == session.id) {
                s.solves = solves;
            }
        }
    }

    /// Get all sessions as JSON
    pub fn get_sessions_json(&self) -> String {
        serde_json::to_string(&self.sessions).unwrap_or_else(|_| "[]".to_string())
    }

    /// Get active session's solves as JSON
    pub fn get_active_session_solves_json(&self) -> String {
        match &self.active_session {
            Some(s) => serde_json::to_string(&s.solves).unwrap_or_else(|_| "[]".to_string()),
            None => "[]".to_string(),
        }
    }

    /// Get active session ID
    pub fn get_active_session_id(&self) -> Option<&str> {
        self.active_session.as_ref().map(|s| s.id.as_str())
    }

    // ========== Legacy Compatibility ==========

    pub fn set_active_session(&mut self, session_json: &str) {
        if let Ok(session) = serde_json::from_str::<Session>(session_json) {
            // Also add to sessions list if not present
            if !self.sessions.iter().any(|s| s.id == session.id) {
                self.sessions.push(session.clone());
            }
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

            // Push solve into active session's in-memory solves
            session.solves.push(solve.clone());

            // Sync to sessions list
            let session_id = session.id.clone();
            if let Some(s) = self.sessions.iter_mut().find(|s| s.id == session_id) {
                s.solves.push(solve.clone());
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
        self.sessions.push(session.clone());
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

            if v.is_ready() && self.flow_state == FlowState::Scrambling {
                return self.enter_inspection(timestamp);
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

        self.flow_state = FlowState::Summary;
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

    // ========== Inspection ==========

    pub fn enter_inspection(&mut self, timestamp: f64) -> String {
        self.flow_state = FlowState::Inspection;
        self.inspection_start = Some(timestamp);
        serde_json::to_string(&CoreAction::FlowStateChanged(self.flow_state)).unwrap_or_default()
    }

    pub fn get_inspection_remaining(&self, now: f64) -> f64 {
        match self.inspection_start {
            Some(start) => {
                let elapsed = now - start;
                let remaining = self.inspection_duration - elapsed;
                if remaining > 0.0 { remaining } else { 0.0 }
            }
            None => 0.0,
        }
    }

    pub fn is_inspection_expired(&self, now: f64) -> bool {
        match self.inspection_start {
            Some(start) => (now - start) >= self.inspection_duration,
            None => false,
        }
    }

    // ========== Scramble State ==========

    pub fn get_scramble_state_json(&self) -> String {
        let state = match &self.scramble_validator {
            Some(v) => ScrambleState {
                scramble: v.scramble.clone(),
                index: v.current_index,
                total: v.scramble.len(),
                is_ready: v.is_ready(),
                is_invalid: v.is_invalid,
                expected_move: v.get_expected_move().map(|s| s.to_string()),
                correction_move: v.get_correction_move(),
            },
            None => ScrambleState {
                scramble: Vec::new(),
                index: 0,
                total: 0,
                is_ready: false,
                is_invalid: false,
                expected_move: None,
                correction_move: None,
            },
        };
        serde_json::to_string(&state).unwrap_or_else(|_| "{}".to_string())
    }

    // ========== Pending Scramble (chaining) ==========

    pub fn set_pending_scramble(&mut self, scramble: String) {
        self.pending_scramble = Some(scramble);
    }

    pub fn get_pending_scramble(&self) -> Option<&str> {
        self.pending_scramble.as_deref()
    }

    // ========== Flow State Enum Access ==========

    pub fn get_flow_state_enum(&self) -> FlowState {
        self.flow_state
    }
}
