//! Single WASM entry point for RouxFlow.
//!
//! Re-exports from: rouxflow-core, rouxflow-render, rouxflow-bluetoothcube, rouxflow-storage.
//! This is the only crate compiled as `cdylib` for wasm-pack.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use log::debug;
use rouxflow_bluetoothcube::codec::{self, CubeCommand, CubeEvent, CubeProtocol};
use rouxflow_core::cube::{CubeMove, facelet::Color as FaceletColor};
use rouxflow_core::session::CoreAction;
use rouxflow_core::telemetry::{GyroSample, RawMove, SolveTelemetry};

// ========== INIT ==========

#[wasm_bindgen(start)]
pub fn wasm_init() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).ok();
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
            format!(r#"{{"type":"Battery","data":{}}}"#, level)
        }
        CubeEvent::Hardware { name, sw_version, hw_version, gyro_supported } => {
            format!(
                r#"{{"type":"Hardware","data":{{"name":"{}","swVersion":"{}","hwVersion":"{}","gyroSupported":{}}}}}"#,
                name, sw_version, hw_version, gyro_supported
            )
        }
        CubeEvent::Facelets { cp, co, ep, eo, .. } => {
            format!(
                r#"{{"type":"Facelets","data":{{"cp":{:?},"co":{:?},"ep":{:?},"eo":{:?}}}}}"#,
                cp, co, ep, eo
            )
        }
        CubeEvent::Disconnect => {
            r#"{"type":"Disconnect"}"#.to_string()
        }
        CubeEvent::MoveHistory { moves } => {
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
            let bytes: Vec<String> = data.iter().map(|b| b.to_string()).collect();
            format!(r#"{{"type":"WriteBack","data":[{}]}}"#, bytes.join(","))
        }
    }
}

// ========== RENDER: re-export WASM functions ==========

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

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn set_gyro_offset(x: f32, y: f32, z: f32, w: f32) {
    rouxflow_render::set_gyro_offset(x, y, z, w)
}

// ========== BLUETOOTHCUBE: cube registry ==========

/// Find cube definition by BLE name. Returns JSON string or empty string if not found.
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

// ========== STORAGE: WasmStorageManager ==========

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

    pub async fn get_sessions_json(&self, user_id: Option<String>) -> Result<String, JsValue> {
        use rouxflow_core::storage::Storage;
        let sessions = self.inner.get_sessions(user_id.as_deref())
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

// ========== WASM AppState: thread_local state ==========
//
// Two thread-locals:
//   APP_STATE — synchronous core state (AppState + protocol + cube logic)
//   STORAGE   — Rc<StorageManager> (cloned out for async .await)
//
// Storage is async (IndexedDB). RefCell can't be held across .await.
// Solution: clone Rc out of RefCell, drop borrow, then .await freely.

struct WasmAppState {
    inner: rouxflow_core::AppState,
    protocol: Option<Box<dyn CubeProtocol>>,
    /// Logical cube state — tracks moves and produces facelets for rendering
    cube_logic: rouxflow_core::cube::CubeState,
    /// Last decrypted gyro packet as hex string (for debug)
    last_gyro_hex: String,
    /// Last 500 gyro packet arrival timestamps (seconds) for interval analysis
    gyro_timestamps: Vec<f64>,
    /// Debug: last gyro quaternion (for move-gyro logging)
    debug_last_gyro: Option<rouxflow_core::cube::Quaternion>,
    /// Debug: moves waiting for their "after" gyro reading
    debug_moves_pending_gyro: Vec<String>,
    /// Debug: whether we've logged the home quaternion yet this solve
    debug_home_logged: bool,
    /// Telemetry: raw solve data for future analyzer
    telemetry: Option<SolveTelemetry>,
    /// Telemetry recording active?
    telemetry_recording: bool,
    /// Telemetry phase: 0=idle, 1=scramble, 2=solve
    telemetry_phase: u8,
}

thread_local! {
    static APP_STATE: RefCell<Option<WasmAppState>> = RefCell::new(None);
    static STORAGE: RefCell<Option<Rc<rouxflow_storage::StorageManager>>> = RefCell::new(None);
    static IN_WASM_CALL: Cell<bool> = Cell::new(false);
}

/// Debug guard: detect reentrancy.
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

/// Initialize the AppState. Call once at startup.
#[wasm_bindgen]
pub fn cm_init() {
    APP_STATE.with(|s| {
        let cube_logic = rouxflow_core::cube::CubeState::new();
        *s.borrow_mut() = Some(WasmAppState {
            inner: rouxflow_core::AppState::new(),
            protocol: None,
            cube_logic,
            last_gyro_hex: String::new(),
            gyro_timestamps: Vec::with_capacity(500),
            debug_last_gyro: None,
            debug_moves_pending_gyro: Vec::new(),
            debug_home_logged: false,
            telemetry: None,
            telemetry_recording: false,
            telemetry_phase: 0,
        });
    });
}

/// Initialize storage, load sessions, ensure DefaultSession, set active, load solves.
/// Call once after cm_init(). Pass user_id if the user is authenticated.
#[wasm_bindgen]
pub async fn cm_init_storage(supabase_url: Option<String>, supabase_key: Option<String>, user_id: Option<String>) -> Result<(), JsValue> {
    use rouxflow_core::storage::Storage;

    // Create StorageManager
    let mgr = rouxflow_storage::StorageManager::new(supabase_url, supabase_key)
        .await
        .map_err(|e| JsValue::from_str(&e.message))?;
    let storage = Rc::new(mgr);

    // Store in thread-local
    STORAGE.with(|s| {
        *s.borrow_mut() = Some(storage.clone());
    });

    // Load sessions from IndexedDB (filtered by user)
    let sessions = storage.get_sessions(user_id.as_deref()).await
        .map_err(|e| JsValue::from_str(&e.message))?;

    // Load into AppState and ensure default session
    let (default_session_to_persist, default_id) = APP_STATE.with(|s| {
        let mut state = s.borrow_mut();
        let st = state.as_mut().expect("cm_init must be called before cm_init_storage");
        st.inner.session.set_user_id(user_id);
        st.inner.session.load_sessions(sessions);
        let default_id = st.inner.session.default_session_id();
        let new_session = st.inner.session.ensure_default_session();
        (new_session, default_id)
    });

    // Persist new default session if it was just created
    if let Some(session) = default_session_to_persist {
        storage.create_session(&session).await
            .map_err(|e| JsValue::from_str(&e.message))?;
    }

    // Set default as active session
    APP_STATE.with(|s| {
        let mut state = s.borrow_mut();
        let st = state.as_mut().unwrap();
        st.inner.session.set_active_session_by_id(&default_id);
    });

    // Load solves for active session
    let solves = storage.get_solves(&default_id).await
        .map_err(|e| JsValue::from_str(&e.message))?;

    APP_STATE.with(|s| {
        let mut state = s.borrow_mut();
        let st = state.as_mut().unwrap();
        st.inner.session.load_solves_into_active(solves);
    });

    Ok(())
}

/// Persist a solve to IndexedDB.
#[wasm_bindgen]
pub async fn cm_persist_solve(session_id: &str, solve_json: &str) -> Result<(), JsValue> {
    use rouxflow_core::storage::Storage;

    let storage = STORAGE.with(|s| s.borrow().clone())
        .ok_or_else(|| JsValue::from_str("Storage not initialized"))?;

    let solve: rouxflow_core::session::Solve =
        serde_json::from_str(solve_json).map_err(|e| JsValue::from_str(&e.to_string()))?;

    storage.save_solve(session_id, &solve).await
        .map_err(|e| JsValue::from_str(&e.message))?;

    Ok(())
}

/// Create a new session in-memory + persist to IndexedDB.
#[wasm_bindgen]
pub async fn cm_create_session_persist(name: &str, session_type: &str) -> Result<String, JsValue> {
    use rouxflow_core::storage::Storage;

    let session_json = APP_STATE.with(|s| {
        let mut state = s.borrow_mut();
        let st = state.as_mut().ok_or_else(|| JsValue::from_str("AppState not initialized"))?;
        let st_enum = match session_type {
            "WCA" => rouxflow_core::session::SessionType::WCA,
            _ => rouxflow_core::session::SessionType::Free,
        };
        Ok::<String, JsValue>(st.inner.session.create_session(name.to_string(), st_enum))
    })?;

    // Persist to IndexedDB
    let storage = STORAGE.with(|s| s.borrow().clone())
        .ok_or_else(|| JsValue::from_str("Storage not initialized"))?;

    let session: rouxflow_core::session::Session =
        serde_json::from_str(&session_json).map_err(|e| JsValue::from_str(&e.to_string()))?;

    storage.create_session(&session).await
        .map_err(|e| JsValue::from_str(&e.message))?;

    Ok(session_json)
}

/// Load solves for the active session from IndexedDB into memory.
#[wasm_bindgen]
pub async fn cm_load_active_session_solves() -> Result<(), JsValue> {
    use rouxflow_core::storage::Storage;

    let session_id = APP_STATE.with(|s| {
        s.borrow().as_ref()
            .and_then(|st| st.inner.session.get_active_session_id().map(|s| s.to_string()))
    }).ok_or_else(|| JsValue::from_str("No active session"))?;

    let storage = STORAGE.with(|s| s.borrow().clone())
        .ok_or_else(|| JsValue::from_str("Storage not initialized"))?;

    let solves = storage.get_solves(&session_id).await
        .map_err(|e| JsValue::from_str(&e.message))?;

    APP_STATE.with(|s| {
        let mut state = s.borrow_mut();
        if let Some(st) = state.as_mut() {
            st.inner.session.load_solves_into_active(solves);
        }
    });

    Ok(())
}

/// Re-persist the active session to IndexedDB (after settings change like pickup_mode).
#[wasm_bindgen]
pub async fn cm_save_active_session() -> Result<(), JsValue> {
    use rouxflow_core::storage::Storage;

    let session_json = APP_STATE.with(|s| {
        s.borrow().as_ref()
            .map(|st| st.inner.session.get_active_session_json())
    }).ok_or_else(|| JsValue::from_str("No active session"))?;

    let session: rouxflow_core::session::Session =
        serde_json::from_str(&session_json).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let storage = STORAGE.with(|s| s.borrow().clone())
        .ok_or_else(|| JsValue::from_str("Storage not initialized"))?;

    storage.create_session(&session).await
        .map_err(|e| JsValue::from_str(&e.message))?;

    Ok(())
}

// ========== Sync query functions ==========

#[wasm_bindgen]
pub fn cm_get_sessions_json() -> String {
    APP_STATE.with(|s| {
        s.borrow().as_ref().map_or_else(|| "[]".to_string(), |st| st.inner.session.get_sessions_json())
    })
}

#[wasm_bindgen]
pub fn cm_get_active_session_json() -> String {
    APP_STATE.with(|s| {
        s.borrow().as_ref().map_or_else(|| "null".to_string(), |st| st.inner.session.get_active_session_json())
    })
}

#[wasm_bindgen]
pub fn cm_get_active_session_solves_json() -> String {
    APP_STATE.with(|s| {
        s.borrow().as_ref().map_or_else(|| "[]".to_string(), |st| st.inner.session.get_active_session_solves_json())
    })
}

#[wasm_bindgen]
pub fn cm_get_active_session_id() -> Option<String> {
    APP_STATE.with(|s| {
        s.borrow().as_ref()
            .and_then(|st| st.inner.session.get_active_session_id().map(|s| s.to_string()))
    })
}

#[wasm_bindgen]
pub fn cm_switch_session(session_id: &str) -> bool {
    APP_STATE.with(|s| {
        let mut state = s.borrow_mut();
        match state.as_mut() {
            Some(st) => st.inner.session.set_active_session_by_id(session_id),
            None => false,
        }
    })
}

// ========== Connection Management ==========

#[wasm_bindgen]
pub fn cm_connect(device_name: String, mac_address: String, protocol_name: String) -> Result<(), JsValue> {
    enter_wasm("cm_connect");
    let result = APP_STATE.with(|s| {
        let mut state = s.borrow_mut();
        let st = state.as_mut().ok_or_else(|| JsValue::from_str("AppState not initialized"))?;

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
        debug!("[connect] device={}, protocol={}, has_gyro={}", device_name, protocol_name, has_gyro);
        st.protocol = Some(protocol);
        st.inner.bluetooth.connect(device_name, mac_address, protocol_name, has_gyro);
        // Reset logical cube to solved; will be overwritten by RawFacelets from cube
        st.cube_logic = rouxflow_core::cube::CubeState::new();
        // Configure interpreter and session for this cube
        st.inner.interpreter.set_has_gyro(has_gyro);
        st.inner.interpreter.reset();
        st.inner.session.set_has_gyro(has_gyro);

        Ok(())
    });
    exit_wasm();
    result
}

#[wasm_bindgen]
pub fn cm_disconnect() {
    enter_wasm("cm_disconnect");
    debug!("[connect] disconnecting");
    APP_STATE.with(|s| {
        if let Some(st) = s.borrow_mut().as_mut() {
            st.inner.disconnect();
            st.inner.session.set_has_gyro(false);
            st.protocol = None;
        }
    });
    exit_wasm();
}

#[wasm_bindgen]
pub fn cm_is_connected() -> bool {
    APP_STATE.with(|s| {
        s.borrow().as_ref().map_or(false, |st| st.inner.bluetooth.is_connected())
    })
}

#[wasm_bindgen]
pub fn cm_get_device_info() -> Option<String> {
    APP_STATE.with(|s| {
        s.borrow().as_ref().and_then(|st| st.inner.bluetooth.get_device_info_json())
    })
}

// ========== BLE Packet Processing ==========

// ========== Scramble Generator ==========

fn generate_scramble() -> String {
    let faces = ["U", "R", "F", "D", "L", "B"];
    let mods = ["", "'", "2"];
    let mut result = Vec::with_capacity(20);
    let mut last: Option<usize> = None;
    let mut second_last: Option<usize> = None;
    for _ in 0..20 {
        loop {
            let fi = (js_sys::Math::random() * 6.0) as usize % 6;
            if last == Some(fi) { continue; }
            // Prevent 3 consecutive on same axis (U/D=0/3, R/L=1/4, F/B=2/5)
            if let (Some(l), Some(sl)) = (last, second_last) {
                if fi % 3 == l % 3 && l % 3 == sl % 3 { continue; }
            }
            let mi = (js_sys::Math::random() * 3.0) as usize % 3;
            result.push(format!("{}{}", faces[fi], mods[mi]));
            second_last = last;
            last = Some(fi);
            break;
        }
    }
    result.join(" ")
}

/// Flow coordinator: after a move, check flow state and react accordingly.
fn flow_coordinate(st: &mut WasmAppState, timestamp: f64, actions: &mut Vec<String>, _notation: Option<&str>) {
    use rouxflow_core::session::FlowState;

    let flow = st.inner.session.get_flow_state_enum();

    match flow {
        FlowState::Idle => {
            // If cube is solved in Idle, auto-generate scramble (unless WCA session is full)
            if st.cube_logic.is_solved() && !st.inner.session.is_wca_full() {
                let scramble = generate_scramble();
                debug!("[flow] Idle -> Scrambling (auto, cube solved)");
                let action = st.inner.session.start_scramble(&scramble);
                if !action.is_empty() {
                    debug!("[calibration] started (new scramble)");
                    st.inner.calibrator.start();
                    st.debug_home_logged = false;
                    // Telemetry: start recording scramble phase
                    st.telemetry = Some(SolveTelemetry {
                        scramble: scramble.clone(),
                        scramble_gyro: Vec::new(),
                        solve_gyro: Vec::new(),
                        solve_moves: Vec::new(),
                        scramble_start_t: timestamp,
                        solve_start_t: 0.0,
                        solve_end_t: 0.0,
                    });
                    st.telemetry_recording = true;
                    st.telemetry_phase = 1;
                    actions.push(action);
                }
            }
        }
        FlowState::Scrambling => {
            // If scramble was invalidated, reset to Idle so user can start over
            if st.inner.session.is_scramble_invalid() {
                debug!("[flow] Scrambling -> Idle (scramble invalidated)");
                st.telemetry = None;
                st.telemetry_recording = false;
                st.telemetry_phase = 0;
                let action = st.inner.session.reset_flow();
                if !action.is_empty() { actions.push(action); }
            }
        }
        FlowState::Inspection => {
            // First move during inspection → start solving
            debug!("[flow] Inspection -> Solving (first move)");
            // Telemetry: switch to solve phase
            if let Some(ref mut tel) = st.telemetry {
                tel.solve_start_t = timestamp;
            }
            st.telemetry_phase = 2;
            let action = st.inner.start_solving(timestamp);
            if !action.is_empty() { actions.push(action); }
        }
        FlowState::Solving => {
            // Check if cube is solved → auto-complete
            if st.cube_logic.is_solved() {
                let time_ms = st.inner.timer.get_current_time_ms() as u32;
                debug!("[flow] Solving -> complete! time={}ms", time_ms);
                let moves_json = st.inner.timer.get_moves_json();

                if st.inner.session.requires_putdown_confirm() {
                    // WCA+gyro: hold solve, wait for putdown to confirm
                    debug!("[flow] WCA+gyro: holding solve, waiting for putdown");
                    // Telemetry: record solve end, stop recording
                    if let Some(ref mut tel) = st.telemetry {
                        tel.solve_end_t = timestamp;
                    }
                    st.telemetry_recording = false;
                    st.inner.hold_solve(timestamp, &moves_json);
                    let action = serde_json::to_string(
                        &CoreAction::FlowStateChanged(rouxflow_core::session::FlowState::Summary)
                    ).unwrap_or_default();
                    if !action.is_empty() { actions.push(action); }
                    // Don't chain — putdown handler will chain to next scramble
                } else {
                    // Standard flow: record immediately and chain
                    // Telemetry: record solve end, stop recording
                    if let Some(ref mut tel) = st.telemetry {
                        tel.solve_end_t = timestamp;
                    }
                    st.telemetry_recording = false;
                    let action = st.inner.record_solve(timestamp, time_ms, &moves_json);
                    if !action.is_empty() { actions.push(action); }
                    // Skip Summary: immediately start next scramble (unless WCA session is full)
                    if !st.inner.session.is_wca_full() {
                        let next = generate_scramble();
                        debug!("[flow] chaining to next scramble");
                        let action2 = st.inner.session.start_scramble(&next);
                        if !action2.is_empty() {
                            debug!("[calibration] started (next scramble)");
                            st.inner.calibrator.start();
                            actions.push(action2);
                        }
                    } else {
                        debug!("[flow] WCA session full — not generating next scramble");
                        let action2 = st.inner.session.reset_flow();
                        if !action2.is_empty() { actions.push(action2); }
                    }
                }
            }
        }
        FlowState::Summary => {
            // WCA+gyro: don't chain while waiting for putdown to confirm solve
            if st.inner.session.has_pending_solve() {
                return;
            }
            if !st.inner.session.is_wca_full() {
                // Fallback: if somehow in Summary, auto-chain to next scramble
                debug!("[flow] Summary -> Scrambling (fallback chain)");
                let next = generate_scramble();
                let action = st.inner.session.start_scramble(&next);
                if !action.is_empty() {
                    debug!("[calibration] started (summary chain)");
                    st.inner.calibrator.start();
                    actions.push(action);
                }
            } else {
                let action = st.inner.session.reset_flow();
                if !action.is_empty() { actions.push(action); }
            }
        }
    }
}

/// Process a raw BLE packet through the protocol handler and update state.
/// Returns JSON array of CoreActions.
#[wasm_bindgen]
pub fn cm_process_ble_packet(raw_data: &[u8], timestamp: f64) -> String {
    enter_wasm("cm_process_ble_packet");
    let result = APP_STATE.with(|s| {
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

        // Store decrypted hex + record timestamp for interval analysis
        if events.iter().any(|e| matches!(e, CubeEvent::Gyro { .. })) {
            st.last_gyro_hex = decrypted.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ");
            if st.gyro_timestamps.len() >= 500 {
                st.gyro_timestamps.remove(0);
            }
            st.gyro_timestamps.push(timestamp);
        }

        let wall_ms = timestamp * 1000.0;
        let mut actions: Vec<String> = Vec::new();
        let mut latest_gyro_q: Option<rouxflow_core::cube::Quaternion> = None;

        // ===== Feed phase: push events into interpreter =====
        for event in &events {
            match event {
                CubeEvent::Move { face, direction, .. } => {
                    let notation = CubeMove { face: *face, amount: *direction }.notation();
                    if let Some(ref gq) = st.debug_last_gyro {
                        debug!("[raw-move] {} | gyro=({:.4}, {:.4}, {:.4}, {:.4})",
                            notation, gq.x, gq.y, gq.z, gq.w);
                    } else {
                        debug!("[raw-move] {} | no gyro yet", notation);
                    }
                    // Record raw BLE face move for telemetry
                    if st.telemetry_recording && st.telemetry_phase == 2 {
                        if let Some(ref mut tel) = st.telemetry {
                            tel.solve_moves.push(RawMove {
                                n: notation.clone(),
                                t: timestamp,
                                k: rouxflow_core::move_interpreter::MoveKind::Face,
                            });
                        }
                    }
                    st.inner.interpreter.feed_face_move(*face, *direction, wall_ms);
                }
                CubeEvent::Gyro { quaternion, .. } => {
                    // Update cube_logic orientation for the 3D renderer
                    let q = rouxflow_core::cube::Quaternion {
                        x: quaternion.x,
                        y: quaternion.y,
                        z: quaternion.z,
                        w: quaternion.w,
                    };
                    st.cube_logic.orientation = Some(q);

                    // Record raw gyro for telemetry
                    if st.telemetry_recording {
                        if let Some(ref mut tel) = st.telemetry {
                            let sample = GyroSample { t: timestamp, x: quaternion.x, y: quaternion.y, z: quaternion.z, w: quaternion.w };
                            match st.telemetry_phase {
                                1 => tel.scramble_gyro.push(sample),
                                2 => tel.solve_gyro.push(sample),
                                _ => {}
                            }
                        }
                    }

                    // Process orientation for pickup/putdown detection
                    let action = st.inner.session.process_orientation(
                        quaternion.x,
                        quaternion.y,
                        quaternion.z,
                        quaternion.w,
                        timestamp,
                    );
                    if !action.is_empty() {
                        if action.contains("Pickup") {
                            debug!("[gyro] pickup detected | last_gyro_raw={}", st.last_gyro_hex);
                        } else if action.contains("Putdown") {
                            let (putdown_t, last_pickup, solve_pickup, stable_since) =
                                st.inner.session.debug_putdown_timing();
                            let pickup_ref = solve_pickup.or(last_pickup).unwrap_or(0.0);
                            let calc_time_ms = if pickup_ref > 0.0 { (putdown_t - pickup_ref) * 1000.0 } else { 0.0 };
                            debug!(
                                "[gyro] putdown detected | stable_since={:.3}s last_putdown={:.3}s pickup={:.3}s solve_pickup={:?} calc_time={:.0}ms now={:.3}s",
                                stable_since, putdown_t, last_pickup.unwrap_or(0.0), solve_pickup, calc_time_ms, timestamp
                            );
                            // Dump gyro debug buffer (deltas from solve complete to putdown)
                            let gyro_dbg = st.inner.session.drain_gyro_debug();
                            if !gyro_dbg.is_empty() {
                                let first_ts = gyro_dbg[0].0;
                                let lines: Vec<String> = gyro_dbg.iter().map(|(ts, delta, dx, dy, dz, dw)| {
                                    format!("+{:.0}ms d={:.10} | dx={:.7} dy={:.7} dz={:.7} dw={:.7}",
                                        (ts - first_ts) * 1000.0, delta, dx, dy, dz, dw)
                                }).collect();
                                debug!("[gyro-debug] {} samples from solve→putdown:\n{}", lines.len(), lines.join("\n"));
                            }

                            // WCA+gyro: if a solve is pending putdown, confirm it now
                            if st.inner.session.has_pending_solve() {
                                debug!("[flow] putdown confirms pending solve");
                                let save_action = st.inner.session.confirm_pending_solve();
                                if !save_action.is_empty() {
                                    actions.push(save_action);
                                }
                                // Chain to next scramble (unless WCA full)
                                if !st.inner.session.is_wca_full() {
                                    let next = generate_scramble();
                                    debug!("[flow] chaining to next scramble after putdown");
                                    let action2 = st.inner.session.start_scramble(&next);
                                    if !action2.is_empty() {
                                        debug!("[calibration] started (putdown chain)");
                                        st.inner.calibrator.start();
                                        actions.push(action2);
                                    }
                                } else {
                                    debug!("[flow] WCA session full after putdown confirm");
                                    let action2 = st.inner.session.reset_flow();
                                    if !action2.is_empty() { actions.push(action2); }
                                }
                            }
                        }
                        actions.push(action);
                    }

                    // Feed gyro to interpreter for rotation detection
                    st.inner.interpreter.feed_gyro(&q, wall_ms);

                    // Feed gyro to calibrator if active
                    if st.inner.calibrator.is_active() {
                        st.inner.calibrator.feed(&q);
                    }

                    // Debug: log "after" gyro for pending moves
                    if !st.debug_moves_pending_gyro.is_empty() {
                        for mv in st.debug_moves_pending_gyro.drain(..) {
                            debug!("[move-gyro] {} AFTER  gyro=({:.4}, {:.4}, {:.4}, {:.4})",
                                mv, q.x, q.y, q.z, q.w);
                        }
                    }
                    st.debug_last_gyro = Some(q);

                    // Store latest gyro for zone tracking after dispatch
                    // (must happen after dispatch so compensate_slice is applied first)
                    latest_gyro_q = Some(q);
                }
                CubeEvent::Battery { level } => {
                    st.inner.bluetooth.update_battery(*level);
                    actions.push(format!(r#"{{"type":"Battery","data":{}}}"#, level));
                }
                CubeEvent::Hardware { name, sw_version, hw_version, gyro_supported } => {
                    st.inner.bluetooth.update_hardware(sw_version.clone(), hw_version.clone());
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
                    }
                    actions.push(format!(r#"{{"type":"RawFacelets","data":"{}"}}"#, facelet_string));
                }
                _ => {
                    let action = handle_cube_event(event, &mut st.inner.session);
                    if !action.is_empty() {
                        actions.push(action);
                    }
                }
            }
        }

        // ===== Flush phase: set zone rotation hint, then get interpreted moves =====
        let has_zone_rotation = st.inner.calibrator.has_pending_zone_rotation();
        st.inner.interpreter.set_zone_rotation_hint(has_zone_rotation);
        let solve_start_ms = st.inner.timer.start_time_ms();
        let interpreted = st.inner.interpreter.flush(wall_ms, solve_start_ms);

        // ===== Dispatch phase: process each interpreted move =====
        for imove in &interpreted {
            // Remap notation from body frame to home frame using gyro zone state.
            // Raw face moves (for cube_logic and scramble validation) stay in body frame.
            let remapped = st.inner.calibrator.remap_notation(&imove.notation);
            let was_remapped = remapped != imove.notation;

            // Log interpreted move (compact: notation + kind, show remap if different)
            if was_remapped {
                debug!("[move] {} -> {} ({:?})", imove.notation, remapped, imove.kind);
            } else {
                debug!("[move] {} ({:?})", imove.notation, imove.kind);
            }

            // Debug: log gyro before each move, home on first move
            if let Some(ref gq) = st.debug_last_gyro {
                if !st.debug_home_logged {
                    if let Some(home) = st.inner.calibrator.home() {
                        debug!("[move-gyro] HOME gyro=({:.4}, {:.4}, {:.4}, {:.4})",
                            home.x, home.y, home.z, home.w);
                    }
                    st.debug_home_logged = true;
                }
                debug!("[move-gyro] {} BEFORE gyro=({:.4}, {:.4}, {:.4}, {:.4}) | {:?}{}",
                    imove.notation, gq.x, gq.y, gq.z, gq.w, imove.kind,
                    if was_remapped { format!(" -> {}", remapped) } else { String::new() });
                st.debug_moves_pending_gyro.push(remapped.clone());
            }

            // Apply raw face moves to cube_logic (correct facelet state — always body frame)
            for &(face, dir) in &imove.raw_face_moves {
                let notation = CubeMove { face, amount: dir }.notation();
                st.cube_logic.apply_move(&notation);
            }

            // Record interpreted move in timer with remapped notation
            let mut remapped_move = imove.clone();
            remapped_move.notation = remapped.clone();
            st.inner.record_interpreted_move(&remapped_move);

            // Animate remapped notation — skip Rotation and Wide moves since
            // the gyro already drives the renderer orientation continuously.
            // Face and Slice moves are safe: body doesn't rotate, only layers turn.
            match imove.kind {
                rouxflow_core::move_interpreter::MoveKind::Face
                | rouxflow_core::move_interpreter::MoveKind::Slice => {
                    rouxflow_render::queue_move_anim(remapped.clone(), 0.15);
                }
                _ => {} // Rotation/Wide: gyro + facelet update handles the visual
            }

            // Slice compensation: the gyro sensor is in the core, so M/E/S slices
            // rotate the sensor without rotating the shell. Compensate the calibrator's
            // core offset so zone tracking stays aligned with the user's shell orientation.
            // Use the RAW (body-frame) notation since that's what the hardware produced.
            if imove.kind == rouxflow_core::move_interpreter::MoveKind::Slice {
                st.inner.calibrator.compensate_slice(&imove.notation);
            }

            // Scramble validation: feed raw face moves (scrambles = body frame, always)
            let flow_before = st.inner.session.get_flow_state_enum();
            for &(face, dir) in &imove.raw_face_moves {
                let notation = CubeMove { face, amount: dir }.notation();
                let a = st.inner.session.handle_scramble_move(&notation, timestamp);
                if !a.is_empty() { actions.push(a); }
            }

            // Check for Scrambling → Inspection transition: finalize gyro calibration
            let flow_after = st.inner.session.get_flow_state_enum();
            if flow_before != flow_after {
                debug!("[flow] {:?} -> {:?}", flow_before, flow_after);
            }
            if flow_before == rouxflow_core::session::FlowState::Scrambling
                && flow_after == rouxflow_core::session::FlowState::Inspection
            {
                // Telemetry: transition from scramble to solve phase
                st.telemetry_phase = 2;

                match st.inner.calibrator.finalize() {
                    Some(_home) => {
                        debug!("[calibration] finalized ({} samples)", st.inner.calibrator.sample_count());
                        if let Some((ox, oy, oz, ow)) = st.inner.calibrator.compute_render_offset() {
                            rouxflow_render::set_gyro_offset(ox, oy, oz, ow);
                        }
                    }
                    None => {
                        debug!("[calibration] finalize failed (< 10 samples)");
                    }
                }
            }

            // Emit CoreAction::Move with remapped notation
            let action_json = serde_json::to_string(&CoreAction::Move(remapped.clone()))
                .unwrap_or_default();
            if !action_json.is_empty() {
                actions.push(action_json);
            }

            // Flow coordination: only for face/slice moves, not rotations
            // (rotations during inspection should not start the solve timer)
            if imove.kind != rouxflow_core::move_interpreter::MoveKind::Rotation {
                if flow_before == flow_after {
                    flow_coordinate(st, timestamp, &mut actions, Some(&remapped));
                }
            }
        }

        // Consume pending zone rotations if a standalone gyro rotation was emitted
        if interpreted.iter().any(|m| m.kind == rouxflow_core::move_interpreter::MoveKind::Rotation) {
            // Deprecated: zone tracking removed
        }

        // Zone tracking: runs AFTER dispatch so compensate_slice has been applied.
        // This ensures zones see the correct core_offset (no one-packet lag).
        if let Some(ref _gq) = latest_gyro_q {
            if !st.inner.calibrator.is_active() && st.inner.calibrator.home().is_some() {
                // Deprecated: zone tracking removed
            }
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
    APP_STATE.with(|s| {
        let state = s.borrow();
        let st = state.as_ref().ok_or_else(|| JsValue::from_str("AppState not initialized"))?;
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
    APP_STATE.with(|s| {
        s.borrow().as_ref().map_or_else(
            || "{}".to_string(),
            |st| {
                let facelets = st.cube_logic.facelets();
                format!(r#"{{"facelets":{:?}}}"#, facelets)
            },
        )
    })
}

#[wasm_bindgen]
pub fn cm_get_orientation() -> String {
    APP_STATE.with(|s| {
        s.borrow().as_ref().map_or_else(
            || "[0,0,0,1]".to_string(),
            |st| {
                if let Some(q) = st.cube_logic.orientation {
                    format!("[{},{},{},{}]", q.x, q.y, q.z, q.w)
                } else {
                    "[0,0,0,1]".to_string()
                }
            },
        )
    })
}

#[wasm_bindgen]
pub fn cm_get_orientation_debug() -> String {
    APP_STATE.with(|s| {
        s.borrow().as_ref().map_or_else(
            || r#"{"raw":[0,0,0,1],"home":null,"shell":[0,0,0,1]}"#.to_string(),
            |st| {
                if let Some(q) = st.cube_logic.orientation {
                    let home_str = st.inner.calibrator.home().map_or("null".to_string(), |h| format!("[{},{},{},{}]", h.x, h.y, h.z, h.w));
                    let mut pos_str = "\"Unknown\"".to_string();
                    let mut shell_str = format!("[{},{},{},{}]", q.x, q.y, q.z, q.w);
                    
                    if let Some(h) = st.inner.calibrator.home() {
                        let q_rel_shell = rouxflow_core::gyro_snap::AbsoluteStateTracker::compute_rel_shell(h, &q);
                        shell_str = format!("[{},{},{},{}]", q_rel_shell.x, q_rel_shell.y, q_rel_shell.z, q_rel_shell.w);
                        
                        if let Some((_idx, cp)) = rouxflow_core::gyro_snap::AbsoluteStateTracker::get_nearest_posture(&q_rel_shell) {
                            pos_str = format!(r#"{{ "top": "{:?}", "front": "{:?}" }}"#, cp.top, cp.front);
                        }
                    }

                    format!(
                        r#"{{"raw":[{},{},{},{}],"home":{},"shell":{},"posture":{}}}"#,
                        q.x, q.y, q.z, q.w, home_str, shell_str, pos_str
                    )
                } else {
                    r#"{"raw":[0,0,0,1],"home":null,"shell":[0,0,0,1],"posture":"Unknown"}"#.to_string()
                }
            },
        )
    })
}

#[wasm_bindgen]
pub fn cm_force_home() {
    APP_STATE.with(|s| {
        if let Some(st) = s.borrow_mut().as_mut() {
            if let Some(q) = st.cube_logic.orientation {
                st.inner.calibrator.start();
                for _ in 0..15 {
                    st.inner.calibrator.feed(&q);
                }
                if let Some(_) = st.inner.calibrator.finalize() {
                    if let Some((ox, oy, oz, ow)) = st.inner.calibrator.compute_render_offset() {
                        rouxflow_render::set_gyro_offset(ox, oy, oz, ow);
                    }
                }
            }
        }
    });
}

#[wasm_bindgen]
pub fn cm_get_facelets() -> String {
    APP_STATE.with(|s| {
        s.borrow().as_ref().map_or_else(
            || "[]".to_string(),
            |st| {
                let f = st.cube_logic.facelets();
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
    APP_STATE.with(|s| {
        if let Some(st) = s.borrow_mut().as_mut() {
            st.inner.timer.start(timestamp);
        }
    });
    exit_wasm();
}

#[wasm_bindgen]
pub fn cm_stop_timer(timestamp: f64) {
    enter_wasm("cm_stop_timer");
    APP_STATE.with(|s| {
        if let Some(st) = s.borrow_mut().as_mut() {
            st.inner.timer.stop(timestamp);
        }
    });
    exit_wasm();
}

#[wasm_bindgen]
pub fn cm_update_timer(timestamp: f64) -> String {
    APP_STATE.with(|s| {
        let mut state = s.borrow_mut();
        let st = match state.as_mut() {
            Some(st) => st,
            None => return String::new(),
        };
        st.inner.timer.update(timestamp);

        // Check scramble move timeout → invalidate (stays visible until next BLE move resets flow)
        if st.inner.session.get_flow_state_enum() == rouxflow_core::session::FlowState::Scrambling {
            if st.inner.session.check_scramble_timeout(timestamp) {
                debug!("[flow] Scramble move timeout -> invalidated");
            }
        }

        // Check inspection timeout → DNF
        if st.inner.session.get_flow_state_enum() == rouxflow_core::session::FlowState::Inspection {
            if st.inner.session.is_inspection_expired(timestamp) {
                debug!("[flow] Inspection expired -> DNF");
                // Telemetry: discard on DNF
                st.telemetry = None;
                st.telemetry_recording = false;
                st.telemetry_phase = 0;
                let mut actions = Vec::new();
                // Record DNF solve
                let action = st.inner.record_dnf(timestamp);
                if !action.is_empty() { actions.push(action); }
                // Chain to next scramble (unless WCA full)
                if !st.inner.session.is_wca_full() {
                    let next = generate_scramble();
                    let action2 = st.inner.session.start_scramble(&next);
                    if !action2.is_empty() {
                        debug!("[calibration] started (DNF chain)");
                        st.inner.calibrator.start();
                        actions.push(action2);
                    }
                } else {
                    debug!("[flow] WCA session full after DNF — not generating next scramble");
                    let action2 = st.inner.session.reset_flow();
                    if !action2.is_empty() { actions.push(action2); }
                }
                if !actions.is_empty() {
                    return if actions.len() == 1 {
                        actions.into_iter().next().unwrap()
                    } else {
                        format!("[{}]", actions.join(","))
                    };
                }
            }
        }

        String::new()
    })
}

#[wasm_bindgen]
pub fn cm_get_timer_state() -> String {
    APP_STATE.with(|s| {
        s.borrow().as_ref().map_or_else(|| "{}".to_string(), |st| st.inner.timer.get_state_json())
    })
}

#[wasm_bindgen]
pub fn cm_is_timer_running() -> bool {
    APP_STATE.with(|s| {
        s.borrow().as_ref().map_or(false, |st| st.inner.timer.is_running())
    })
}

#[wasm_bindgen]
pub fn cm_get_current_time_ms() -> u64 {
    APP_STATE.with(|s| {
        s.borrow().as_ref().map_or(0, |st| st.inner.timer.get_current_time_ms())
    })
}

// ========== Session Management ==========

#[wasm_bindgen]
pub fn cm_get_flow_state() -> String {
    APP_STATE.with(|s| {
        s.borrow().as_ref().map_or_else(|| "\"Idle\"".to_string(), |st| st.inner.session.get_flow_state())
    })
}

#[wasm_bindgen]
pub fn cm_set_active_session(session_json: &str) {
    APP_STATE.with(|s| {
        if let Some(st) = s.borrow_mut().as_mut() {
            st.inner.session.set_active_session(session_json);
        }
    });
}

#[wasm_bindgen]
pub fn cm_create_session(session_json: &str) {
    APP_STATE.with(|s| {
        if let Some(st) = s.borrow_mut().as_mut() {
            st.inner.session.set_active_session(session_json);
        }
    });
}

#[wasm_bindgen]
pub fn cm_start_scramble(scramble: &str) -> String {
    APP_STATE.with(|s| {
        s.borrow_mut().as_mut().map_or_else(String::new, |st| {
            debug!("[flow] cm_start_scramble called");
            debug!("[calibration] started (cm_start_scramble)");
            st.inner.calibrator.start();
            let action = st.inner.session.start_scramble(scramble);

            let now = js_sys::Date::now() / 1000.0;
            st.telemetry = Some(SolveTelemetry {
                scramble: scramble.to_string(),
                scramble_gyro: Vec::new(),
                solve_gyro: Vec::new(),
                solve_moves: Vec::new(),
                scramble_start_t: now,
                solve_start_t: 0.0,
                solve_end_t: 0.0,
            });
            st.telemetry_recording = true;
            st.telemetry_phase = 1;

            action
        })
    })
}

#[wasm_bindgen]
pub fn cm_handle_scramble_move(move_str: &str) -> String {
    let timestamp = js_sys::Date::now() / 1000.0;
    APP_STATE.with(|s| {
        s.borrow_mut().as_mut().map_or_else(String::new, |st| {
            st.inner.session.handle_scramble_move(move_str, timestamp)
        })
    })
}

#[wasm_bindgen]
pub fn cm_set_solving() -> String {
    let timestamp = js_sys::Date::now() / 1000.0;
    APP_STATE.with(|s| {
        s.borrow_mut().as_mut().map_or_else(String::new, |st| st.inner.start_solving(timestamp))
    })
}

#[wasm_bindgen]
pub fn cm_record_solve(time_ms: u32, moves_json: &str) -> String {
    let timestamp = js_sys::Date::now() / 1000.0;
    APP_STATE.with(|s| {
        s.borrow_mut().as_mut().map_or_else(String::new, |st| {
            st.inner.record_solve(timestamp, time_ms, moves_json)
        })
    })
}

// ========== Session Stats Queries ==========

#[wasm_bindgen]
pub fn cm_get_session_stats_json() -> String {
    APP_STATE.with(|s| {
        s.borrow().as_ref().map_or_else(|| "{}".to_string(), |st| st.inner.session.get_session_stats_json())
    })
}

#[wasm_bindgen]
pub fn cm_get_solve_list_json() -> String {
    APP_STATE.with(|s| {
        s.borrow().as_ref().map_or_else(|| "[]".to_string(), |st| st.inner.session.get_solve_list_json())
    })
}

#[wasm_bindgen]
pub fn cm_get_solve_by_id_json(solve_id: &str) -> String {
    APP_STATE.with(|s| {
        s.borrow().as_ref().map_or_else(|| "null".to_string(), |st| st.inner.session.get_solve_by_id_json(solve_id))
    })
}

// ========== Soft-Delete Solve ==========

#[wasm_bindgen]
pub fn cm_delete_solve(solve_id: &str) -> String {
    APP_STATE.with(|s| {
        let mut state = s.borrow_mut();
        match state.as_mut() {
            Some(st) => {
                let result = st.inner.session.delete_solve(solve_id);
                result
            }
            None => serde_json::to_string(&rouxflow_core::session::CoreAction::Error(
                "AppState not initialized".into()
            )).unwrap(),
        }
    })
}

// ========== Flow + Scramble Queries ==========

#[wasm_bindgen]
pub fn cm_is_wca_session_full() -> bool {
    APP_STATE.with(|s| {
        s.borrow().as_ref().map_or(false, |st| st.inner.session.is_wca_full())
    })
}

#[wasm_bindgen]
pub fn cm_is_cube_solved() -> bool {
    APP_STATE.with(|s| {
        s.borrow().as_ref().map_or(true, |st| st.cube_logic.is_solved())
    })
}

#[wasm_bindgen]
pub fn cm_reset_flow() -> String {
    APP_STATE.with(|s| {
        s.borrow_mut().as_mut().map_or_else(String::new, |st| {
            debug!("[flow] cm_reset_flow called");
            // Telemetry: abort any in-progress recording
            st.telemetry = None;
            st.telemetry_recording = false;
            st.telemetry_phase = 0;
            let action = st.inner.session.reset_flow();
            // If cube is solved, auto-generate a scramble (unless WCA full)
            if st.cube_logic.is_solved() && !st.inner.session.is_wca_full() {
                let scramble = generate_scramble();
                let action2 = st.inner.session.start_scramble(&scramble);
                if !action2.is_empty() {
                    debug!("[calibration] started (reset_flow + solved)");
                    st.inner.calibrator.start();
                    
                    let now = js_sys::Date::now() / 1000.0;
                    st.telemetry = Some(SolveTelemetry {
                        scramble: scramble.clone(),
                        scramble_gyro: Vec::new(),
                        solve_gyro: Vec::new(),
                        solve_moves: Vec::new(),
                        scramble_start_t: now,
                        solve_start_t: 0.0,
                        solve_end_t: 0.0,
                    });
                    st.telemetry_recording = true;
                    st.telemetry_phase = 1;

                    return action2;
                }
            }
            action
        })
    })
}

#[wasm_bindgen]
pub fn cm_get_scramble_state(now: f64) -> String {
    APP_STATE.with(|s| {
        s.borrow().as_ref().map_or_else(|| "{}".to_string(), |st| st.inner.session.get_scramble_state_json(now))
    })
}

#[wasm_bindgen]
pub fn cm_get_inspection_remaining(now: f64) -> f64 {
    APP_STATE.with(|s| {
        s.borrow().as_ref().map_or(0.0, |st| st.inner.session.get_inspection_remaining(now))
    })
}

#[wasm_bindgen]
pub fn cm_set_inspection_duration(seconds: f64) {
    APP_STATE.with(|s| {
        if let Some(st) = s.borrow_mut().as_mut() {
            st.inner.session.set_inspection_duration(seconds);
        }
    })
}

#[wasm_bindgen]
pub fn cm_get_inspection_duration() -> f64 {
    APP_STATE.with(|s| {
        s.borrow().as_ref().map_or(15.0, |st| st.inner.session.get_inspection_duration())
    })
}

/// Get pickup mode of the active session: "None", "Fixed", or "Gyro".
#[wasm_bindgen]
pub fn cm_get_pickup_mode() -> String {
    APP_STATE.with(|s| {
        s.borrow().as_ref().map_or_else(
            || "None".to_string(),
            |st| match st.inner.session.get_pickup_mode() {
                rouxflow_core::session::PickupMode::None => "None".to_string(),
                rouxflow_core::session::PickupMode::Fixed => "Fixed".to_string(),
                rouxflow_core::session::PickupMode::Gyro => "Gyro".to_string(),
            },
        )
    })
}

/// Returns true if the cube is detected as stable (on table).
#[wasm_bindgen]
pub fn cm_is_cube_stable() -> bool {
    APP_STATE.with(|s| {
        s.borrow().as_ref().map_or(true, |st| st.inner.session.is_cube_stable())
    })
}

/// Set pickup mode on the active session (Free mode only).
/// Accepts "None", "Fixed", or "Gyro".
#[wasm_bindgen]
pub fn cm_set_pickup_mode(mode: &str) {
    let pickup_mode = match mode {
        "Fixed" => rouxflow_core::session::PickupMode::Fixed,
        "Gyro" => rouxflow_core::session::PickupMode::Gyro,
        _ => rouxflow_core::session::PickupMode::None,
    };
    APP_STATE.with(|s| {
        if let Some(st) = s.borrow_mut().as_mut() {
            st.inner.session.set_pickup_mode(pickup_mode);
        }
    })
}

#[wasm_bindgen]
pub fn cm_generate_new_scramble() -> String {
    APP_STATE.with(|s| {
        s.borrow_mut().as_mut().map_or_else(String::new, |st| {
            if st.inner.session.is_wca_full() {
                debug!("[flow] cm_generate_new_scramble: WCA session full, skipping");
                return String::new();
            }
            let scramble = generate_scramble();
            debug!("[flow] cm_generate_new_scramble called");
            debug!("[calibration] started (new scramble generated)");
            st.inner.calibrator.start();
            st.inner.session.start_scramble(&scramble);

            let now = js_sys::Date::now() / 1000.0;
            st.telemetry = Some(SolveTelemetry {
                scramble: scramble.clone(),
                scramble_gyro: Vec::new(),
                solve_gyro: Vec::new(),
                solve_moves: Vec::new(),
                scramble_start_t: now,
                solve_start_t: 0.0,
                solve_end_t: 0.0,
            });
            st.telemetry_recording = true;
            st.telemetry_phase = 1;

            scramble
        })
    })
}

#[wasm_bindgen]
pub fn cm_get_pending_scramble() -> String {
    APP_STATE.with(|s| {
        s.borrow().as_ref().map_or_else(String::new, |st| {
            st.inner.session.get_pending_scramble().unwrap_or("").to_string()
        })
    })
}

/// Get gyro packet interval statistics as JSON.
/// Returns: { count, hz, min_ms, max_ms, median_ms, p5_ms, p95_ms, intervals: [...] }
#[wasm_bindgen]
pub fn cm_get_gyro_stats() -> String {
    APP_STATE.with(|s| {
        let state = s.borrow();
        let st = match state.as_ref() {
            Some(st) => st,
            None => return "{}".to_string(),
        };

        let ts = &st.gyro_timestamps;
        if ts.len() < 2 {
            return "{}".to_string();
        }

        // Compute all intervals in ms
        let mut intervals: Vec<f64> = Vec::with_capacity(ts.len() - 1);
        for i in 1..ts.len() {
            intervals.push((ts[i] - ts[i - 1]) * 1000.0);
        }
        intervals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let count = intervals.len();
        let total_ms: f64 = intervals.iter().sum();
        let avg_ms = total_ms / count as f64;
        let hz = 1000.0 / avg_ms;
        let min_ms = intervals[0];
        let max_ms = intervals[count - 1];
        let median_ms = intervals[count / 2];
        let p5_ms = intervals[(count as f64 * 0.05) as usize];
        let p95_ms = intervals[(count as f64 * 0.95).min((count - 1) as f64) as usize];

        // Build histogram buckets: 0-10, 10-20, 20-30, ..., 90-100, 100+
        let mut buckets = [0u32; 11];
        for &iv in &intervals {
            let idx = (iv / 10.0) as usize;
            if idx >= 10 { buckets[10] += 1; } else { buckets[idx] += 1; }
        }

        format!(
            r#"{{"count":{},"hz":{:.1},"avg_ms":{:.1},"min_ms":{:.1},"max_ms":{:.1},"median_ms":{:.1},"p5_ms":{:.1},"p95_ms":{:.1},"buckets":[{}],"bucket_labels":["0-10","10-20","20-30","30-40","40-50","50-60","60-70","70-80","80-90","90-100","100+"]}}"#,
            count, hz, avg_ms, min_ms, max_ms, median_ms, p5_ms, p95_ms,
            buckets.iter().map(|b| b.to_string()).collect::<Vec<_>>().join(","),
        )
    })
}

/// Reset gyro stats (clear recorded timestamps).
#[wasm_bindgen]
pub fn cm_reset_gyro_stats() {
    APP_STATE.with(|s| {
        if let Some(st) = s.borrow_mut().as_mut() {
            st.gyro_timestamps.clear();
        }
    })
}

// ========== MAC Address Validation ==========

#[wasm_bindgen]
pub fn cm_needs_mac_input(device_id: &str, protocol: &str) -> bool {
    rouxflow_core::BluetoothManager::needs_mac_input(device_id, protocol)
}

#[wasm_bindgen]
pub fn cm_requires_handshake() -> bool {
    APP_STATE.with(|s| {
        s.borrow().as_ref()
            .and_then(|st| st.protocol.as_ref())
            .map_or(false, |p| p.requires_handshake())
    })
}

#[wasm_bindgen]
pub fn cm_handshake_data() -> Vec<u8> {
    APP_STATE.with(|s| {
        s.borrow().as_ref()
            .and_then(|st| st.protocol.as_ref())
            .and_then(|p| p.handshake_data())
            .unwrap_or_default()
    })
}

// ========== Debug: gyro raw hex ==========

#[wasm_bindgen]
pub fn cm_get_last_gyro_hex() -> String {
    APP_STATE.with(|s| {
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

// ========== Telemetry: drain raw solve data ==========

/// Drain the completed solve telemetry as JSON. Returns "null" if none available.
/// Call this after a SaveSolve action to retrieve the raw data.
#[wasm_bindgen]
pub fn cm_drain_solve_telemetry() -> String {
    APP_STATE.with(|s| {
        let mut state = s.borrow_mut();
        match state.as_mut() {
            Some(st) => match st.telemetry.take() {
                Some(t) => serde_json::to_string(&t).unwrap_or_else(|_| "null".into()),
                None => "null".into(),
            },
            None => "null".into(),
        }
    })
}

// ========== Debug: decrypt/encrypt hex strings ==========

#[wasm_bindgen]
pub fn cm_decrypt_hex(hex_input: &str) -> String {
    APP_STATE.with(|s| {
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

#[wasm_bindgen]
pub fn cm_encrypt_hex(hex_input: &str) -> String {
    APP_STATE.with(|s| {
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
