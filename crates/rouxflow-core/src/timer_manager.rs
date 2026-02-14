use serde::{Deserialize, Serialize};
use crate::move_interpreter::InterpretedMove;
use crate::session::TimedMove;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerState {
    pub is_running: bool,
    pub time_ms: u64,
    pub moves: Vec<String>,
    #[serde(skip)]
    pub timed_moves: Vec<TimedMove>,
}

impl Default for TimerState {
    fn default() -> Self {
        Self {
            is_running: false,
            time_ms: 0,
            moves: Vec::new(),
            timed_moves: Vec::new(),
        }
    }
}

pub struct TimerManager {
    state: TimerState,
    start_time: Option<f64>,
}

impl TimerManager {
    pub fn new() -> Self {
        Self {
            state: TimerState::default(),
            start_time: None,
        }
    }

    pub fn start(&mut self, timestamp: f64) {
        self.state.is_running = true;
        self.state.time_ms = 0;
        self.state.moves.clear();
        self.state.timed_moves.clear();
        self.start_time = Some(timestamp);
    }

    pub fn stop(&mut self, _timestamp: f64) {
        self.state.is_running = false;
        self.start_time = None;
    }

    pub fn update(&mut self, timestamp: f64) {
        if self.state.is_running {
            if let Some(start) = self.start_time {
                self.state.time_ms = ((timestamp - start) * 1000.0) as u64;
            }
        }
    }

    pub fn record_move(&mut self, move_str: String) {
        if self.state.is_running {
            self.state.moves.push(move_str);
        }
    }

    pub fn record_interpreted_move(&mut self, m: &InterpretedMove) {
        if self.state.is_running {
            self.state.moves.push(m.notation.clone());
            self.state.timed_moves.push(TimedMove {
                n: m.notation.clone(),
                t: m.timestamp_ms,
                k: m.kind,
                g: m.gyro_delta,
            });
        }
    }

    pub fn get_state_json(&self) -> String {
        serde_json::to_string(&self.state).unwrap_or_else(|_| "{}".to_string())
    }

    pub fn is_running(&self) -> bool {
        self.state.is_running
    }

    pub fn get_current_time_ms(&self) -> u64 {
        self.state.time_ms
    }

    pub fn get_moves_json(&self) -> String {
        serde_json::to_string(&self.state.moves).unwrap_or_else(|_| "[]".to_string())
    }

    pub fn get_timed_moves_json(&self) -> String {
        serde_json::to_string(&self.state.timed_moves).unwrap_or_else(|_| "[]".to_string())
    }

    /// Start time in milliseconds (wall-clock ms). Returns 0.0 if not running.
    pub fn start_time_ms(&self) -> f64 {
        self.start_time.map(|t| t * 1000.0).unwrap_or(0.0)
    }
}

impl Default for TimerManager {
    fn default() -> Self {
        Self::new()
    }
}
