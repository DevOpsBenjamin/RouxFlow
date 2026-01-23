mod db;
mod ble;

use db::{DbState, init_db};
use ble::{BleState, init_ble, ble_scan, ble_connect, ble_list_devices, ble_check_available};
use roux_core::session::{Session, Solve};
use tauri::{Manager, State};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tauri::command]
fn db_save_solve(state: State<'_, DbState>, session_id: String, solve_json: String) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    let solve: Solve = serde_json::from_str(&solve_json).map_err(|e| e.to_string())?;
    db::save_solve(&conn, &session_id, &solve).map_err(|e| e.to_string())
}

#[tauri::command]
fn db_get_sessions(state: State<'_, DbState>) -> Result<String, String> {
    let conn = state.0.lock().unwrap();
    let sessions = db::get_sessions(&conn).map_err(|e| e.to_string())?;
    serde_json::to_string(&sessions).map_err(|e| e.to_string())
}

#[tauri::command]
fn db_create_session(state: State<'_, DbState>, session_json: String) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    let session: Session = serde_json::from_str(&session_json).map_err(|e| e.to_string())?;
    db::create_session(&conn, &session).map_err(|e| e.to_string())
}

#[tauri::command]
fn db_demote_session(state: State<'_, DbState>, id: String) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    db::demote_session(&conn, &id).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // DB Init
            let app_data_dir = app.path().app_data_dir().expect("failed to get app data dir");
            std::fs::create_dir_all(&app_data_dir).expect("failed to create app data dir");
            let db_path = app_data_dir.join("RouxFlow.db");

            let conn = init_db(db_path).expect("failed to initialize database");
            app.manage(DbState(std::sync::Mutex::new(conn)));

            // BLE Init
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Ok(adapter) = init_ble().await {
                    handle.manage(BleState {
                        adapter,
                        connected_peripheral: Arc::new(Mutex::new(None)),
                    });
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            db_save_solve,
            db_get_sessions,
            db_create_session,
            db_demote_session,
            ble_scan,
            ble_connect,
            ble_list_devices,
            ble_check_available
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
