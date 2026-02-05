//! Single WASM entry point for RouxFlow.
//!
//! Re-exports from: rouxflow-core, rouxflow-render, rouxflow-bluetoothcube, rouxflow-storage.
//! This is the only crate compiled as `cdylib` for wasm-pack.

use wasm_bindgen::prelude::*;

// ========== INIT ==========

#[wasm_bindgen(start)]
pub fn wasm_init() {
    console_error_panic_hook::set_once();
}

// ========== CORE: BLE packet handling ==========

#[wasm_bindgen]
pub fn encode_cube_command(command_id: u8, device_id: &str, use_moyu_key: bool) -> Vec<u8> {
    rouxflow_core::encode_cube_command(command_id, device_id, use_moyu_key)
}

#[wasm_bindgen]
pub fn handle_ble_packet(data: &[u8], device_id: &str, session: &mut SessionManager) -> String {
    rouxflow_core::handle_ble_packet(data, device_id, &mut session.inner)
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

#[wasm_bindgen]
pub fn find_cube_by_ble_name(device_name: &str) -> JsValue {
    match rouxflow_bluetoothcube::find_cube_by_ble_name(device_name) {
        Some(cube) => {
            // Return a simple JS object with the fields the frontend needs
            let obj = js_sys::Object::new();
            js_sys::Reflect::set(&obj, &"name".into(), &cube.name.into()).unwrap();
            js_sys::Reflect::set(&obj, &"manufacturer".into(), &format!("{:?}", cube.manufacturer).into()).unwrap();
            js_sys::Reflect::set(&obj, &"protocol".into(), &format!("{:?}", cube.protocol).into()).unwrap();

            let has_gyro = cube.features.contains(&rouxflow_bluetoothcube::CubeFeature::Gyroscope);
            js_sys::Reflect::set(&obj, &"hasGyro".into(), &has_gyro.into()).unwrap();

            let profile = cube.protocol.ble_profile();
            js_sys::Reflect::set(&obj, &"serviceUuid".into(), &profile.service_uuid.into()).unwrap();
            js_sys::Reflect::set(&obj, &"stateCharacteristic".into(), &profile.state_characteristic.into()).unwrap();
            js_sys::Reflect::set(&obj, &"commandCharacteristic".into(), &profile.command_characteristic.into()).unwrap();

            obj.into()
        }
        None => JsValue::NULL,
    }
}

#[wasm_bindgen]
pub fn all_scan_service_uuids() -> Vec<String> {
    rouxflow_bluetoothcube::all_scan_service_uuids()
        .into_iter()
        .map(|s| s.to_string())
        .collect()
}

#[wasm_bindgen]
pub fn all_scan_name_prefixes() -> Vec<String> {
    rouxflow_bluetoothcube::all_scan_name_prefixes()
        .into_iter()
        .map(|s| s.to_string())
        .collect()
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
