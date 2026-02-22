use crate::bluetooth_manager::BluetoothManager;
use crate::cube::{CubeMove, CubeState, Face, Quaternion};
use crate::cube::facelet::Color as FaceletColor;
use crate::gyro_calibrator::GyroCalibrator;
use crate::move_interpreter::{InterpretedMove, InterpreterConfig, MoveInterpreter, MoveKind};
use crate::scramble::generate_scramble;
use crate::session::{CoreAction, FlowState, SessionManager};
use crate::telemetry::{GyroSample, RawMove, SolveTelemetry};
use crate::timer_manager::TimerManager;

/// Result of `flow_coordinate` — actions to emit and whether calibration was started.
pub struct FlowCoordinateResult {
    pub actions: Vec<String>,
    pub calibration_started: bool,
}

/// Result of `flush_and_dispatch` — actions, renderer animations, and gyro offset.
pub struct DispatchResult {
    pub actions: Vec<String>,
    /// (remapped_notation, kind) pairs for WASM to pass to the renderer.
    pub animations: Vec<(String, MoveKind)>,
    /// If calibration was finalized, the render offset quaternion (x, y, z, w).
    pub render_offset: Option<(f32, f32, f32, f32)>,
    /// Whether a new calibration was started during flow coordination.
    pub calibration_started: bool,
    /// Info about interpreted moves for WASM debug logging.
    pub interpreted_info: Vec<InterpretedMoveInfo>,
}

/// Minimal info about each interpreted move for WASM debug logging.
pub struct InterpretedMoveInfo {
    pub notation: String,
    pub remapped: String,
    pub kind: MoveKind,
}

/// Unified application state composed of sub-managers.
/// Each sub-manager owns a single concern; cross-domain
/// coordination lives in the `impl AppState` methods below.
pub struct AppState {
    pub bluetooth: BluetoothManager,
    pub session: SessionManager,
    pub timer: TimerManager,
    pub interpreter: MoveInterpreter,
    pub calibrator: GyroCalibrator,
    /// Logical cube state — tracks moves and produces facelets for rendering
    pub cube_logic: CubeState,
    /// Telemetry: raw solve data for future analyzer
    pub telemetry: Option<SolveTelemetry>,
    /// Telemetry recording active?
    pub telemetry_recording: bool,
    /// Telemetry phase: 0=idle, 1=scramble, 2=solve
    pub telemetry_phase: u8,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            bluetooth: BluetoothManager::new(),
            session: SessionManager::new(),
            timer: TimerManager::new(),
            interpreter: MoveInterpreter::new(InterpreterConfig::default()),
            calibrator: GyroCalibrator::new(),
            cube_logic: CubeState::new(),
            telemetry: None,
            telemetry_recording: false,
            telemetry_phase: 0,
        }
    }

    // ========== Cross-domain coordination ==========

    /// Begin solving: starts timer and transitions session flow state.
    pub fn start_solving(&mut self, timestamp: f64) -> String {
        let action = self.session.set_solving();
        self.timer.start(timestamp);
        action
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

    /// Record a DNF (inspection timeout): stops timer and saves DNF solve.
    pub fn record_dnf(&mut self, timestamp: f64) -> String {
        self.timer.stop(timestamp);
        self.session.record_dnf()
    }

    /// After a move, check flow state and react accordingly.
    /// Returns actions and whether a new calibration was started (for WASM debug hooks).
    pub fn flow_coordinate(
        &mut self,
        timestamp: f64,
        rng: &mut impl FnMut() -> f64,
    ) -> FlowCoordinateResult {
        let mut actions = Vec::new();
        let mut calibration_started = false;

        let flow = self.session.get_flow_state_enum();
        match flow {
            FlowState::Idle => {
                if self.cube_logic.is_solved() && !self.session.is_wca_full() {
                    let scramble = generate_scramble(rng);
                    let action = self.session.start_scramble(&scramble);
                    if !action.is_empty() {
                        self.calibrator.start();
                        calibration_started = true;
                        self.telemetry = Some(SolveTelemetry {
                            scramble: scramble.clone(),
                            scramble_gyro: Vec::new(),
                            solve_gyro: Vec::new(),
                            solve_moves: Vec::new(),
                            scramble_start_t: timestamp,
                            solve_start_t: 0.0,
                            solve_end_t: 0.0,
                        });
                        self.telemetry_recording = true;
                        self.telemetry_phase = 1;
                        actions.push(action);
                    }
                }
            }
            FlowState::Scrambling => {
                if self.session.is_scramble_invalid() {
                    self.telemetry = None;
                    self.telemetry_recording = false;
                    self.telemetry_phase = 0;
                    let action = self.session.reset_flow();
                    if !action.is_empty() { actions.push(action); }
                }
            }
            FlowState::Inspection => {
                if let Some(ref mut tel) = self.telemetry {
                    tel.solve_start_t = timestamp;
                }
                self.telemetry_phase = 2;
                let action = self.start_solving(timestamp);
                if !action.is_empty() { actions.push(action); }
            }
            FlowState::Solving => {
                if self.cube_logic.is_solved() {
                    let time_ms = self.timer.get_current_time_ms() as u32;
                    let moves_json = self.timer.get_moves_json();

                    if let Some(ref mut tel) = self.telemetry {
                        tel.solve_end_t = timestamp;
                    }
                    self.telemetry_recording = false;
                    let action = self.record_solve(timestamp, time_ms, &moves_json);
                    if !action.is_empty() { actions.push(action); }
                    if !self.session.is_wca_full() {
                        let next = generate_scramble(rng);
                        let action2 = self.session.start_scramble(&next);
                        if !action2.is_empty() {
                            self.calibrator.start();
                            calibration_started = true;
                            actions.push(action2);
                        }
                    } else {
                        let action2 = self.session.reset_flow();
                        if !action2.is_empty() { actions.push(action2); }
                    }
                }
            }
            FlowState::Summary => {
                if !self.session.is_wca_full() {
                    let next = generate_scramble(rng);
                    let action = self.session.start_scramble(&next);
                    if !action.is_empty() {
                        self.calibrator.start();
                        calibration_started = true;
                        actions.push(action);
                    }
                } else {
                    let action = self.session.reset_flow();
                    if !action.is_empty() { actions.push(action); }
                }
            }
        }

        FlowCoordinateResult { actions, calibration_started }
    }

    // ========== BLE event processing ==========

    /// Feed a raw BLE face-move event into the interpreter and record telemetry.
    pub fn feed_ble_move(&mut self, face: Face, direction: i8, timestamp: f64, wall_ms: f64) {
        // Record raw BLE face move for telemetry (solve phase only)
        if self.telemetry_recording && self.telemetry_phase == 2 {
            if let Some(ref mut tel) = self.telemetry {
                let notation = CubeMove { face, amount: direction }.notation();
                tel.solve_moves.push(RawMove {
                    n: notation,
                    t: timestamp,
                    k: MoveKind::Face,
                });
            }
        }
        self.interpreter.feed_face_move(face, direction, wall_ms);
    }

    /// Feed a raw BLE gyro event: update orientation, record telemetry,
    /// feed interpreter and calibrator.
    /// Returns whether calibration was started.
    pub fn feed_ble_gyro(
        &mut self,
        qx: f32, qy: f32, qz: f32, qw: f32,
        timestamp: f64,
        wall_ms: f64,
    ) -> bool {
        let q = Quaternion { x: qx, y: qy, z: qz, w: qw };
        self.cube_logic.orientation = Some(q);

        // Record raw gyro for telemetry
        if self.telemetry_recording {
            if let Some(ref mut tel) = self.telemetry {
                let sample = GyroSample { t: timestamp, x: qx, y: qy, z: qz, w: qw };
                match self.telemetry_phase {
                    1 => tel.scramble_gyro.push(sample),
                    2 => tel.solve_gyro.push(sample),
                    _ => {}
                }
            }
        }

        // Feed gyro to interpreter for rotation detection
        self.interpreter.feed_gyro(&q, wall_ms);

        // Feed gyro to calibrator if active
        if self.calibrator.is_active() {
            self.calibrator.feed(&q);
        }

        false
    }

    /// Feed a raw facelet string (54 chars) into cube_logic.
    pub fn handle_raw_facelets(&mut self, facelet_string: &str) {
        let colors: Vec<FaceletColor> = facelet_string.chars().map(|c| match c {
            'U' => FaceletColor::White,
            'R' => FaceletColor::Red,
            'F' => FaceletColor::Green,
            'D' => FaceletColor::Yellow,
            'L' => FaceletColor::Orange,
            'B' => FaceletColor::Blue,
            _ => FaceletColor::White,
        }).collect();
        if colors.len() == 54 {
            self.cube_logic.logic.facelets = colors;
        }
    }

    /// Flush the interpreter and dispatch interpreted moves: remap notation,
    /// apply to cube_logic, record in timer, validate scramble, finalize
    /// calibration, emit CoreAction::Move, and run flow coordination.
    pub fn flush_and_dispatch(
        &mut self,
        timestamp: f64,
        wall_ms: f64,
        rng: &mut impl FnMut() -> f64,
    ) -> DispatchResult {
        let mut actions = Vec::new();
        let mut animations = Vec::new();
        let mut render_offset = None;
        let mut calibration_started = false;
        let mut interpreted_info = Vec::new();

        // Set zone rotation hint, then flush
        let has_zone_rotation = self.calibrator.has_pending_zone_rotation();
        self.interpreter.set_zone_rotation_hint(has_zone_rotation);
        let solve_start_ms = self.timer.start_time_ms();
        let interpreted = self.interpreter.flush(wall_ms, solve_start_ms);

        for imove in &interpreted {
            let remapped = self.calibrator.remap_notation(&imove.notation);

            // Record info for WASM debug logging
            interpreted_info.push(InterpretedMoveInfo {
                notation: imove.notation.clone(),
                remapped: remapped.clone(),
                kind: imove.kind,
            });

            // Apply raw face moves to cube_logic (body frame)
            for &(face, dir) in &imove.raw_face_moves {
                let notation = CubeMove { face, amount: dir }.notation();
                self.cube_logic.apply_move(&notation);
            }

            // Record interpreted move in timer with remapped notation
            let mut remapped_move = imove.clone();
            remapped_move.notation = remapped.clone();
            self.record_interpreted_move(&remapped_move);

            // Queue animation for Face/Slice moves
            match imove.kind {
                MoveKind::Face | MoveKind::Slice => {
                    animations.push((remapped.clone(), imove.kind));
                }
                _ => {}
            }

            // Slice compensation
            if imove.kind == MoveKind::Slice {
                self.calibrator.compensate_slice(&imove.notation);
            }

            // Scramble validation: feed raw face moves (body frame)
            let flow_before = self.session.get_flow_state_enum();
            for &(face, dir) in &imove.raw_face_moves {
                let notation = CubeMove { face, amount: dir }.notation();
                let a = self.session.handle_scramble_move(&notation, timestamp);
                if !a.is_empty() { actions.push(a); }
            }

            // Check for Scrambling → Inspection transition: finalize gyro calibration
            let flow_after = self.session.get_flow_state_enum();
            if flow_before == FlowState::Scrambling && flow_after == FlowState::Inspection {
                self.telemetry_phase = 2;
                if let Some(_home) = self.calibrator.finalize() {
                    render_offset = self.calibrator.compute_render_offset();
                }
            }

            // Emit CoreAction::Move with remapped notation
            let action_json = serde_json::to_string(&CoreAction::Move(remapped.clone()))
                .unwrap_or_default();
            if !action_json.is_empty() { actions.push(action_json); }

            // Flow coordination: only for face/slice moves (not rotations)
            if imove.kind != MoveKind::Rotation && flow_before == flow_after {
                let result = self.flow_coordinate(timestamp, rng);
                if result.calibration_started { calibration_started = true; }
                actions.extend(result.actions);
            }
        }

        DispatchResult { actions, animations, render_offset, calibration_started, interpreted_info }
    }

    // ========== Timer orchestration ==========

    /// Tick timer + check scramble timeout + inspection timeout.
    /// Returns actions if flow state changed.
    pub fn update_timer(
        &mut self,
        timestamp: f64,
        rng: &mut impl FnMut() -> f64,
    ) -> Vec<String> {
        self.timer.update(timestamp);
        let mut actions = Vec::new();

        // Check scramble move timeout
        if self.session.get_flow_state_enum() == FlowState::Scrambling {
            self.session.check_scramble_timeout(timestamp);
        }

        // Check inspection timeout → DNF
        if self.session.get_flow_state_enum() == FlowState::Inspection {
            if self.session.is_inspection_expired(timestamp) {
                self.telemetry = None;
                self.telemetry_recording = false;
                self.telemetry_phase = 0;
                let action = self.record_dnf(timestamp);
                if !action.is_empty() { actions.push(action); }
                if !self.session.is_wca_full() {
                    let next = generate_scramble(rng);
                    let action2 = self.session.start_scramble(&next);
                    if !action2.is_empty() {
                        self.calibrator.start();
                        actions.push(action2);
                    }
                } else {
                    let action2 = self.session.reset_flow();
                    if !action2.is_empty() { actions.push(action2); }
                }
            }
        }

        actions
    }

    // ========== Reset / scramble generation ==========

    /// Reset flow state. Optionally auto-generates a new scramble if cube is solved.
    pub fn reset_flow(
        &mut self,
        cube_solved: bool,
        rng: &mut impl FnMut() -> f64,
    ) -> (Vec<String>, bool) {
        let mut actions = Vec::new();
        let mut calibration_started = false;

        self.telemetry = None;
        self.telemetry_recording = false;
        self.telemetry_phase = 0;

        let action = self.session.reset_flow();
        if !action.is_empty() { actions.push(action); }

        if cube_solved && !self.session.is_wca_full() {
            let scramble = generate_scramble(rng);
            let action2 = self.session.start_scramble(&scramble);
            if !action2.is_empty() {
                self.calibrator.start();
                calibration_started = true;
                self.telemetry = Some(SolveTelemetry {
                    scramble: scramble.clone(),
                    scramble_gyro: Vec::new(),
                    solve_gyro: Vec::new(),
                    solve_moves: Vec::new(),
                    scramble_start_t: 0.0,
                    solve_start_t: 0.0,
                    solve_end_t: 0.0,
                });
                self.telemetry_recording = true;
                self.telemetry_phase = 1;
                actions.push(action2);
            }
        }

        (actions, calibration_started)
    }

    /// Start a user-provided scramble: calibrate, record telemetry, begin scramble phase.
    pub fn start_scramble_with(&mut self, scramble: &str, timestamp: f64) -> String {
        self.calibrator.start();
        let action = self.session.start_scramble(scramble);
        self.telemetry = Some(SolveTelemetry {
            scramble: scramble.to_string(),
            scramble_gyro: Vec::new(),
            solve_gyro: Vec::new(),
            solve_moves: Vec::new(),
            scramble_start_t: timestamp,
            solve_start_t: 0.0,
            solve_end_t: 0.0,
        });
        self.telemetry_recording = true;
        self.telemetry_phase = 1;
        action
    }

    /// Generate a new scramble and start the scramble phase.
    pub fn generate_new_scramble(
        &mut self,
        rng: &mut impl FnMut() -> f64,
    ) -> (Vec<String>, bool) {
        let mut actions = Vec::new();
        let mut calibration_started = false;

        let scramble = generate_scramble(rng);
        let action = self.session.start_scramble(&scramble);
        if !action.is_empty() {
            self.calibrator.start();
            calibration_started = true;
            self.telemetry = Some(SolveTelemetry {
                scramble: scramble.clone(),
                scramble_gyro: Vec::new(),
                solve_gyro: Vec::new(),
                solve_moves: Vec::new(),
                scramble_start_t: 0.0,
                solve_start_t: 0.0,
                solve_end_t: 0.0,
            });
            self.telemetry_recording = true;
            self.telemetry_phase = 1;
            actions.push(action);
        }

        (actions, calibration_started)
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
