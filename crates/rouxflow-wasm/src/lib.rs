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
        // Configure interpreter for this cube
        st.inner.interpreter.set_has_gyro(has_gyro);
        st.inner.interpreter.reset();

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
                    actions.push(action);
                }
            }
        }
        FlowState::Scrambling => {
            // If scramble was invalidated, reset to Idle so user can start over
            if st.inner.session.is_scramble_invalid() {
                debug!("[flow] Scrambling -> Idle (scramble invalidated)");
                let action = st.inner.session.reset_flow();
                if !action.is_empty() { actions.push(action); }
            }
        }
        FlowState::Inspection => {
            // First move during inspection → start solving
            debug!("[flow] Inspection -> Solving (first move)");
            let action = st.inner.start_solving(timestamp);
            if !action.is_empty() { actions.push(action); }
        }
        FlowState::Solving => {
            // Check if cube is solved → auto-complete + immediately chain to next scramble
            if st.cube_logic.is_solved() {
                let time_ms = st.inner.timer.get_current_time_ms() as u32;
                debug!("[flow] Solving -> complete! time={}ms", time_ms);
                let moves_json = st.inner.timer.get_moves_json();
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
        FlowState::Summary => {
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

        // Store decrypted hex if any event is Gyro (for gyroDebug.show())
        if events.iter().any(|e| matches!(e, CubeEvent::Gyro { .. })) {
            st.last_gyro_hex = decrypted.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ");
        }

        let wall_ms = timestamp * 1000.0;
        let mut actions: Vec<String> = Vec::new();

        // ===== Feed phase: push events into interpreter =====
        for event in &events {
            match event {
                CubeEvent::Move { face, direction, .. } => {
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
                            debug!("[gyro] pickup detected");
                        } else if action.contains("Putdown") {
                            debug!("[gyro] putdown detected");
                        }
                        actions.push(action);
                    }

                    // Feed gyro to interpreter for rotation detection
                    st.inner.interpreter.feed_gyro(&q, wall_ms);

                    // Feed gyro to calibrator if active
                    if st.inner.calibrator.is_active() {
                        st.inner.calibrator.feed(&q);
                    }

                    // Zone tracking: log orientation changes during inspection/solving
                    if !st.inner.calibrator.is_active() && st.inner.calibrator.home().is_some() {
                        let zone_logs = st.inner.calibrator.track_orientation(&q);
                        for log_msg in &zone_logs {
                            debug!("{}", log_msg);
                        }
                    }
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

            // Log interpreted move (show remapped if different)
            if was_remapped {
                debug!("[move] {} -> {} kind={:?} raw={:?} gyro_delta={:?}",
                    imove.notation, remapped, imove.kind,
                    imove.raw_face_moves.iter().map(|(f,d)| CubeMove{face:*f, amount:*d}.notation()).collect::<Vec<_>>(),
                    imove.gyro_delta);
            } else {
                debug!("[move] {} kind={:?} raw={:?} gyro_delta={:?}",
                    imove.notation, imove.kind,
                    imove.raw_face_moves.iter().map(|(f,d)| CubeMove{face:*f, amount:*d}.notation()).collect::<Vec<_>>(),
                    imove.gyro_delta);
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
                // Update renderer offset to account for new core offset
                if let Some((ox, oy, oz, ow)) = st.inner.calibrator.compute_render_offset_compensated() {
                    rouxflow_render::set_gyro_offset(ox, oy, oz, ow);
                }
                debug!("[slice-compensate] {} -> core_offset updated, zones: {}",
                    imove.notation, st.inner.calibrator.debug_zones());
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
                match st.inner.calibrator.finalize() {
                    Some(home) => {
                        debug!("[calibration] finalized home=({:.4}, {:.4}, {:.4}, {:.4}) samples={}",
                            home.x, home.y, home.z, home.w, st.inner.calibrator.sample_count());
                        if let Some(axes_str) = st.inner.calibrator.debug_home_axes() {
                            debug!("[calibration] home axes: {}", axes_str);
                        }
                        debug!("[calibration] initial zones: {}", st.inner.calibrator.debug_zones());
                        // Use compensated offset (accounts for any accumulated core offset,
                        // though it should be identity right after finalize)
                        if let Some((ox, oy, oz, ow)) = st.inner.calibrator.compute_render_offset_compensated() {
                            debug!("[calibration] applying gyro offset=({:.4}, {:.4}, {:.4}, {:.4})",
                                ox, oy, oz, ow);
                            rouxflow_render::set_gyro_offset(ox, oy, oz, ow);
                        }
                    }
                    None => {
                        debug!("[calibration] finalize failed (not enough samples)");
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
            st.inner.calibrator.consume_zone_rotations();
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
            st.inner.session.start_scramble(scramble)
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
            let action = st.inner.session.reset_flow();
            // If cube is solved, auto-generate a scramble (unless WCA full)
            if st.cube_logic.is_solved() && !st.inner.session.is_wca_full() {
                let scramble = generate_scramble();
                let action2 = st.inner.session.start_scramble(&scramble);
                if !action2.is_empty() {
                    debug!("[calibration] started (reset_flow + solved)");
                    st.inner.calibrator.start();
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
