mod db;
mod ble;

use db::{DbState, init_db};
use ble::{BleState, init_ble, ble_scan, ble_connect, ble_list_devices, ble_check_available};
use roux_core::session::{Session, Solve};
use tauri::{Manager, State, Emitter};
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

#[tauri::command]
fn db_save_cube(state: State<'_, DbState>, cube_json: String) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    let cube: db::Cube = serde_json::from_str(&cube_json).map_err(|e| e.to_string())?;
    db::save_cube(&conn, &cube).map_err(|e| e.to_string())
}

#[tauri::command]
fn db_get_cubes(state: State<'_, DbState>, user_id: Option<String>) -> Result<String, String> {
    let conn = state.0.lock().unwrap();
    let cubes = db::get_cubes(&conn, user_id.as_deref()).map_err(|e| e.to_string())?;
    serde_json::to_string(&cubes).map_err(|e| e.to_string())
}

#[tauri::command]
fn db_delete_cube(state: State<'_, DbState>, id: String) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    db::delete_cube(&conn, &id).map_err(|e| e.to_string())
}

#[tauri::command]
async fn db_sync_cubes(state: State<'_, DbState>, user_id: String, url: String, key: String) -> Result<(), String> {
    let storage = roux_core::storage::CloudStorage::new(url, key);
    
    // 1. Get local cubes
    let local_cubes = {
        let conn = state.0.lock().unwrap();
        db::get_cubes(&conn, Some(&user_id)).map_err(|e| e.to_string())?
    };

    // 2. Push local to cloud
    for cube in &local_cubes {
        let core_cube = roux_core::storage::Cube {
            id: cube.id.clone(),
            user_id: cube.user_id.clone(),
            name: cube.name.clone(),
            device_type: cube.device_type.clone(),
            mac_address: cube.mac_address.clone(),
            created_at: cube.created_at,
        };
        storage.save_cube(&core_cube).await.map_err(|e| e.to_string())?;
    }

    // 3. Pull from cloud
    let remote_cubes = storage.get_cubes(&user_id).await.map_err(|e| e.to_string())?;

    // 4. Save to local
    {
        let conn = state.0.lock().unwrap();
        for remote in remote_cubes {
            let local_cube = db::Cube {
                id: remote.id.clone(),
                user_id: remote.user_id.clone(),
                name: remote.name.clone(),
                device_type: remote.device_type.clone(),
                mac_address: remote.mac_address.clone(),
                created_at: remote.created_at,
            };
            db::save_cube(&conn, &local_cube).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            let _ = app.emit("deep-link://new-url", args);
        }))
        .plugin(tauri_plugin_deep_link::init())
        .setup(|app| {
            // Deep Link registration (Windows)
            #[cfg(target_os = "windows")]
            {
                use tauri_plugin_deep_link::DeepLinkExt;
                if let Err(e) = app.deep_link().register("rouxflow") {
                    println!("Failed to register deep link: {}", e);
                } else {
                    println!("Deep link 'rouxflow' registered successfully");
                }
            }

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
            ble_check_available,
            db_save_cube,
            db_get_cubes,
            db_delete_cube,
            db_sync_cubes,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
