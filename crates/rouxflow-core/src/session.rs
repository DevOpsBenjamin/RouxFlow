use serde::{Serialize, Deserialize};
use crate::cube::Quaternion;
use crate::move_interpreter::MoveKind;

pub const DEFAULT_SESSION_ID: &str = "default";
pub const DEFAULT_SESSION_NAME: &str = "Default Session";

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum SessionType {
    Free,
    WCA,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TimedMove {
    pub n: String,
    pub t: u32,
    pub k: MoveKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub g: Option<[f32; 3]>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Solve {
    pub id: String,
    pub time: u32,
    pub moves: Vec<String>,
    pub date: i64,
    pub is_valid: bool,
    #[serde(default)]
    pub scramble: Option<String>,
    #[serde(default)]
    pub timed_moves: Option<Vec<TimedMove>>,
    #[serde(default)]
    pub penalty: Option<String>,  // "DNF" or "+2"
    #[serde(default)]
    pub deleted_at: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Session {
    pub id: String,
    pub name: String,
    pub session_type: SessionType,
    pub solves: Vec<Solve>,
    pub first_solve_at: Option<i64>,
    #[serde(default)]
    pub user_id: Option<String>,
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
    pub mistake_count: usize,
    /// True when we're on the second half of a double move (e.g. first D of D2 was done).
    pub half_done: bool,
    /// Internal move counter — increments on every accepted expanded move (including each half of D2).
    pub accepted_count: usize,
    /// Milliseconds elapsed since the last correct scramble move (or since scramble start).
    pub move_elapsed_ms: u32,
    /// Configured timeout per move in ms. None = no timeout.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub move_timeout_ms: Option<u32>,
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

/// Max uncorrected mistakes in Free mode before invalidation (cube is too far gone).
const FREE_MODE_MAX_MISTAKE_STACK: usize = 4;

pub struct ScrambleValidator {
    /// Original scramble for display: ["R", "U2", "L'"]
    pub scramble: Vec<String>,
    /// Expanded for validation: ["R", "U", "U", "L'"] — double moves split into quarter turns
    expanded: Vec<String>,
    /// 0 = normal, 1 = first half of double (any direction), 2 = second half (must match first)
    pub(crate) double_kind: Vec<u8>,
    /// Maps each expanded index → original scramble index (for display highlighting)
    display_map: Vec<usize>,
    /// Current position in expanded sequence
    pub current_index: usize,
    /// Wall-clock time (seconds) of last correct move (or scramble start)
    last_move_time: f64,
    /// Wall-clock time (seconds) when scramble started
    start_time: f64,
    pub is_invalid: bool,
    mistakes: Vec<String>,
    /// Total mistake count (never decremented, even when mistakes are undone)
    total_mistakes: usize,
    /// WCA mode: strict 3-total-mistake invalidation. Free mode: only invalidate on deep mistake stack.
    wca_mode: bool,
}

impl ScrambleValidator {
    pub fn new(scramble_str: &str, wca_mode: bool) -> Self {
        let scramble: Vec<String> = scramble_str.split_whitespace().map(|s| s.to_string()).collect();
        let mut expanded = Vec::new();
        let mut double_kind = Vec::new();
        let mut display_map = Vec::new();

        for (i, m) in scramble.iter().enumerate() {
            if m.ends_with("2") {
                // "U2" → two quarter turns: first accepts any direction, second must match
                let face = &m[..m.len()-1];
                expanded.push(face.to_string());
                expanded.push(face.to_string());
                double_kind.push(1); // first half: any direction of same face
                double_kind.push(2); // second half: must match first half exactly
                display_map.push(i);
                display_map.push(i);
            } else {
                expanded.push(m.clone());
                double_kind.push(0); // normal: exact match
                display_map.push(i);
            }
        }

        Self {
            scramble,
            expanded,
            double_kind,
            display_map,
            current_index: 0,
            last_move_time: 0.0,
            start_time: 0.0,
            is_invalid: false,
            mistakes: Vec::new(),
            total_mistakes: 0,
            wca_mode,
        }
    }

    /// Set start time (called when scramble begins, so elapsed tracking works from the start).
    pub fn set_start_time(&mut self, timestamp: f64) {
        self.start_time = timestamp;
        self.last_move_time = timestamp;
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

        // Set start time on the very first move
        if self.start_time <= 0.0 {
            self.start_time = timestamp;
            self.last_move_time = timestamp;
        }

        // 1. Check if it's the expected move and we have no pending mistakes
        //    All moves (including both halves of D2) require exact match.
        //    D2 expands to [D, D] — both must be D. Doing D' is a mistake.
        if self.mistakes.is_empty() && self.current_index < self.expanded.len() {
            if move_str == self.expanded[self.current_index] {
                self.current_index += 1;
                self.last_move_time = timestamp;
                return true;
            }
        }

        // 2. Check if it's an undo of a mistake
        if !self.mistakes.is_empty() {
            if move_str == Self::get_inverse(self.mistakes.last().unwrap()) {
                self.mistakes.pop();
                self.last_move_time = timestamp; // fresh 3s for next move
                return true;
            }
        }

        // 3. Check if it's an undo of the previous correct move
        //    BUT: don't allow undoing the first half of a double move when we're on the second half.
        //    e.g. F2 = [F, F]: after first F accepted, F' should be a mistake, not an undo.
        if self.mistakes.is_empty() && self.current_index > 0 {
            let on_second_half = self.current_index < self.double_kind.len()
                && self.double_kind[self.current_index] == 2;
            if !on_second_half {
                if move_str == Self::get_inverse(&self.expanded[self.current_index - 1]) {
                    self.current_index -= 1;
                    self.last_move_time = timestamp;
                    return true;
                }
            }
        }

        // 4. Otherwise, it's a mistake — reset timer so correction gets its own 3s window
        self.mistakes.push(move_str.to_string());
        self.total_mistakes += 1;
        self.last_move_time = timestamp;

        if self.wca_mode {
            // WCA: 3 total mistakes (even corrected) invalidate the scramble.
            // Prevents R R' loops to stall for extra inspection time.
            if self.total_mistakes >= 3 {
                self.is_invalid = true;
            }
        } else {
            // Free: invalidate only when uncorrected mistakes pile up too deep.
            // The cube is too far from expected state to recover.
            if self.mistakes.len() >= FREE_MODE_MAX_MISTAKE_STACK {
                self.is_invalid = true;
            }
        }

        true // Still processing, but might be invalid
    }

    pub fn is_ready(&self) -> bool {
        self.current_index >= self.expanded.len() && !self.is_invalid && self.mistakes.is_empty()
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
        if self.current_index < self.expanded.len() {
            Some(&self.expanded[self.current_index])
        } else {
            None
        }
    }

    /// Map expanded index to original scramble index (for display highlighting)
    pub fn display_index(&self) -> usize {
        if self.current_index >= self.expanded.len() {
            self.scramble.len()
        } else {
            self.display_map[self.current_index]
        }
    }

    /// Total mistakes made (includes undone mistakes).
    pub fn total_mistakes(&self) -> usize {
        self.total_mistakes
    }

    /// Milliseconds elapsed since last correct move (or scramble start).
    pub fn move_elapsed_ms(&self, now: f64) -> u32 {
        if self.last_move_time <= 0.0 {
            return 0;
        }
        ((now - self.last_move_time) * 1000.0).max(0.0) as u32
    }
}

/// Consolidate consecutive identical quarter-turn moves into double notation.
/// ["D", "D", "R'"] → ["D2", "R'"]
pub fn consolidate_moves(moves: &[String]) -> Vec<String> {
    let mut result = Vec::with_capacity(moves.len());
    let mut i = 0;
    while i < moves.len() {
        if !moves[i].ends_with('2') && i + 1 < moves.len() && moves[i] == moves[i + 1] {
            let face = moves[i].trim_end_matches('\'');
            result.push(format!("{}2", face));
            i += 2;
        } else {
            result.push(moves[i].clone());
            i += 1;
        }
    }
    result
}

/// WCA scramble move timeout: 3 seconds per move.
pub const WCA_SCRAMBLE_MOVE_TIMEOUT_MS: u32 = 3_000;

pub struct SessionManager {
    sessions: Vec<Session>,
    active_session: Option<Session>,
    scramble_validator: Option<ScrambleValidator>,

    // Current user (None = guest)
    current_user_id: Option<String>,

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
            current_user_id: None,
            last_orientation: None,
            is_stable: true,
            stable_since: 0.0,
            flow_state: FlowState::Idle,
            inspection_start: None,
            inspection_duration: 15.0,
            pending_scramble: None,
        }
    }

    /// Set the current user ID. Call before loading sessions.
    pub fn set_user_id(&mut self, user_id: Option<String>) {
        self.current_user_id = user_id;
    }

    pub fn get_user_id(&self) -> Option<&str> {
        self.current_user_id.as_deref()
    }

    // ========== Session List Management ==========

    /// Bulk load sessions from storage (replaces in-memory list)
    pub fn load_sessions(&mut self, sessions: Vec<Session>) {
        self.sessions = sessions;
    }

    /// Compute the default session ID for the current user.
    pub fn default_session_id(&self) -> String {
        match &self.current_user_id {
            Some(uid) => format!("default_{}", uid),
            None => DEFAULT_SESSION_ID.to_string(),
        }
    }

    /// Ensure the DefaultSession exists for the current user.
    /// Returns Some(session) if a new one was created (caller should persist it).
    pub fn ensure_default_session(&mut self) -> Option<Session> {
        let default_id = self.default_session_id();
        if self.sessions.iter().any(|s| s.id == default_id) {
            return None;
        }

        let session = Session {
            id: default_id,
            name: DEFAULT_SESSION_NAME.to_string(),
            session_type: SessionType::Free,
            solves: Vec::new(),
            first_solve_at: None,
            user_id: self.current_user_id.clone(),
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

    /// Populate active session's solves from storage.
    /// Sorts by date to ensure chronological order (IndexedDB returns by key/UUID).
    /// Also recomputes `first_solve_at` from the earliest valid solve if not already set.
    pub fn load_solves_into_active(&mut self, mut solves: Vec<Solve>) {
        solves.sort_by_key(|s| s.date);
        if let Some(session) = &mut self.active_session {
            // Recompute first_solve_at from earliest valid (non-deleted) solve
            if session.first_solve_at.is_none() {
                session.first_solve_at = solves.iter()
                    .find(|s| s.is_valid && s.deleted_at.is_none())
                    .map(|s| s.date);
            }
            session.solves = solves.clone();
            // Also sync into sessions list
            if let Some(s) = self.sessions.iter_mut().find(|s| s.id == session.id) {
                s.first_solve_at = session.first_solve_at;
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
            user_id: self.current_user_id.clone(),
        };
        self.sessions.push(session.clone());
        self.active_session = Some(session.clone());
        serde_json::to_string(&session).unwrap_or_default()
    }

    pub fn start_scramble(&mut self, scramble: &str) -> String {
        let wca = self.active_session.as_ref()
            .is_some_and(|s| matches!(s.session_type, SessionType::WCA));
        self.scramble_validator = Some(ScrambleValidator::new(scramble, wca));
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

    pub fn record_solve(&mut self, time_ms: u32, moves_json: &str, timed_moves_json: &str) -> String {
        let moves: Vec<String> = serde_json::from_str(moves_json).unwrap_or_default();
        let timed_moves: Option<Vec<TimedMove>> = serde_json::from_str(timed_moves_json).ok()
            .filter(|v: &Vec<TimedMove>| !v.is_empty());
        let scramble = self.scramble_validator.as_ref()
            .map(|v| v.scramble.join(" "));
        let solve = Solve {
            id: uuid::Uuid::new_v4().to_string(),
            time: time_ms,
            moves,
            date: chrono::Utc::now().timestamp_millis(),
            is_valid: true,
            scramble,
            timed_moves,
            penalty: None,
            deleted_at: None,
        };

        self.flow_state = FlowState::Summary;
        self.save_solve_internal(solve)
    }

    /// Record a DNF solve (inspection timeout — no moves made).
    pub fn record_dnf(&mut self) -> String {
        let scramble = self.scramble_validator.as_ref()
            .map(|v| v.scramble.join(" "));
        let solve = Solve {
            id: uuid::Uuid::new_v4().to_string(),
            time: 0,
            moves: Vec::new(),
            date: chrono::Utc::now().timestamp_millis(),
            is_valid: true,
            scramble,
            timed_moves: None,
            penalty: Some("DNF".to_string()),
            deleted_at: None,
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

    /// Check if the current scramble move has timed out (WCA only).
    /// Only applies after the first move — the first move has no timeout
    /// (user can pause between scrambles, like waiting for a scrambler in WCA).
    /// Returns true if the scramble was just invalidated.
    pub fn check_scramble_timeout(&mut self, now: f64) -> bool {
        let is_wca = self.active_session.as_ref()
            .is_some_and(|s| matches!(s.session_type, SessionType::WCA));
        if !is_wca {
            return false;
        }
        if let Some(v) = &mut self.scramble_validator {
            if v.is_invalid || v.is_ready() || v.current_index == 0 {
                return false;
            }
            if v.last_move_time > 0.0 {
                let elapsed_ms = ((now - v.last_move_time) * 1000.0).max(0.0) as u32;
                if elapsed_ms >= WCA_SCRAMBLE_MOVE_TIMEOUT_MS {
                    v.is_invalid = true;
                    return true;
                }
            }
        }
        false
    }

    pub fn get_scramble_index(&self) -> usize {
        self.scramble_validator.as_ref().map(|v| v.display_index()).unwrap_or(0)
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

    /// Set inspection duration in seconds. WCA = 15s (enforced), Free = configurable.
    pub fn set_inspection_duration(&mut self, seconds: f64) {
        self.inspection_duration = seconds;
    }

    pub fn get_inspection_duration(&self) -> f64 {
        self.inspection_duration
    }

    // ========== Scramble State ==========

    pub fn get_scramble_state_json(&self, now: f64) -> String {
        let is_wca = self.active_session.as_ref()
            .is_some_and(|s| matches!(s.session_type, SessionType::WCA));
        let move_timeout = if is_wca { Some(WCA_SCRAMBLE_MOVE_TIMEOUT_MS) } else { None };

        let state = match &self.scramble_validator {
            Some(v) => {
                let half = v.current_index < v.double_kind.len()
                    && v.double_kind[v.current_index] == 2;
                ScrambleState {
                    scramble: v.scramble.clone(),
                    index: v.display_index(),
                    total: v.scramble.len(),
                    is_ready: v.is_ready(),
                    is_invalid: v.is_invalid,
                    expected_move: v.get_expected_move().map(|s| s.to_string()),
                    correction_move: v.get_correction_move(),
                    mistake_count: v.total_mistakes(),
                    half_done: half,
                    accepted_count: v.current_index,
                    move_elapsed_ms: v.move_elapsed_ms(now),
                    move_timeout_ms: move_timeout,
                }
            },
            None => ScrambleState {
                scramble: Vec::new(),
                index: 0,
                total: 0,
                is_ready: false,
                is_invalid: false,
                expected_move: None,
                correction_move: None,
                mistake_count: 0,
                half_done: false,
                accepted_count: 0,
                move_elapsed_ms: 0,
                move_timeout_ms: None,
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

    // ========== Stats Queries ==========

    pub fn get_session_stats_json(&self) -> String {
        match &self.active_session {
            Some(s) => {
                let now_ms = chrono::Utc::now().timestamp_millis();
                let stats = crate::stats::compute_session_stats(&s.solves, s.session_type, s.first_solve_at, now_ms);
                serde_json::to_string(&stats).unwrap_or_else(|_| "{}".to_string())
            }
            None => "{}".to_string(),
        }
    }

    pub fn get_solve_list_json(&self) -> String {
        match &self.active_session {
            Some(s) => {
                let list = crate::stats::compute_solve_list(&s.solves);
                serde_json::to_string(&list).unwrap_or_else(|_| "[]".to_string())
            }
            None => "[]".to_string(),
        }
    }

    pub fn get_solve_by_id_json(&self, solve_id: &str) -> String {
        match &self.active_session {
            Some(s) => {
                match s.solves.iter().find(|solve| solve.id == solve_id) {
                    Some(solve) => {
                        let mut display = solve.clone();
                        display.moves = consolidate_moves(&display.moves);
                        serde_json::to_string(&display).unwrap_or_else(|_| "null".to_string())
                    }
                    None => "null".to_string(),
                }
            }
            None => "null".to_string(),
        }
    }

    // ========== Soft-Delete ==========

    pub fn delete_solve(&mut self, solve_id: &str) -> String {
        let now = chrono::Utc::now().timestamp_millis();

        // Update in active session
        if let Some(session) = &mut self.active_session {
            if let Some(solve) = session.solves.iter_mut().find(|s| s.id == solve_id) {
                solve.deleted_at = Some(now);
                let result = serde_json::to_string(&CoreAction::SaveSolve(solve.clone())).unwrap();

                // Sync to sessions list
                let session_id = session.id.clone();
                if let Some(s) = self.sessions.iter_mut().find(|s| s.id == session_id) {
                    if let Some(solve) = s.solves.iter_mut().find(|s| s.id == solve_id) {
                        solve.deleted_at = Some(now);
                    }
                }

                return result;
            }
        }

        serde_json::to_string(&CoreAction::Error("Solve not found".into())).unwrap()
    }

    // ========== WCA Session Queries ==========

    /// Returns true if the active session is WCA and has 5 valid (non-deleted) solves.
    pub fn is_wca_full(&self) -> bool {
        self.active_session.as_ref().is_some_and(|s| {
            matches!(s.session_type, SessionType::WCA)
                && s.solves.iter().filter(|v| v.is_valid && v.deleted_at.is_none()).count() >= 5
        })
    }

    // ========== Flow State Enum Access ==========

    pub fn get_flow_state_enum(&self) -> FlowState {
        self.flow_state
    }
}
