use rusqlite::{params, Connection, Result};
use roux_core::session::{Session, Solve, SessionType};
use std::sync::Mutex;

pub struct DbState(pub Mutex<Connection>);

pub fn init_db<P: AsRef<std::path::Path>>(path: P) -> Result<Connection> {
    let conn = Connection::open(path)?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            session_type TEXT NOT NULL,
            first_solve_at INTEGER
        )",
        [],
    )?;

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
    )?;

    Ok(conn)
}

pub fn save_solve(conn: &Connection, session_id: &str, solve: &Solve) -> Result<()> {
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
    )?;
    Ok(())
}

pub fn get_sessions(conn: &Connection) -> Result<Vec<Session>> {
    let mut stmt = conn.prepare("SELECT id, name, session_type, first_solve_at FROM sessions")?;
    let session_iter = stmt.query_map([], |row| {
        let type_str: String = row.get(2)?;
        let session_type = if type_str == "WCA" { SessionType::WCA } else { SessionType::Free };
        
        Ok(Session {
            id: row.get(0)?,
            name: row.get(1)?,
            session_type,
            solves: Vec::new(), // Will be populated later or on demand
            first_solve_at: row.get(3)?,
        })
    })?;

    let mut sessions = Vec::new();
    for session in session_iter {
        let mut s = session?;
        // Load solves for each session (simplified for now)
        let mut solve_stmt = conn.prepare("SELECT id, time, moves, date, is_valid FROM solves WHERE session_id = ?")?;
        let solve_iter = solve_stmt.query_map([&s.id], |row| {
            let moves_str: String = row.get(2)?;
            let moves: Vec<String> = serde_json::from_str(&moves_str).unwrap_or_default();
            Ok(Solve {
                id: row.get(0)?,
                time: row.get(1)?,
                moves,
                date: row.get(3)?,
                is_valid: row.get::<_, i32>(4)? == 1,
            })
        })?;
        for solve in solve_iter {
            s.solves.push(solve?);
        }
        sessions.push(s);
    }
    Ok(sessions)
}

pub fn create_session(conn: &Connection, session: &Session) -> Result<()> {
    let type_str = match session.session_type {
        SessionType::WCA => "WCA",
        SessionType::Free => "Free",
    };
    conn.execute(
        "INSERT INTO sessions (id, name, session_type, first_solve_at) VALUES (?1, ?2, ?3, ?4)",
        params![session.id, session.name, type_str, session.first_solve_at],
    )?;
    Ok(())
}

pub fn demote_session(conn: &Connection, id: &str) -> Result<()> {
    conn.execute(
        "UPDATE sessions SET session_type = 'Free' WHERE id = ?",
        params![id],
    )?;
    Ok(())
}
