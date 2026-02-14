//! IndexedDB local storage implementation using rexie.
//!
//! Object stores mirror the SQLite schema:
//! - `cubes` — {id, user_id, name, device_type, mac_address, created_at}
//! - `sessions` — {id, name, session_type, first_solve_at}
//! - `solves` — {id, session_id, time, moves, date, is_valid}

use rexie::{Rexie, ObjectStore, TransactionMode};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

use rouxflow_core::session::{Session, SessionType, Solve};
use rouxflow_core::storage::{Cube, Storage, StorageError};

const DB_NAME: &str = "rouxflow";
const DB_VERSION: u32 = 1;

pub struct LocalStorage {
    db: Rexie,
}

// Intermediate struct for serializing solves to IndexedDB
#[derive(Serialize, Deserialize)]
struct SolveRecord {
    id: String,
    session_id: String,
    time: u32,
    moves: String, // JSON-encoded Vec<String>
    date: i64,
    is_valid: bool,
    #[serde(default)]
    scramble: Option<String>,
    #[serde(default)]
    timed_moves: Option<String>, // JSON-encoded Vec<TimedMove>
    #[serde(default)]
    penalty: Option<String>,
    #[serde(default)]
    deleted_at: Option<i64>,
}

// Intermediate struct for serializing sessions to IndexedDB
#[derive(Serialize, Deserialize)]
struct SessionRecord {
    id: String,
    name: String,
    session_type: String,
    first_solve_at: Option<i64>,
}

impl LocalStorage {
    pub async fn new() -> Result<Self, StorageError> {
        let db = Rexie::builder(DB_NAME)
            .version(DB_VERSION)
            .add_object_store(ObjectStore::new("cubes").key_path("id"))
            .add_object_store(ObjectStore::new("sessions").key_path("id"))
            .add_object_store(
                ObjectStore::new("solves")
                    .key_path("id")
                    .add_index(rexie::Index::new("session_id", "session_id")),
            )
            .build()
            .await
            .map_err(|e| StorageError { message: format!("IndexedDB init failed: {:?}", e) })?;

        Ok(Self { db })
    }
}

fn to_js(val: &impl Serialize) -> Result<JsValue, StorageError> {
    serde_wasm_bindgen::to_value(val)
        .map_err(|e| StorageError { message: format!("Serialize error: {}", e) })
}

fn from_js<T: for<'de> Deserialize<'de>>(val: JsValue) -> Result<T, StorageError> {
    serde_wasm_bindgen::from_value(val)
        .map_err(|e| StorageError { message: format!("Deserialize error: {}", e) })
}

#[async_trait::async_trait(?Send)]
impl Storage for LocalStorage {
    async fn get_cubes(&self, user_id: Option<&str>) -> Result<Vec<Cube>, StorageError> {
        let tx = self.db.transaction(&["cubes"], TransactionMode::ReadOnly)
            .map_err(|e| StorageError { message: format!("Transaction error: {:?}", e) })?;
        let store = tx.store("cubes")
            .map_err(|e| StorageError { message: format!("Store error: {:?}", e) })?;

        let all = store.get_all(None, None).await
            .map_err(|e| StorageError { message: format!("GetAll error: {:?}", e) })?;

        let mut cubes: Vec<Cube> = Vec::new();
        for value in all {
            if let Ok(cube) = from_js::<Cube>(value) {
                match user_id {
                    Some(uid) => {
                        if cube.user_id.as_deref() == Some(uid) || cube.user_id.is_none() {
                            cubes.push(cube);
                        }
                    }
                    None => {
                        if cube.user_id.is_none() {
                            cubes.push(cube);
                        }
                    }
                }
            }
        }
        Ok(cubes)
    }

    async fn save_cube(&self, cube: &Cube) -> Result<(), StorageError> {
        let tx = self.db.transaction(&["cubes"], TransactionMode::ReadWrite)
            .map_err(|e| StorageError { message: format!("Transaction error: {:?}", e) })?;
        let store = tx.store("cubes")
            .map_err(|e| StorageError { message: format!("Store error: {:?}", e) })?;

        store.put(&to_js(cube)?, None).await
            .map_err(|e| StorageError { message: format!("Put error: {:?}", e) })?;

        tx.done().await
            .map_err(|e| StorageError { message: format!("Commit error: {:?}", e) })?;
        Ok(())
    }

    async fn delete_cube(&self, id: &str, _user_id: &str) -> Result<(), StorageError> {
        let tx = self.db.transaction(&["cubes"], TransactionMode::ReadWrite)
            .map_err(|e| StorageError { message: format!("Transaction error: {:?}", e) })?;
        let store = tx.store("cubes")
            .map_err(|e| StorageError { message: format!("Store error: {:?}", e) })?;

        store.delete(JsValue::from_str(id)).await
            .map_err(|e| StorageError { message: format!("Delete error: {:?}", e) })?;

        tx.done().await
            .map_err(|e| StorageError { message: format!("Commit error: {:?}", e) })?;
        Ok(())
    }

    async fn get_sessions(&self) -> Result<Vec<Session>, StorageError> {
        let tx = self.db.transaction(&["sessions", "solves"], TransactionMode::ReadOnly)
            .map_err(|e| StorageError { message: format!("Transaction error: {:?}", e) })?;
        let session_store = tx.store("sessions")
            .map_err(|e| StorageError { message: format!("Store error: {:?}", e) })?;
        let solve_store = tx.store("solves")
            .map_err(|e| StorageError { message: format!("Store error: {:?}", e) })?;

        let all_sessions = session_store.get_all(None, None).await
            .map_err(|e| StorageError { message: format!("GetAll error: {:?}", e) })?;

        let all_solves = solve_store.get_all(None, None).await
            .map_err(|e| StorageError { message: format!("GetAll error: {:?}", e) })?;

        // Parse all solves into a lookup by session_id
        let mut solve_map: std::collections::HashMap<String, Vec<Solve>> = std::collections::HashMap::new();
        for value in all_solves {
            if let Ok(record) = from_js::<SolveRecord>(value) {
                if record.deleted_at.is_some() { continue; } // Skip soft-deleted
                let moves: Vec<String> = serde_json::from_str(&record.moves).unwrap_or_default();
                let timed_moves = record.timed_moves.as_deref()
                    .and_then(|s| serde_json::from_str(s).ok());
                let solve = Solve {
                    id: record.id,
                    time: record.time,
                    moves,
                    date: record.date,
                    is_valid: record.is_valid,
                    scramble: record.scramble,
                    timed_moves,
                    penalty: record.penalty,
                    deleted_at: record.deleted_at,
                };
                solve_map.entry(record.session_id).or_default().push(solve);
            }
        }

        let mut sessions = Vec::new();
        for value in all_sessions {
            if let Ok(record) = from_js::<SessionRecord>(value) {
                let session_type = if record.session_type == "WCA" {
                    SessionType::WCA
                } else {
                    SessionType::Free
                };
                let solves = solve_map.remove(&record.id).unwrap_or_default();
                sessions.push(Session {
                    id: record.id,
                    name: record.name,
                    session_type,
                    solves,
                    first_solve_at: record.first_solve_at,
                });
            }
        }

        Ok(sessions)
    }

    async fn create_session(&self, session: &Session) -> Result<(), StorageError> {
        let tx = self.db.transaction(&["sessions"], TransactionMode::ReadWrite)
            .map_err(|e| StorageError { message: format!("Transaction error: {:?}", e) })?;
        let store = tx.store("sessions")
            .map_err(|e| StorageError { message: format!("Store error: {:?}", e) })?;

        let record = SessionRecord {
            id: session.id.clone(),
            name: session.name.clone(),
            session_type: match session.session_type {
                SessionType::WCA => "WCA".to_string(),
                SessionType::Free => "Free".to_string(),
            },
            first_solve_at: session.first_solve_at,
        };

        store.put(&to_js(&record)?, None).await
            .map_err(|e| StorageError { message: format!("Put error: {:?}", e) })?;

        tx.done().await
            .map_err(|e| StorageError { message: format!("Commit error: {:?}", e) })?;
        Ok(())
    }

    async fn save_solve(&self, session_id: &str, solve: &Solve) -> Result<(), StorageError> {
        let tx = self.db.transaction(&["solves"], TransactionMode::ReadWrite)
            .map_err(|e| StorageError { message: format!("Transaction error: {:?}", e) })?;
        let store = tx.store("solves")
            .map_err(|e| StorageError { message: format!("Store error: {:?}", e) })?;

        let record = SolveRecord {
            id: solve.id.clone(),
            session_id: session_id.to_string(),
            time: solve.time,
            moves: serde_json::to_string(&solve.moves).unwrap_or_default(),
            date: solve.date,
            is_valid: solve.is_valid,
            scramble: solve.scramble.clone(),
            timed_moves: solve.timed_moves.as_ref()
                .map(|tm| serde_json::to_string(tm).unwrap_or_default()),
            penalty: solve.penalty.clone(),
            deleted_at: solve.deleted_at,
        };

        store.put(&to_js(&record)?, None).await
            .map_err(|e| StorageError { message: format!("Put error: {:?}", e) })?;

        tx.done().await
            .map_err(|e| StorageError { message: format!("Commit error: {:?}", e) })?;
        Ok(())
    }

    async fn get_solves(&self, session_id: &str) -> Result<Vec<Solve>, StorageError> {
        let tx = self.db.transaction(&["solves"], TransactionMode::ReadOnly)
            .map_err(|e| StorageError { message: format!("Transaction error: {:?}", e) })?;
        let store = tx.store("solves")
            .map_err(|e| StorageError { message: format!("Store error: {:?}", e) })?;

        let all = store.get_all(None, None).await
            .map_err(|e| StorageError { message: format!("GetAll error: {:?}", e) })?;

        let mut solves = Vec::new();
        for value in all {
            if let Ok(record) = from_js::<SolveRecord>(value) {
                if record.session_id == session_id && record.deleted_at.is_none() {
                    let moves: Vec<String> = serde_json::from_str(&record.moves).unwrap_or_default();
                    let timed_moves = record.timed_moves.as_deref()
                        .and_then(|s| serde_json::from_str(s).ok());
                    solves.push(Solve {
                        id: record.id,
                        time: record.time,
                        moves,
                        date: record.date,
                        is_valid: record.is_valid,
                        scramble: record.scramble,
                        timed_moves,
                        penalty: record.penalty,
                        deleted_at: record.deleted_at,
                    });
                }
            }
        }
        Ok(solves)
    }

    async fn demote_session(&self, session_id: &str) -> Result<(), StorageError> {
        let tx = self.db.transaction(&["sessions"], TransactionMode::ReadWrite)
            .map_err(|e| StorageError { message: format!("Transaction error: {:?}", e) })?;
        let store = tx.store("sessions")
            .map_err(|e| StorageError { message: format!("Store error: {:?}", e) })?;

        let existing = store.get(JsValue::from_str(session_id)).await
            .map_err(|e| StorageError { message: format!("Get error: {:?}", e) })?;

        if let Some(value) = existing {
            if let Ok(mut record) = from_js::<SessionRecord>(value) {
                record.session_type = "Free".to_string();
                store.put(&to_js(&record)?, None).await
                    .map_err(|e| StorageError { message: format!("Put error: {:?}", e) })?;
            }
        }

        tx.done().await
            .map_err(|e| StorageError { message: format!("Commit error: {:?}", e) })?;
        Ok(())
    }
}
