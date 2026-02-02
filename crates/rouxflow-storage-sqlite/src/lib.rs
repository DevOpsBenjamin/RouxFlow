//! SQLite Storage implementation for Tauri desktop app
//! Uses rusqlite for local persistence

use rouxflow_core::storage::{Storage, Cube, StorageError};
use rouxflow_core::session::{Session, Solve, SessionType};
use rusqlite::{Connection, params};
use std::sync::Mutex;

pub struct SqliteStorage {
    conn: Mutex<Connection>,
}

impl SqliteStorage {
    pub fn new(db_path: &str) -> Result<Self, StorageError> {
        let conn = Connection::open(db_path)
            .map_err(|e| StorageError { message: e.to_string() })?;
        
        // Initialize tables
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

        conn.execute(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                session_type TEXT NOT NULL,
                first_solve_at INTEGER
            )",
            [],
        ).map_err(|e| StorageError { message: e.to_string() })?;
    
        conn.execute(
            "CREATE TABLE IF NOT EXISTS solves (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                time INTEGER NOT NULL,
                moves TEXT NOT NULL,
                date INTEGER NOT NULL,
                is_valid INTEGER NOT NULL,
                FOREIGN KEY(session_id) REFERENCES sessions(id)
            )",
            [],
        ).map_err(|e| StorageError { message: e.to_string() })?;

        Ok(Self { conn: Mutex::new(conn) })
    }
}

#[async_trait::async_trait]
impl Storage for SqliteStorage {
    // Cubes
    async fn get_cubes(&self, user_id: Option<&str>) -> Result<Vec<Cube>, StorageError> {
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

    async fn save_cube(&self, cube: &Cube) -> Result<(), StorageError> {
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

    async fn delete_cube(&self, id: &str, user_id: &str) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM cubes WHERE id = ? AND (user_id = ? OR user_id IS NULL)",
            params![id, user_id],
        ).map_err(|e| StorageError { message: e.to_string() })?;
        Ok(())
    }

    // Sessions & Solves
    async fn get_sessions(&self) -> Result<Vec<Session>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name, session_type, first_solve_at FROM sessions")
            .map_err(|e| StorageError { message: e.to_string() })?;
            
        let session_iter = stmt.query_map([], |row| {
            let type_str: String = row.get(2)?;
            let session_type = if type_str == "WCA" { SessionType::WCA } else { SessionType::Free };
            
            Ok(Session {
                id: row.get(0)?,
                name: row.get(1)?,
                session_type,
                solves: Vec::new(),
                first_solve_at: row.get(3)?,
            })
        }).map_err(|e| StorageError { message: e.to_string() })?;
    
        let mut sessions = Vec::new();
        // Since we are inside a single lock, we need to iterate carefully.
        // Collecting sessions first to release the statement, but we hold the connection lock.
        // We can prepare another statement.
        
        let sessions_basic: Vec<Session> = session_iter.collect::<Result<Vec<_>, _>>()
            .map_err(|e| StorageError { message: e.to_string() })?;

        let mut solve_stmt = conn.prepare("SELECT id, time, moves, date, is_valid FROM solves WHERE session_id = ?")
            .map_err(|e| StorageError { message: e.to_string() })?;

        for mut session in sessions_basic {
            let solve_iter = solve_stmt.query_map([&session.id], |row| {
                let moves_str: String = row.get(2)?;
                let moves: Vec<String> = serde_json::from_str(&moves_str).unwrap_or_default();
                Ok(Solve {
                    id: row.get(0)?,
                    time: row.get(1)?,
                    moves,
                    date: row.get(3)?,
                    is_valid: (row.get::<_, i32>(4)? != 0),
                })
            }).map_err(|e| StorageError { message: e.to_string() })?;

            for solve in solve_iter {
                session.solves.push(solve.map_err(|e| StorageError { message: e.to_string() })?);
            }
            sessions.push(session);
        }
        Ok(sessions)
    }

    async fn create_session(&self, session: &Session) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        let type_str = match session.session_type {
            SessionType::WCA => "WCA",
            SessionType::Free => "Free",
        };
        conn.execute(
            "INSERT INTO sessions (id, name, session_type, first_solve_at) VALUES (?1, ?2, ?3, ?4)",
            params![session.id, session.name, type_str, session.first_solve_at],
        ).map_err(|e| StorageError { message: e.to_string() })?;
        Ok(())
    }

    async fn save_solve(&self, session_id: &str, solve: &Solve) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO solves (id, session_id, time, moves, date, is_valid)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                solve.id,
                session_id,
                solve.time,
                serde_json::to_string(&solve.moves).unwrap_or_default(),
                solve.date,
                if solve.is_valid { 1 } else { 0 }
            ],
        ).map_err(|e| StorageError { message: e.to_string() })?;
        Ok(())
    }

    async fn demote_session(&self, session_id: &str) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE sessions SET session_type = 'Free' WHERE id = ?",
            params![session_id],
        ).map_err(|e| StorageError { message: e.to_string() })?;
        Ok(())
    }
}
