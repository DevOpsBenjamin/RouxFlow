//! Single WASM entry point for RouxFlow.
//!
//! Re-exports from: rouxflow-core, rouxflow-render, rouxflow-bluetoothcube, rouxflow-storage.
//! This is the only crate compiled as `cdylib` for wasm-pack.

use std::cell::{Cell, RefCell};
use wasm_bindgen::prelude::*;
use rouxflow_bluetoothcube::codec::{self, CubeCommand, CubeEvent, CubeProtocol};
use rouxflow_core::cube::{CubeMove, Face, facelet::Color as FaceletColor};
use rouxflow_core::session::CoreAction;

// ========== INIT ==========

#[wasm_bindgen(start)]
pub fn wasm_init() {
    console_error_panic_hook::set_once();
}

// ========== CORE: Protocol-aware BLE packet handling ==========

/// Opaque protocol handler. Created by `create_protocol()`, used by
/// `process_packet()` and `encode_cube_command()`.
#[wasm_bindgen]
pub struct ProtocolHandler {
    inner: Box<dyn CubeProtocol>,
}

#[wasm_bindgen]
impl ProtocolHandler {
    /// Protocol display name.
    pub fn name(&self) -> String {
        self.inner.name().to_string()
    }

    /// Whether this protocol's cubes support gyroscope.
    pub fn has_gyro(&self) -> bool {
        self.inner.has_gyro()
    }

    /// Whether this protocol requires a handshake after BLE connection.
    pub fn requires_handshake(&self) -> bool {
        self.inner.requires_handshake()
    }

    /// Get handshake data to send after connection, if required.
    /// Returns null if no handshake needed.
    pub fn handshake_data(&self) -> Option<Vec<u8>> {
        self.inner.handshake_data()
    }
}

/// Create a protocol handler for a given protocol name and MAC address.
///
/// Protocol names: "GanV1", "GanV2", "GanV3", "GanV4", "MoYuAi", "MoYuV3",
/// "GiikerV1", "GoCube", "QiYiSmart".
#[wasm_bindgen]
pub fn create_protocol(protocol_name: &str, mac_address: &str) -> Result<ProtocolHandler, JsValue> {
    let protocol = match protocol_name {
        "GanV1" => rouxflow_bluetoothcube::ProtocolVersion::GanV1,
        "GanV2" => rouxflow_bluetoothcube::ProtocolVersion::GanV2,
        "GanV3" => rouxflow_bluetoothcube::ProtocolVersion::GanV3,
        "GanV4" => rouxflow_bluetoothcube::ProtocolVersion::GanV4,
        "MoYuAi" => rouxflow_bluetoothcube::ProtocolVersion::MoYuAi,
        "MoYuV3" => rouxflow_bluetoothcube::ProtocolVersion::MoYuV3,
        "GiikerV1" => rouxflow_bluetoothcube::ProtocolVersion::GiikerV1,
        "GoCube" => rouxflow_bluetoothcube::ProtocolVersion::GoCube,
        "QiYiSmart" => rouxflow_bluetoothcube::ProtocolVersion::QiYiSmart,
        _ => return Err(JsValue::from_str(&format!("Unknown protocol: {}", protocol_name))),
    };
    Ok(ProtocolHandler {
        inner: codec::create_protocol(protocol, mac_address),
    })
}

/// Process a raw BLE notification packet through the protocol handler.
///
/// Flow: raw bytes → decrypt → decode → Vec<CubeEvent> → process each through session.
/// Returns a JSON string of CoreAction results, or empty string if no actions.
#[wasm_bindgen]
pub fn process_packet(
    protocol: &mut ProtocolHandler,
    raw_data: &[u8],
    session: &mut SessionManager,
) -> String {
    let decrypted = protocol.inner.decrypt(raw_data);
    let events = protocol.inner.decode_event(&decrypted);

    let mut actions: Vec<String> = Vec::new();

    for event in events {
        let action = handle_cube_event(&event, &mut session.inner);
        if !action.is_empty() {
            actions.push(action);
        }
    }

    if actions.is_empty() {
        String::new()
    } else if actions.len() == 1 {
        actions.into_iter().next().unwrap()
    } else {
        format!("[{}]", actions.join(","))
    }
}

/// Encrypt and encode a cube command for sending via BLE.
///
/// Command names: "facelets", "hardware", "battery", "reset".
#[wasm_bindgen]
pub fn encode_cube_command(protocol: &ProtocolHandler, command_name: &str) -> Result<Vec<u8>, JsValue> {
    let cmd = match command_name {
        "facelets" => CubeCommand::RequestFacelets,
        "hardware" => CubeCommand::RequestHardware,
        "battery" => CubeCommand::RequestBattery,
        "reset" => CubeCommand::RequestReset,
        _ => return Err(JsValue::from_str(&format!("Unknown command: {}", command_name))),
    };

    match protocol.inner.create_command(cmd) {
        Some(msg) => Ok(protocol.inner.encrypt(&msg)),
        None => Err(JsValue::from_str("Command not supported by this protocol")),
    }
}

/// Process a CubeEvent through the session manager.
fn handle_cube_event(
    event: &CubeEvent,
    session: &mut rouxflow_core::session::SessionManager,
) -> String {
    match event {
        CubeEvent::Move { face, direction, .. } => {
            let cube_move = CubeMove {
                face: *face,
                amount: *direction,
            };
            let notation = cube_move.notation();
            let now = chrono::Utc::now().timestamp_millis() as f64 / 1000.0;

            // Try scramble validation first
            let action = session.handle_scramble_move(&notation, now);
            if !action.is_empty() {
                return action;
            }

            // If not in scramble phase, emit as a plain move
            serde_json::to_string(&CoreAction::Move(notation)).unwrap_or_default()
        }
        CubeEvent::Gyro { quaternion, .. } => {
            let now = chrono::Utc::now().timestamp_millis() as f64 / 1000.0;
            session.process_orientation(quaternion.x, quaternion.y, quaternion.z, quaternion.w, now)
        }
        CubeEvent::Battery { level } => {
            // TODO: route to frontend via a BatteryUpdate action
            format!(r#"{{"type":"Battery","data":{}}}"#, level)
        }
        CubeEvent::Hardware { name, sw_version, hw_version, gyro_supported } => {
            format!(
                r#"{{"type":"Hardware","data":{{"name":"{}","swVersion":"{}","hwVersion":"{}","gyroSupported":{}}}}}"#,
                name, sw_version, hw_version, gyro_supported
            )
        }
        CubeEvent::Facelets { cp, co, ep, eo, .. } => {
            // Update the cube state in the session manager's internal cube
            // For now, return the state as JSON for the frontend
            format!(
                r#"{{"type":"Facelets","data":{{"cp":{:?},"co":{:?},"ep":{:?},"eo":{:?}}}}}"#,
                cp, co, ep, eo
            )
        }
        CubeEvent::Disconnect => {
            r#"{"type":"Disconnect"}"#.to_string()
        }
        CubeEvent::MoveHistory { moves } => {
            // Process each recovered move
            let mut last_action = String::new();
            for m in moves {
                let action = handle_cube_event(m, session);
                if !action.is_empty() {
                    last_action = action;
                }
            }
            last_action
        }
        CubeEvent::RawFacelets { facelet_string } => {
            format!(r#"{{"type":"RawFacelets","data":"{}"}}"#, facelet_string)
        }
        CubeEvent::WriteBack { data } => {
            // Return write-back data as a JSON array of bytes for the TS bridge to send
            let bytes: Vec<String> = data.iter().map(|b| b.to_string()).collect();
            format!(r#"{{"type":"WriteBack","data":[{}]}}"#, bytes.join(","))
        }
    }
}

// ========== Legacy compatibility wrappers ==========
// These maintain the old API for the bridge.ts until it's updated.

/// Legacy: handle_ble_packet (delegates to protocol auto-detection).
/// This tries GAN Gen2 with standard GAN keys as a fallback.
#[wasm_bindgen]
pub fn handle_ble_packet(data: &[u8], device_id: &str, session: &mut SessionManager) -> String {
    // Create a temporary GAN V2 protocol with standard keys
    let mut proto = codec::create_protocol(
        rouxflow_bluetoothcube::ProtocolVersion::GanV2,
        device_id,
    );
    let decrypted = proto.decrypt(data);
    let events = proto.decode_event(&decrypted);

    for event in &events {
        let action = handle_cube_event(event, &mut session.inner);
        if !action.is_empty() {
            return action;
        }
    }

    // Try MoYu AI keys
    let mut proto_moyu = codec::create_protocol(
        rouxflow_bluetoothcube::ProtocolVersion::MoYuAi,
        device_id,
    );
    let decrypted_moyu = proto_moyu.decrypt(data);
    let events_moyu = proto_moyu.decode_event(&decrypted_moyu);

    for event in &events_moyu {
        let action = handle_cube_event(event, &mut session.inner);
        if !action.is_empty() {
            return action;
        }
    }

    String::new()
}

#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    rouxflow_core::greet(name)
}

// ========== CORE: SessionManager wrapper ==========

#[wasm_bindgen]
pub struct SessionManager {
    inner: rouxflow_core::session::SessionManager,
}

#[wasm_bindgen]
impl SessionManager {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: rouxflow_core::session::SessionManager::new(),
        }
    }

    pub fn set_active_session(&mut self, session_json: &str) {
        self.inner.set_active_session(session_json);
    }

    pub fn add_solve(&mut self, solve_json: &str) -> String {
        self.inner.add_solve(solve_json)
    }

    pub fn process_orientation(&mut self, x: f32, y: f32, z: f32, w: f32, timestamp: f64) -> String {
        self.inner.process_orientation(x, y, z, w, timestamp)
    }

    pub fn create_session(&mut self, name: String, session_type: &str) -> String {
        let st = match session_type {
            "WCA" => rouxflow_core::session::SessionType::WCA,
            _ => rouxflow_core::session::SessionType::Free,
        };
        self.inner.create_session(name, st)
    }

    pub fn start_scramble(&mut self, scramble: &str) -> String {
        self.inner.start_scramble(scramble)
    }

    pub fn reset_flow(&mut self) -> String {
        self.inner.reset_flow()
    }

    pub fn get_active_session_json(&self) -> String {
        self.inner.get_active_session_json()
    }

    pub fn handle_scramble_move(&mut self, move_str: &str, timestamp: f64) -> String {
        self.inner.handle_scramble_move(move_str, timestamp)
    }

    pub fn record_solve(&mut self, time_ms: u32, moves_json: &str) -> String {
        self.inner.record_solve(time_ms, moves_json)
    }

    pub fn set_solving(&mut self) -> String {
        self.inner.set_solving()
    }

    pub fn get_flow_state(&self) -> String {
        self.inner.get_flow_state()
    }

    pub fn is_scramble_ready(&self) -> bool {
        self.inner.is_scramble_ready()
    }

    pub fn is_scramble_invalid(&self) -> bool {
        self.inner.is_scramble_invalid()
    }

    pub fn get_scramble_index(&self) -> usize {
        self.inner.get_scramble_index()
    }

    pub fn get_scramble_len(&self) -> usize {
        self.inner.get_scramble_len()
    }
}

// ========== RENDER: re-export WASM functions ==========
// These functions are cfg(wasm32) in rouxflow-render, so we gate them here too.

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn init_renderer(canvas_id: String) -> Result<(), JsValue> {
    rouxflow_render::init_renderer(canvas_id)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn set_gyro_enabled(enabled: bool) {
    rouxflow_render::set_gyro_enabled(enabled)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn update_render_state(facelets: Vec<u8>, x: f32, y: f32, z: f32, w: f32) {
    rouxflow_render::update_render_state(facelets, x, y, z, w)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn reset_gyro() {
    rouxflow_render::reset_gyro()
}

// ========== BLUETOOTHCUBE: cube registry ==========

/// Find cube definition by BLE name. Returns JSON string or empty string if not found.
/// Avoids wasm_bindgen JsValue/Reflect alloc churn that corrupts dlmalloc.
#[wasm_bindgen]
pub fn find_cube_by_ble_name(device_name: String) -> String {
    match rouxflow_bluetoothcube::find_cube_by_ble_name(&device_name) {
        Some(cube) => {
            let has_gyro = cube.features.contains(&rouxflow_bluetoothcube::CubeFeature::Gyroscope);
            let profile = cube.protocol.ble_profile();
            format!(
                r#"{{"name":"{}","manufacturer":"{:?}","protocol":"{:?}","hasGyro":{},"serviceUuid":"{}","stateCharacteristic":"{}","commandCharacteristic":"{}"}}"#,
                cube.name,
                cube.manufacturer,
                cube.protocol,
                has_gyro,
                profile.service_uuid,
                profile.state_characteristic,
                profile.command_characteristic,
            )
        }
        None => String::new(),
    }
}

/// Get all scan service UUIDs as JSON array string.
#[wasm_bindgen]
pub fn all_scan_service_uuids() -> String {
    let uuids = rouxflow_bluetoothcube::all_scan_service_uuids();
    let parts: Vec<String> = uuids.iter().map(|s| format!("\"{}\"", s)).collect();
    format!("[{}]", parts.join(","))
}

/// Get all scan name prefixes as JSON array string.
#[wasm_bindgen]
pub fn all_scan_name_prefixes() -> String {
    let prefixes = rouxflow_bluetoothcube::all_scan_name_prefixes();
    let parts: Vec<String> = prefixes.iter().map(|s| format!("\"{}\"", s)).collect();
    format!("[{}]", parts.join(","))
}

// ========== STORAGE: StorageManager ==========
// StorageManager uses IndexedDB (rexie) which is wasm32-only.

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct WasmStorageManager {
    inner: rouxflow_storage::StorageManager,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WasmStorageManager {
    #[wasm_bindgen(constructor)]
    pub async fn new(supabase_url: Option<String>, supabase_key: Option<String>) -> Result<WasmStorageManager, JsValue> {
        let mgr = rouxflow_storage::StorageManager::new(supabase_url, supabase_key)
            .await
            .map_err(|e| JsValue::from_str(&e.message))?;
        Ok(Self { inner: mgr })
    }

    // --- Cubes ---

    pub async fn get_cubes_json(&self, user_id: Option<String>) -> Result<String, JsValue> {
        use rouxflow_core::storage::Storage;
        let cubes = self.inner.get_cubes(user_id.as_deref())
            .await
            .map_err(|e| JsValue::from_str(&e.message))?;
        serde_json::to_string(&cubes).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub async fn save_cube_json(&self, cube_json: &str) -> Result<(), JsValue> {
        use rouxflow_core::storage::Storage;
        let cube: rouxflow_core::storage::Cube =
            serde_json::from_str(cube_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.inner.save_cube(&cube)
            .await
            .map_err(|e| JsValue::from_str(&e.message))
    }

    pub async fn delete_cube(&self, id: &str, user_id: &str) -> Result<(), JsValue> {
        use rouxflow_core::storage::Storage;
        self.inner.delete_cube(id, user_id)
            .await
            .map_err(|e| JsValue::from_str(&e.message))
    }

    pub async fn sync_cubes(&self, user_id: &str) -> Result<(), JsValue> {
        self.inner.sync_cubes(user_id).await;
        Ok(())
    }

    // --- Sessions ---

    pub async fn get_sessions_json(&self) -> Result<String, JsValue> {
        use rouxflow_core::storage::Storage;
        let sessions = self.inner.get_sessions()
            .await
            .map_err(|e| JsValue::from_str(&e.message))?;
        serde_json::to_string(&sessions).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub async fn create_session_json(&self, session_json: &str) -> Result<(), JsValue> {
        use rouxflow_core::storage::Storage;
        let session: rouxflow_core::session::Session =
            serde_json::from_str(session_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.inner.create_session(&session)
            .await
            .map_err(|e| JsValue::from_str(&e.message))
    }

    pub async fn save_solve_json(&self, session_id: &str, solve_json: &str) -> Result<(), JsValue> {
        use rouxflow_core::storage::Storage;
        let solve: rouxflow_core::session::Solve =
            serde_json::from_str(solve_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.inner.save_solve(session_id, &solve)
            .await
            .map_err(|e| JsValue::from_str(&e.message))
    }

    pub async fn demote_session(&self, session_id: &str) -> Result<(), JsValue> {
        use rouxflow_core::storage::Storage;
        self.inner.demote_session(session_id)
            .await
            .map_err(|e| JsValue::from_str(&e.message))
    }
}

// ========== WASM CubeManager: thread_local state ==========
//
// State lives in thread_local! so borrows are scoped to .with() closures and
// can never leak across the JS↔WASM boundary. This prevents the "recursive use
// of an object" / "RefCell already borrowed" errors caused by reentrancy
// (e.g., a WASM call triggers a Vue reactive update that calls back into WASM).
//
// Same proven pattern as rouxflow-render's STATE thread_local.

struct CubeManagerState {
    inner: rouxflow_core::CubeManager,
    protocol: Option<Box<dyn CubeProtocol>>,
    /// Logical cube state — tracks moves and produces facelets for rendering
    cube_logic: rouxflow_core::cube::CubeState,
    /// Last decrypted gyro packet as hex string (for debug)
    last_gyro_hex: String,
}

thread_local! {
    static CM_STATE: RefCell<Option<CubeManagerState>> = RefCell::new(None);
    static IN_WASM_CALL: Cell<bool> = Cell::new(false);
}

/// Debug guard: detect reentrancy. Panics with a clear message identifying
/// which method was re-entered, so we can find the exact reentrant call path.
fn enter_wasm(label: &str) {
    IN_WASM_CALL.with(|flag| {
        if flag.get() {
            panic!("REENTRANT WASM CALL detected in: {}", label);
        }
        flag.set(true);
    });
}

fn exit_wasm() {
    IN_WASM_CALL.with(|flag| flag.set(false));
}

// ========== Free functions: no struct, no WasmRefCell, no borrow corruption ==========
//
// Why free functions instead of a struct?
// wasm-bindgen wraps every #[wasm_bindgen] struct in a WasmRefCell with a
// 4-byte borrow counter on the WASM heap. For small/ZST structs, dlmalloc
// can recycle that allocation for string arguments. The borrow-count
// increment then corrupts the first byte of whichever string landed at
// the same address (observed: "MoYuV3" → "NoYuV3", +1 on first byte).
//
// Free functions eliminate the struct entirely — no heap allocation,
// no borrow counter, no possible corruption. State lives in CM_STATE
// thread_local, accessed only via scoped .with() closures.

/// Initialize the CubeManager state. Call once at startup.
#[wasm_bindgen]
pub fn cm_init() {
    CM_STATE.with(|s| {
        let cube_logic = rouxflow_core::cube::CubeState::new();
        let mut cm = rouxflow_core::CubeManager::new();
        // Set initial facelets to solved state (not all-zeros)
        cm.update_facelets(&cube_logic.facelets());
        *s.borrow_mut() = Some(CubeManagerState {
            inner: cm,
            protocol: None,
            cube_logic,
            last_gyro_hex: String::new(),
        });
    });
}

// ========== Connection Management ==========

#[wasm_bindgen]
pub fn cm_connect(device_name: String, mac_address: String, protocol_name: String) -> Result<(), JsValue> {
    enter_wasm("cm_connect");
    let result = CM_STATE.with(|s| {
        let mut state = s.borrow_mut();
        let st = state.as_mut().ok_or_else(|| JsValue::from_str("CubeManager not initialized"))?;

        let protocol_version = match protocol_name.as_str() {
            "GanV1" => rouxflow_bluetoothcube::ProtocolVersion::GanV1,
            "GanV2" => rouxflow_bluetoothcube::ProtocolVersion::GanV2,
            "GanV3" => rouxflow_bluetoothcube::ProtocolVersion::GanV3,
            "GanV4" => rouxflow_bluetoothcube::ProtocolVersion::GanV4,
            "MoYuAi" => rouxflow_bluetoothcube::ProtocolVersion::MoYuAi,
            "MoYuV3" => rouxflow_bluetoothcube::ProtocolVersion::MoYuV3,
            "GiikerV1" => rouxflow_bluetoothcube::ProtocolVersion::GiikerV1,
            "GoCube" => rouxflow_bluetoothcube::ProtocolVersion::GoCube,
            "QiYiSmart" => rouxflow_bluetoothcube::ProtocolVersion::QiYiSmart,
            _ => return Err(JsValue::from_str(&format!("Unknown protocol: {}", protocol_name))),
        };

        let protocol = codec::create_protocol(protocol_version, &mac_address);
        let has_gyro = protocol.has_gyro();
        st.protocol = Some(protocol);
        st.inner.connect(device_name, mac_address, protocol_name, has_gyro);
        // Reset logical cube to solved; will be overwritten by RawFacelets from cube
        st.cube_logic = rouxflow_core::cube::CubeState::new();
        st.inner.update_facelets(&st.cube_logic.facelets());

        Ok(())
    });
    exit_wasm();
    result
}

#[wasm_bindgen]
pub fn cm_disconnect() {
    enter_wasm("cm_disconnect");
    CM_STATE.with(|s| {
        if let Some(st) = s.borrow_mut().as_mut() {
            st.inner.disconnect();
            st.protocol = None;
        }
    });
    exit_wasm();
}

#[wasm_bindgen]
pub fn cm_is_connected() -> bool {
    CM_STATE.with(|s| {
        s.borrow().as_ref().map_or(false, |st| st.inner.is_connected())
    })
}

#[wasm_bindgen]
pub fn cm_get_device_info() -> Option<String> {
    CM_STATE.with(|s| {
        s.borrow().as_ref().and_then(|st| st.inner.get_device_info_json())
    })
}

// ========== BLE Packet Processing ==========

/// Detect pairs of opposite-face moves that form slice moves (M, S, E).
/// Returns the slice notation string if the pair merges, None otherwise.
fn try_merge_slice(a: &CubeEvent, b: &CubeEvent) -> Option<String> {
    if let (
        CubeEvent::Move { face: f1, direction: d1, .. },
        CubeEvent::Move { face: f2, direction: d2, .. },
    ) = (a, b) {
        // Only merge if opposite faces with opposite directions
        if *d1 != -(*d2) {
            return None;
        }
        let suffix = |d: i8| if d > 0 { "" } else { "'" };
        match (*f1, *f2) {
            (Face::L, Face::R) | (Face::R, Face::L) => {
                // M follows L direction
                let dir = if *f1 == Face::L { *d1 } else { *d2 };
                Some(format!("M{}", suffix(dir)))
            }
            (Face::F, Face::B) | (Face::B, Face::F) => {
                // S follows F direction
                let dir = if *f1 == Face::F { *d1 } else { *d2 };
                Some(format!("S{}", suffix(dir)))
            }
            (Face::U, Face::D) | (Face::D, Face::U) => {
                // E follows D direction
                let dir = if *f1 == Face::D { *d1 } else { *d2 };
                Some(format!("E{}", suffix(dir)))
            }
            _ => None,
        }
    } else {
        None
    }
}

/// Process a raw BLE packet through the protocol handler and update state.
/// Returns JSON array of CoreActions.
#[wasm_bindgen]
pub fn cm_process_ble_packet(raw_data: &[u8], timestamp: f64) -> String {
    enter_wasm("cm_process_ble_packet");
    let result = CM_STATE.with(|s| {
        let mut state = s.borrow_mut();
        let st = match state.as_mut() {
            Some(st) => st,
            None => return String::new(),
        };

        let protocol = match st.protocol.as_mut() {
            Some(p) => p,
            None => return String::new(),
        };

        let decrypted = protocol.decrypt(raw_data);
        let events = protocol.decode_event(&decrypted);

        // Store decrypted hex if any event is Gyro (for gyroDebug.show())
        if events.iter().any(|e| matches!(e, CubeEvent::Gyro { .. })) {
            st.last_gyro_hex = decrypted.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ");
        }

        let mut actions: Vec<String> = Vec::new();

        let mut i = 0;
        while i < events.len() {
            // Lookahead: try to merge adjacent Move events into slice notation (M/S/E)
            // The cube firmware reports L+R' for an M move. We apply both individual
            // face moves to keep facelets in sync with the cube's internal tracking,
            // but queue a single slice animation and record the slice notation.
            if i + 1 < events.len() {
                if let Some(slice_notation) = try_merge_slice(&events[i], &events[i + 1]) {
                    // Apply both individual face moves for correct facelet state
                    if let (
                        CubeEvent::Move { face: f1, direction: d1, .. },
                        CubeEvent::Move { face: f2, direction: d2, .. },
                    ) = (&events[i], &events[i + 1]) {
                        let n1 = CubeMove { face: *f1, amount: *d1 }.notation();
                        let n2 = CubeMove { face: *f2, amount: *d2 }.notation();
                        st.cube_logic.apply_move(&n1);
                        st.cube_logic.apply_move(&n2);
                    }
                    st.inner.update_facelets(&st.cube_logic.facelets());

                    // Record and animate as single slice move
                    st.inner.record_move(slice_notation.clone());
                    rouxflow_render::queue_move_anim(slice_notation.clone(), 0.15);

                    // Scramble: pass individual face moves (scrambles never use M/S/E)
                    if let (
                        CubeEvent::Move { face: f1, direction: d1, .. },
                        CubeEvent::Move { face: f2, direction: d2, .. },
                    ) = (&events[i], &events[i + 1]) {
                        let n1 = CubeMove { face: *f1, amount: *d1 }.notation();
                        let n2 = CubeMove { face: *f2, amount: *d2 }.notation();
                        let a1 = st.inner.handle_scramble_move(&n1, timestamp);
                        if !a1.is_empty() { actions.push(a1); }
                        let a2 = st.inner.handle_scramble_move(&n2, timestamp);
                        if !a2.is_empty() { actions.push(a2); }
                    }

                    // Emit slice move action to frontend
                    let action_json = serde_json::to_string(&CoreAction::Move(slice_notation))
                        .unwrap_or_default();
                    if !action_json.is_empty() {
                        actions.push(action_json);
                    }

                    i += 2; // Skip both events
                    continue;
                }
            }

            match &events[i] {
                CubeEvent::Move { face, direction, .. } => {
                    let cube_move = CubeMove {
                        face: *face,
                        amount: *direction,
                    };
                    let notation = cube_move.notation();

                    st.inner.record_move(notation.clone());
                    // Apply move to logical cube and update render facelets
                    st.cube_logic.apply_move(&notation);
                    st.inner.update_facelets(&st.cube_logic.facelets());
                    // Queue slice animation in the renderer (0.15s like standalone)
                    rouxflow_render::queue_move_anim(notation.clone(), 0.15);

                    let action = st.inner.handle_scramble_move(&notation, timestamp);
                    if !action.is_empty() {
                        actions.push(action);
                    } else {
                        let action_json = serde_json::to_string(&CoreAction::Move(notation))
                            .unwrap_or_default();
                        if !action_json.is_empty() {
                            actions.push(action_json);
                        }
                    }
                }
                CubeEvent::Gyro { quaternion, .. } => {
                    st.inner.update_orientation(
                        quaternion.x,
                        quaternion.y,
                        quaternion.z,
                        quaternion.w,
                    );

                    let action = st.inner.get_session_manager_mut().process_orientation(
                        quaternion.x,
                        quaternion.y,
                        quaternion.z,
                        quaternion.w,
                        timestamp,
                    );
                    if !action.is_empty() {
                        actions.push(action);
                    }

                    // GyroRaw debug disabled (was spamming logs)
                }
                CubeEvent::Battery { level } => {
                    st.inner.update_battery(*level);
                    actions.push(format!(r#"{{"type":"Battery","data":{}}}"#, level));
                }
                CubeEvent::Hardware { name, sw_version, hw_version, gyro_supported } => {
                    st.inner.update_hardware(sw_version.clone(), hw_version.clone());
                    actions.push(format!(
                        r#"{{"type":"Hardware","data":{{"name":"{}","swVersion":"{}","hwVersion":"{}","gyroSupported":{}}}}}"#,
                        name, sw_version, hw_version, gyro_supported
                    ));
                }
                CubeEvent::Facelets { cp, co, ep, eo, .. } => {
                    actions.push(format!(
                        r#"{{"type":"Facelets","data":{{"cp":{:?},"co":{:?},"ep":{:?},"eo":{:?}}}}}"#,
                        cp, co, ep, eo
                    ));
                }
                CubeEvent::RawFacelets { facelet_string } => {
                    // Parse 54-char facelet string (URFDLB order) into logical cube state
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
                        st.cube_logic.logic.facelets = colors;
                        st.inner.update_facelets(&st.cube_logic.facelets());
                    }
                    actions.push(format!(r#"{{"type":"RawFacelets","data":"{}"}}"#, facelet_string));
                }
                _ => {
                    let action = handle_cube_event(&events[i], st.inner.get_session_manager_mut());
                    if !action.is_empty() {
                        actions.push(action);
                    }
                }
            }

            i += 1;
        }

        if actions.is_empty() {
            String::new()
        } else if actions.len() == 1 {
            actions.into_iter().next().unwrap()
        } else {
            format!("[{}]", actions.join(","))
        }
    });
    exit_wasm();
    result
}

/// Encode a command using the current protocol. Returns JSON array of bytes.
#[wasm_bindgen]
pub fn cm_encode_command(command_name: String) -> Result<String, JsValue> {
    CM_STATE.with(|s| {
        let state = s.borrow();
        let st = state.as_ref().ok_or_else(|| JsValue::from_str("CubeManager not initialized"))?;
        let protocol = st.protocol.as_ref()
            .ok_or_else(|| JsValue::from_str("No protocol configured"))?;

        let cmd = match command_name.as_str() {
            "facelets" => CubeCommand::RequestFacelets,
            "hardware" => CubeCommand::RequestHardware,
            "battery" => CubeCommand::RequestBattery,
            "reset" => CubeCommand::RequestReset,
            _ => return Err(JsValue::from_str(&format!("Unknown command: {}", command_name))),
        };

        match protocol.create_command(cmd) {
            Some(msg) => {
                let encrypted = protocol.encrypt(&msg);
                let parts: Vec<String> = encrypted.iter().map(|b| b.to_string()).collect();
                Ok(format!("[{}]", parts.join(",")))
            }
            None => Err(JsValue::from_str("Command not supported by this protocol")),
        }
    })
}

// ========== Cube State Queries ==========

#[wasm_bindgen]
pub fn cm_get_cube_state() -> String {
    CM_STATE.with(|s| {
        s.borrow().as_ref().map_or_else(|| "{}".to_string(), |st| st.inner.get_cube_state_json())
    })
}

#[wasm_bindgen]
pub fn cm_get_orientation() -> String {
    CM_STATE.with(|s| {
        s.borrow().as_ref().map_or_else(
            || "[0,0,0,1]".to_string(),
            |st| {
                let o = st.inner.get_orientation();
                format!("[{},{},{},{}]", o[0], o[1], o[2], o[3])
            },
        )
    })
}

#[wasm_bindgen]
pub fn cm_get_facelets() -> String {
    CM_STATE.with(|s| {
        s.borrow().as_ref().map_or_else(
            || "[]".to_string(),
            |st| {
                let f = st.inner.get_facelets();
                let parts: Vec<String> = f.iter().map(|b| b.to_string()).collect();
                format!("[{}]", parts.join(","))
            },
        )
    })
}

// ========== Timer Management ==========

#[wasm_bindgen]
pub fn cm_start_timer(timestamp: f64) {
    enter_wasm("cm_start_timer");
    CM_STATE.with(|s| {
        if let Some(st) = s.borrow_mut().as_mut() {
            st.inner.start_timer(timestamp);
        }
    });
    exit_wasm();
}

#[wasm_bindgen]
pub fn cm_stop_timer(timestamp: f64) {
    enter_wasm("cm_stop_timer");
    CM_STATE.with(|s| {
        if let Some(st) = s.borrow_mut().as_mut() {
            st.inner.stop_timer(timestamp);
        }
    });
    exit_wasm();
}

#[wasm_bindgen]
pub fn cm_update_timer(timestamp: f64) {
    CM_STATE.with(|s| {
        if let Some(st) = s.borrow_mut().as_mut() {
            st.inner.update_timer(timestamp);
        }
    });
}

#[wasm_bindgen]
pub fn cm_get_timer_state() -> String {
    CM_STATE.with(|s| {
        s.borrow().as_ref().map_or_else(|| "{}".to_string(), |st| st.inner.get_timer_state_json())
    })
}

#[wasm_bindgen]
pub fn cm_is_timer_running() -> bool {
    CM_STATE.with(|s| {
        s.borrow().as_ref().map_or(false, |st| st.inner.is_timer_running())
    })
}

#[wasm_bindgen]
pub fn cm_get_current_time_ms() -> u64 {
    CM_STATE.with(|s| {
        s.borrow().as_ref().map_or(0, |st| st.inner.get_current_time_ms())
    })
}

// ========== Session Management ==========

#[wasm_bindgen]
pub fn cm_get_flow_state() -> String {
    CM_STATE.with(|s| {
        s.borrow().as_ref().map_or_else(|| "\"Idle\"".to_string(), |st| st.inner.get_flow_state())
    })
}

#[wasm_bindgen]
pub fn cm_set_active_session(session_json: &str) {
    CM_STATE.with(|s| {
        if let Some(st) = s.borrow_mut().as_mut() {
            st.inner.set_active_session(session_json);
        }
    });
}

#[wasm_bindgen]
pub fn cm_create_session(session_json: &str) {
    CM_STATE.with(|s| {
        if let Some(st) = s.borrow_mut().as_mut() {
            st.inner.create_session(session_json);
        }
    });
}

#[wasm_bindgen]
pub fn cm_start_scramble(scramble: &str) -> String {
    CM_STATE.with(|s| {
        s.borrow_mut().as_mut().map_or_else(String::new, |st| st.inner.start_scramble(scramble))
    })
}

#[wasm_bindgen]
pub fn cm_handle_scramble_move(move_str: &str) -> String {
    let timestamp = js_sys::Date::now() / 1000.0;
    CM_STATE.with(|s| {
        s.borrow_mut().as_mut().map_or_else(String::new, |st| {
            st.inner.handle_scramble_move(move_str, timestamp)
        })
    })
}

#[wasm_bindgen]
pub fn cm_set_solving() -> String {
    let timestamp = js_sys::Date::now() / 1000.0;
    CM_STATE.with(|s| {
        s.borrow_mut().as_mut().map_or_else(String::new, |st| st.inner.set_solving(timestamp))
    })
}

#[wasm_bindgen]
pub fn cm_record_solve(time_ms: u32, moves_json: &str) -> String {
    let timestamp = js_sys::Date::now() / 1000.0;
    CM_STATE.with(|s| {
        s.borrow_mut().as_mut().map_or_else(String::new, |st| {
            st.inner.record_solve(timestamp, time_ms, moves_json)
        })
    })
}

// ========== MAC Address Validation ==========

// ========== Protocol Queries ==========

#[wasm_bindgen]
pub fn cm_needs_mac_input(device_id: &str, protocol: &str) -> bool {
    rouxflow_core::CubeManager::needs_mac_input(device_id, protocol)
}

#[wasm_bindgen]
pub fn cm_requires_handshake() -> bool {
    CM_STATE.with(|s| {
        s.borrow().as_ref()
            .and_then(|st| st.protocol.as_ref())
            .map_or(false, |p| p.requires_handshake())
    })
}

#[wasm_bindgen]
pub fn cm_handshake_data() -> Vec<u8> {
    CM_STATE.with(|s| {
        s.borrow().as_ref()
            .and_then(|st| st.protocol.as_ref())
            .and_then(|p| p.handshake_data())
            .unwrap_or_default()
    })
}

#[wasm_bindgen]
pub fn cm_protocol_requires_mac(protocol: &str) -> bool {
    rouxflow_core::CubeManager::protocol_requires_mac(protocol)
}

#[wasm_bindgen]
pub fn cm_is_valid_mac_format(device_id: &str) -> bool {
    rouxflow_core::CubeManager::is_valid_mac_format(device_id)
}

// ========== Debug: gyro raw hex ==========

/// Get the last decrypted gyro packet as hex string.
/// Usage from console: gyroDebug.show()
#[wasm_bindgen]
pub fn cm_get_last_gyro_hex() -> String {
    CM_STATE.with(|s| {
        s.borrow().as_ref().map_or_else(
            || "No state".to_string(),
            |st| {
                if st.last_gyro_hex.is_empty() {
                    "No gyro packet received yet".to_string()
                } else {
                    st.last_gyro_hex.clone()
                }
            },
        )
    })
}

// ========== Debug: decrypt/encrypt hex strings ==========

/// Decrypt a hex string using the current protocol. Returns decrypted hex.
/// Usage from console: cubeDebug.decode("90 5c 36 ...")
#[wasm_bindgen]
pub fn cm_decrypt_hex(hex_input: &str) -> String {
    CM_STATE.with(|s| {
        let state = s.borrow();
        let st = match state.as_ref() {
            Some(st) => st,
            None => return "ERROR: No cube connected".to_string(),
        };
        let protocol = match st.protocol.as_ref() {
            Some(p) => p,
            None => return "ERROR: No protocol configured".to_string(),
        };

        let bytes = parse_hex_string(hex_input);
        if bytes.is_empty() {
            return "ERROR: Invalid hex".to_string();
        }

        let decrypted = protocol.decrypt(&bytes);
        decrypted.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ")
    })
}

/// Encrypt a hex string using the current protocol. Returns encrypted hex.
/// Usage from console: cubeDebug.encode("a4 00 00 ...")
#[wasm_bindgen]
pub fn cm_encrypt_hex(hex_input: &str) -> String {
    CM_STATE.with(|s| {
        let state = s.borrow();
        let st = match state.as_ref() {
            Some(st) => st,
            None => return "ERROR: No cube connected".to_string(),
        };
        let protocol = match st.protocol.as_ref() {
            Some(p) => p,
            None => return "ERROR: No protocol configured".to_string(),
        };

        let bytes = parse_hex_string(hex_input);
        if bytes.is_empty() {
            return "ERROR: Invalid hex".to_string();
        }

        let encrypted = protocol.encrypt(&bytes);
        encrypted.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ")
    })
}

fn parse_hex_string(hex: &str) -> Vec<u8> {
    let hex_clean: String = hex.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    let mut bytes = Vec::new();
    let mut i = 0;
    while i + 1 < hex_clean.len() {
        if let Ok(b) = u8::from_str_radix(&hex_clean[i..i + 2], 16) {
            bytes.push(b);
        }
        i += 2;
    }
    bytes
}
