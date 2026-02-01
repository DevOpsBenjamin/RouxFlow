//! SQLite Storage implementation for Tauri desktop app
//! Uses rusqlite for local persistence

use roux_core::storage::{Storage, Cube, StorageError};
use rusqlite::{Connection, params};
use std::sync::Mutex;

pub struct SqliteStorage {
    conn: Mutex<Connection>,
}

impl SqliteStorage {
    pub fn new(db_path: &str) -> Result<Self, StorageError> {
        let conn = Connection::open(db_path)
            .map_err(|e| StorageError { message: e.to_string() })?;
        
        // Initialize table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS cubes (
                id TEXT PRIMARY KEY,
                user_id TEXT,
                name TEXT NOT NULL,
                device_type TEXT NOT NULL,
                mac_address TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )",
            [],
        ).map_err(|e| StorageError { message: e.to_string() })?;

        Ok(Self { conn: Mutex::new(conn) })
    }
}

impl Storage for SqliteStorage {
    fn get_cubes(&self, user_id: &str) -> Result<Vec<Cube>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, user_id, name, device_type, mac_address, created_at 
             FROM cubes WHERE user_id = ? OR user_id IS NULL"
        ).map_err(|e| StorageError { message: e.to_string() })?;

        let cubes = stmt.query_map(params![user_id], |row| {
            Ok(Cube {
                id: row.get(0)?,
                user_id: row.get(1)?,
                name: row.get(2)?,
                device_type: row.get(3)?,
                mac_address: row.get(4)?,
                created_at: row.get(5)?,
            })
        }).map_err(|e| StorageError { message: e.to_string() })?;

        cubes.collect::<Result<Vec<_>, _>>()
            .map_err(|e| StorageError { message: e.to_string() })
    }

    fn save_cube(&self, cube: &Cube) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO cubes (id, user_id, name, device_type, mac_address, created_at)
             VALUES (?, ?, ?, ?, ?, ?)",
            params![
                cube.id,
                cube.user_id,
                cube.name,
                cube.device_type,
                cube.mac_address,
                cube.created_at
            ],
        ).map_err(|e| StorageError { message: e.to_string() })?;
        Ok(())
    }

    fn delete_cube(&self, id: &str, user_id: &str) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM cubes WHERE id = ? AND (user_id = ? OR user_id IS NULL)",
            params![id, user_id],
        ).map_err(|e| StorageError { message: e.to_string() })?;
        Ok(())
    }
}
