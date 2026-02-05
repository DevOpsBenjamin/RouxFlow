mod ble;

use ble::{BleState, init_ble, ble_scan, ble_connect, ble_list_devices, ble_check_available};
use rouxflow_core::session::{Session, Solve};
use rouxflow_core::storage::{Storage, Cube};
use rouxflow_storage_sqlite::SqliteStorage;
use tauri::{Manager, State, Emitter};
use std::sync::Arc;
use tokio::sync::Mutex;

// Generic wrapper if we wanted to abstract it, but concrete type is fine
// We will manage SqliteStorage directly in Tauri state.

#[tauri::command]
async fn db_save_solve(state: State<'_, SqliteStorage>, session_id: String, solve_json: String) -> Result<(), String> {
    let solve: Solve = serde_json::from_str(&solve_json).map_err(|e| e.to_string())?;
    state.save_solve(&session_id, &solve).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn db_get_sessions(state: State<'_, SqliteStorage>) -> Result<String, String> {
    let sessions = state.get_sessions().await.map_err(|e| e.to_string())?;
    serde_json::to_string(&sessions).map_err(|e| e.to_string())
}

#[tauri::command]
async fn db_create_session(state: State<'_, SqliteStorage>, session_json: String) -> Result<(), String> {
    let session: Session = serde_json::from_str(&session_json).map_err(|e| e.to_string())?;
    state.create_session(&session).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn db_demote_session(state: State<'_, SqliteStorage>, id: String) -> Result<(), String> {
    state.demote_session(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn db_save_cube(state: State<'_, SqliteStorage>, cube_json: String) -> Result<(), String> {
    let cube: Cube = serde_json::from_str(&cube_json).map_err(|e| e.to_string())?;
    state.save_cube(&cube).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn db_get_cubes(state: State<'_, SqliteStorage>, user_id: Option<String>) -> Result<String, String> {
    let cubes = state.get_cubes(user_id.as_deref()).await.map_err(|e| e.to_string())?;
    serde_json::to_string(&cubes).map_err(|e| e.to_string())
}

#[tauri::command]
async fn db_delete_cube(state: State<'_, SqliteStorage>, id: String, user_id: String) -> Result<(), String> {
    state.delete_cube(&id, &user_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn db_sync_cubes(_state: State<'_, SqliteStorage>, _user_id: String) -> Result<(), String> {
    // TODO: Implement sync logic
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            let _ = app.get_webview_window("main").expect("no main window").set_focus();
        }))
        .setup(|app| {
            let app_handle = app.handle().clone();
            let app_dir = app.path().app_data_dir().expect("failed to get app data dir");
            
            // Initialize Storage and BLE asynchronously
            tauri::async_runtime::spawn(async move {
                std::fs::create_dir_all(&app_dir).expect("failed to create app dir");
                let db_path = app_dir.join("rouxflow.db");
                
                match SqliteStorage::new(db_path.to_str().unwrap()) {
                    Ok(storage) => {
                         app_handle.manage(storage);
                    },
                    Err(e) => {
                        eprintln!("Failed to init SqliteStorage: {}", e);
                    }
                }

                match init_ble().await {
                   Ok(ble_state) => {
                       app_handle.manage(ble_state);
                   },
                   Err(e) => {
                       eprintln!("Failed to init BLE: {}", e);
                   }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            db_save_solve,
            db_get_sessions,
            db_create_session,
            db_demote_session,
            db_save_cube,
            db_get_cubes,
            db_delete_cube,
            db_sync_cubes,
            ble_scan,
            ble_connect,
            ble_list_devices,
            ble_check_available
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}