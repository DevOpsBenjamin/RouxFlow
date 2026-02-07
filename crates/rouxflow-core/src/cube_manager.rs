use crate::session::{FlowState, SessionManager};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub name: String,
    pub mac_address: String,
    pub protocol_name: String,
    pub has_gyro: bool,
    pub battery_level: Option<u8>,
    pub sw_version: Option<String>,
    pub hw_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CubeState {
    pub orientation: [f32; 4], // quaternion (x, y, z, w)
    pub facelets: Vec<u8>,
    pub battery_level: Option<u8>,
    pub last_move: Option<String>,
}

impl Default for CubeState {
    fn default() -> Self {
        Self {
            orientation: [0.0, 0.0, 0.0, 1.0],
            facelets: vec![0; 54],
            battery_level: None,
            last_move: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerState {
    pub is_running: bool,
    pub time_ms: u64,
    pub moves: Vec<String>,
}

impl Default for TimerState {
    fn default() -> Self {
        Self {
            is_running: false,
            time_ms: 0,
            moves: Vec::new(),
        }
    }
}

/// Unified cube state manager that acts as single source of truth
pub struct CubeManager {
    connection_state: ConnectionState,
    device_info: Option<DeviceInfo>,
    cube_state: CubeState,
    session_manager: SessionManager,
    timer_state: TimerState,
    timer_start_time: Option<f64>,
}

impl CubeManager {
    pub fn new() -> Self {
        Self {
            connection_state: ConnectionState::Disconnected,
            device_info: None,
            cube_state: CubeState::default(),
            session_manager: SessionManager::new(),
            timer_state: TimerState::default(),
            timer_start_time: None,
        }
    }

    // ========== Connection Management ==========

    pub fn connect(&mut self, name: String, mac_address: String, protocol_name: String, has_gyro: bool) {
        self.connection_state = ConnectionState::Connected;
        self.device_info = Some(DeviceInfo {
            name,
            mac_address,
            protocol_name,
            has_gyro,
            battery_level: None,
            sw_version: None,
            hw_version: None,
        });
    }

    pub fn disconnect(&mut self) {
        self.connection_state = ConnectionState::Disconnected;
        self.device_info = None;
        self.cube_state = CubeState::default();
        self.stop_timer(0.0); // Stop timer if running
    }

    pub fn is_connected(&self) -> bool {
        self.connection_state == ConnectionState::Connected
    }

    pub fn get_device_info_json(&self) -> Option<String> {
        self.device_info.as_ref().and_then(|info| serde_json::to_string(info).ok())
    }

    // ========== Cube State Management ==========

    pub fn update_orientation(&mut self, x: f32, y: f32, z: f32, w: f32) {
        self.cube_state.orientation = [x, y, z, w];
    }

    pub fn update_facelets(&mut self, facelets: &[u8]) {
        self.cube_state.facelets = facelets.to_vec();
    }

    pub fn update_battery(&mut self, level: u8) {
        self.cube_state.battery_level = Some(level);
        if let Some(device_info) = &mut self.device_info {
            device_info.battery_level = Some(level);
        }
    }

    pub fn update_hardware(&mut self, sw_version: String, hw_version: String) {
        if let Some(device_info) = &mut self.device_info {
            device_info.sw_version = Some(sw_version);
            device_info.hw_version = Some(hw_version);
        }
    }

    pub fn record_move(&mut self, move_str: String) {
        self.cube_state.last_move = Some(move_str.clone());
        if self.timer_state.is_running {
            self.timer_state.moves.push(move_str);
        }
    }

    pub fn get_cube_state_json(&self) -> String {
        serde_json::to_string(&self.cube_state).unwrap_or_else(|_| "{}".to_string())
    }

    pub fn get_orientation(&self) -> [f32; 4] {
        self.cube_state.orientation
    }

    pub fn get_facelets(&self) -> Vec<u8> {
        self.cube_state.facelets.clone()
    }

    // ========== Timer Management ==========

    pub fn start_timer(&mut self, timestamp: f64) {
        self.timer_state.is_running = true;
        self.timer_state.time_ms = 0;
        self.timer_state.moves.clear();
        self.timer_start_time = Some(timestamp);
    }

    pub fn stop_timer(&mut self, _timestamp: f64) {
        self.timer_state.is_running = false;
        self.timer_start_time = None;
    }

    pub fn update_timer(&mut self, timestamp: f64) {
        if self.timer_state.is_running {
            if let Some(start) = self.timer_start_time {
                self.timer_state.time_ms = ((timestamp - start) * 1000.0) as u64;
            }
        }
    }

    pub fn get_timer_state_json(&self) -> String {
        serde_json::to_string(&self.timer_state).unwrap_or_else(|_| "{}".to_string())
    }

    pub fn is_timer_running(&self) -> bool {
        self.timer_state.is_running
    }

    pub fn get_current_time_ms(&self) -> u64 {
        self.timer_state.time_ms
    }

    // ========== Session Delegation ==========

    pub fn get_session_manager(&self) -> &SessionManager {
        &self.session_manager
    }

    pub fn get_session_manager_mut(&mut self) -> &mut SessionManager {
        &mut self.session_manager
    }

    pub fn get_flow_state(&self) -> String {
        self.session_manager.get_flow_state()
    }

    pub fn set_active_session(&mut self, session_json: &str) {
        self.session_manager.set_active_session(session_json);
    }

    pub fn create_session(&mut self, session_json: &str) {
        self.session_manager.set_active_session(session_json);
    }

    pub fn start_scramble(&mut self, scramble: &str) -> String {
        self.session_manager.start_scramble(scramble)
    }

    pub fn handle_scramble_move(&mut self, move_str: &str, timestamp: f64) -> String {
        self.session_manager.handle_scramble_move(move_str, timestamp)
    }

    pub fn set_solving(&mut self, timestamp: f64) -> String {
        // Start timer when solving begins
        self.start_timer(timestamp);
        self.session_manager.set_solving()
    }

    pub fn record_solve(&mut self, timestamp: f64, time_ms: u32, moves_json: &str) -> String {
        // Stop timer when solve is recorded
        self.stop_timer(timestamp);
        self.session_manager.record_solve(time_ms, moves_json)
    }

    // ========== MAC Address Validation ==========

    /// Check if a protocol requires MAC address for encryption
    pub fn protocol_requires_mac(protocol: &str) -> bool {
        matches!(protocol, "MoYuAi" | "MoYuV3" | "GanV2" | "GanV3" | "GanV4")
    }

    /// Check if device_id is a valid MAC address format (XX:XX:XX:XX:XX:XX)
    pub fn is_valid_mac_format(device_id: &str) -> bool {
        if device_id.len() != 17 {
            return false;
        }

        let parts: Vec<&str> = device_id.split(':').collect();
        if parts.len() != 6 {
            return false;
        }

        parts.iter().all(|part| {
            part.len() == 2 && part.chars().all(|c| c.is_ascii_hexdigit())
        })
    }

    /// Determine if we need to prompt user for MAC address
    /// Returns true if:
    /// 1. Protocol requires MAC for encryption, AND
    /// 2. Device ID is not a valid MAC address
    pub fn needs_mac_input(device_id: &str, protocol: &str) -> bool {
        Self::protocol_requires_mac(protocol) && !Self::is_valid_mac_format(device_id)
    }
}

impl Default for CubeManager {
    fn default() -> Self {
        Self::new()
    }
}
