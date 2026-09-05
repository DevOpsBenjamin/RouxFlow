//! Single WASM entry point for RouxFlow.
//!
//! Re-exports from: rouxflow-core, rouxflow-render, rouxflow-bluetoothcube, rouxflow-storage.
//! This is the only crate compiled as `cdylib` for wasm-pack.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use log::debug;
use rouxflow_bluetoothcube::codec::{self, CubeCommand, CubeEvent, CubeProtocol};
// ========== INIT ==========

#[wasm_bindgen(start)]
pub fn wasm_init() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).ok();
}

/// Serialize a list of action JSON strings into a single response.
fn format_actions(actions: &[String]) -> String {
    if actions.is_empty() {
        String::new()
    } else if actions.len() == 1 {
        actions[0].clone()
    } else {
        format!("[{}]", actions.join(","))
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
        *s.borrow_mut() = Some(WasmAppState {
            inner: rouxflow_core::AppState::new(),
            protocol: None,
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

/// Re-persist the active session to IndexedDB (after settings change).
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
        st.inner.cube_logic = rouxflow_core::cube::CubeState::new();
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

/// Process a raw BLE packet through the protocol handler and update state.
/// Returns JSON array of CoreActions.
#[wasm_bindgen]
pub fn cm_process_ble_packet(raw_data: &[u8], timestamp: f64) -> String {
    enter_wasm("cm_process_ble_packet");
    let result = APP_STATE.with(|s| {
        let mut state = s.borrow_mut();
        let st = match state.as_mut() { Some(st) => st, None => return String::new() };
        let protocol = match st.protocol.as_mut() { Some(p) => p, None => return String::new() };

        let decrypted = protocol.decrypt(raw_data);
        let events = protocol.decode_event(&decrypted);

        let wall_ms = timestamp * 1000.0;
        let mut actions: Vec<String> = Vec::new();

        // Feed events into core
        for event in &events {
            match event {
                CubeEvent::Move { face, direction, .. } => {
                    st.inner.feed_ble_move(*face, *direction, timestamp, wall_ms);
                }
                CubeEvent::Gyro { quaternion, .. } => {
                    st.inner.feed_ble_gyro(
                        quaternion.x, quaternion.y, quaternion.z, quaternion.w,
                        timestamp, wall_ms,
                    );
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
                CubeEvent::RawFacelets { facelet_string } => {
                    st.inner.handle_raw_facelets(facelet_string);
                    actions.push(format!(r#"{{"type":"RawFacelets","data":"{}"}}"#, facelet_string));
                }
                CubeEvent::Facelets { cp, co, ep, eo, .. } => {
                    actions.push(format!(
                        r#"{{"type":"Facelets","data":{{"cp":{:?},"co":{:?},"ep":{:?},"eo":{:?}}}}}"#,
                        cp, co, ep, eo
                    ));
                }
                CubeEvent::Disconnect => {
                    actions.push(r#"{"type":"Disconnect"}"#.to_string());
                }
                CubeEvent::WriteBack { data } => {
                    let bytes: Vec<String> = data.iter().map(|b| b.to_string()).collect();
                    actions.push(format!(r#"{{"type":"WriteBack","data":[{}]}}"#, bytes.join(",")));
                }
                CubeEvent::MoveHistory { moves } => {
                    for m in moves {
                        if let CubeEvent::Move { face, direction, .. } = m {
                            st.inner.feed_ble_move(*face, *direction, timestamp, wall_ms);
                        }
                    }
                }
            }
        }

        // Flush interpreter + dispatch interpreted moves (core)
        let dispatch = st.inner.flush_and_dispatch(timestamp, wall_ms, &mut || js_sys::Math::random());
        actions.extend(dispatch.actions);

        // Renderer: animate face/slice moves + apply gyro offset
        for (notation, _kind) in &dispatch.animations {
            rouxflow_render::queue_move_anim(notation.clone(), 0.15);
        }
        if let Some((ox, oy, oz, ow)) = dispatch.render_offset {
            rouxflow_render::set_gyro_offset(ox, oy, oz, ow);
        }

        format_actions(&actions)
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
                let facelets = st.inner.cube_logic.facelets();
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
                if let Some(q) = st.inner.cube_logic.orientation {
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
                if let Some(q) = st.inner.cube_logic.orientation {
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
            if let Some(q) = st.inner.cube_logic.orientation {
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
                let f = st.inner.cube_logic.facelets();
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
        s.borrow_mut().as_mut().map_or_else(String::new, |st| {
            let actions = st.inner.update_timer(timestamp, &mut || js_sys::Math::random());
            format_actions(&actions)
        })
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
            let now = js_sys::Date::now() / 1000.0;
            st.inner.start_scramble_with(scramble, now)
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
        s.borrow().as_ref().map_or(true, |st| st.inner.cube_logic.is_solved())
    })
}

#[wasm_bindgen]
pub fn cm_reset_flow() -> String {
    APP_STATE.with(|s| {
        s.borrow_mut().as_mut().map_or_else(String::new, |st| {
            let cube_solved = st.inner.cube_logic.is_solved();
            let (actions, _cal_started) = st.inner.reset_flow(cube_solved, &mut || js_sys::Math::random());
            format_actions(&actions)
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
                return String::new();
            }
            st.inner.generate_new_scramble(&mut || js_sys::Math::random());
            st.inner.session.get_pending_scramble().unwrap_or("").to_string()
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

// ========== Telemetry: drain raw solve data ==========

/// Drain the completed solve telemetry as JSON. Returns "null" if none available.
/// Call this after a SaveSolve action to retrieve the raw data.
#[wasm_bindgen]
pub fn cm_drain_solve_telemetry() -> String {
    APP_STATE.with(|s| {
        let mut state = s.borrow_mut();
        match state.as_mut() {
            Some(st) => match st.inner.telemetry.take() {
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
