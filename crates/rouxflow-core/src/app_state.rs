use crate::bluetooth_manager::BluetoothManager;
use crate::move_interpreter::{InterpreterConfig, InterpretedMove, MoveInterpreter};
use crate::session::SessionManager;
use crate::timer_manager::TimerManager;

/// Unified application state composed of sub-managers.
/// Each sub-manager owns a single concern; cross-domain
/// coordination lives in the `impl AppState` methods below.
pub struct AppState {
    pub bluetooth: BluetoothManager,
    pub session: SessionManager,
    pub timer: TimerManager,
    pub interpreter: MoveInterpreter,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            bluetooth: BluetoothManager::new(),
            session: SessionManager::new(),
            timer: TimerManager::new(),
            interpreter: MoveInterpreter::new(InterpreterConfig::default()),
        }
    }

    // ========== Cross-domain coordination ==========

    /// Begin solving: starts timer and transitions session flow state.
    pub fn start_solving(&mut self, timestamp: f64) -> String {
        self.timer.start(timestamp);
        self.session.set_solving()
    }

    /// Record a completed solve: stops timer and saves solve via session.
    pub fn record_solve(&mut self, timestamp: f64, time_ms: u32, moves_json: &str) -> String {
        let timed_moves_json = self.timer.get_timed_moves_json();
        self.timer.stop(timestamp);
        self.session.record_solve(time_ms, moves_json, &timed_moves_json)
    }

    /// Record a move: tracks in timer (if running) and returns notation.
    pub fn record_move(&mut self, move_str: String) {
        self.timer.record_move(move_str);
    }

    /// Record an interpreted move in the timer.
    pub fn record_interpreted_move(&mut self, m: &InterpretedMove) {
        self.timer.record_interpreted_move(m);
    }

    /// Disconnect: clears bluetooth state, stops timer, resets interpreter.
    pub fn disconnect(&mut self) {
        self.bluetooth.disconnect();
        self.timer.stop(0.0);
        self.interpreter.reset();
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
