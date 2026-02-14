use crate::bluetooth_manager::BluetoothManager;
use crate::gyro_calibrator::GyroCalibrator;
use crate::move_interpreter::{InterpreterConfig, InterpretedMove, MoveInterpreter};
use crate::session::{PickupMode, SessionManager, FIXED_PICKUP_MS, FIXED_PUTDOWN_MS};
use crate::timer_manager::TimerManager;

/// Unified application state composed of sub-managers.
/// Each sub-manager owns a single concern; cross-domain
/// coordination lives in the `impl AppState` methods below.
pub struct AppState {
    pub bluetooth: BluetoothManager,
    pub session: SessionManager,
    pub timer: TimerManager,
    pub interpreter: MoveInterpreter,
    pub calibrator: GyroCalibrator,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            bluetooth: BluetoothManager::new(),
            session: SessionManager::new(),
            timer: TimerManager::new(),
            interpreter: MoveInterpreter::new(InterpreterConfig::default()),
            calibrator: GyroCalibrator::new(),
        }
    }

    // ========== Cross-domain coordination ==========

    /// Begin solving: starts timer and transitions session flow state.
    /// For WCA+gyro: timer starts at frozen pickup time (not first move).
    /// set_solving() freezes the pickup time first, then we read it.
    pub fn start_solving(&mut self, timestamp: f64) -> String {
        let action = self.session.set_solving();
        let timer_start = if self.session.requires_putdown_confirm() {
            self.session.solve_pickup_time().unwrap_or(timestamp)
        } else {
            timestamp
        };
        self.timer.start(timer_start);
        action
    }

    /// Record a completed solve: stops timer and saves solve via session.
    /// Fixed pickup mode: adds FIXED_PICKUP_MS + FIXED_PUTDOWN_MS to account for both.
    pub fn record_solve(&mut self, timestamp: f64, time_ms: u32, moves_json: &str) -> String {
        let timed_moves_json = self.timer.get_timed_moves_json();
        self.timer.stop(timestamp);
        let adjusted_time = if matches!(self.session.effective_pickup_mode(), PickupMode::Fixed) {
            time_ms + FIXED_PICKUP_MS + FIXED_PUTDOWN_MS
        } else {
            time_ms
        };
        self.session.record_solve(adjusted_time, moves_json, &timed_moves_json)
    }

    /// Record a move: tracks in timer (if running) and returns notation.
    pub fn record_move(&mut self, move_str: String) {
        self.timer.record_move(move_str);
    }

    /// Record an interpreted move in the timer.
    pub fn record_interpreted_move(&mut self, m: &InterpretedMove) {
        self.timer.record_interpreted_move(m);
    }

    /// Hold a solve pending putdown confirmation (WCA+gyro).
    /// Stops timer, stores moves/scramble, but does not save yet.
    /// Solve time will be computed at putdown as (putdown_moment - last_pickup).
    pub fn hold_solve(&mut self, timestamp: f64, moves_json: &str) {
        let timed_moves_json = self.timer.get_timed_moves_json();
        self.timer.stop(timestamp);
        self.session.hold_pending_solve(moves_json, &timed_moves_json, timestamp);
    }

    /// Record a DNF (inspection timeout): stops timer and saves DNF solve.
    pub fn record_dnf(&mut self, timestamp: f64) -> String {
        self.timer.stop(timestamp);
        self.session.record_dnf()
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
