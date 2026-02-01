use rusqlite::{params, Connection, Result};
use rouxflow_core::session::{Session, Solve, SessionType};
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
    let session_iter = stmt.query_map([], |row: &rusqlite::Row| {
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
        let solve_iter = solve_stmt.query_map([&s.id], |row: &rusqlite::Row| {
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

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Cube {
    pub id: String,
    pub user_id: Option<String>,
    pub name: String,
    pub device_type: String,
    pub mac_address: String,
    pub created_at: i64,
}

pub fn save_cube(conn: &Connection, cube: &Cube) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO cubes (id, user_id, name, device_type, mac_address, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![cube.id, cube.user_id, cube.name, cube.device_type, cube.mac_address, cube.created_at],
    )?;
    Ok(())
}

pub fn get_cubes(conn: &Connection, user_id: Option<&str>) -> Result<Vec<Cube>> {
    let mut stmt = match user_id {
        Some(_) => conn.prepare("SELECT id, user_id, name, device_type, mac_address, created_at FROM cubes WHERE user_id = ?")?,
        None => conn.prepare("SELECT id, user_id, name, device_type, mac_address, created_at FROM cubes WHERE user_id IS NULL")?,
    };

    let mapper = |row: &rusqlite::Row| {
        Ok(Cube {
            id: row.get(0)?,
            user_id: row.get(1)?,
            name: row.get(2)?,
            device_type: row.get(3)?,
            mac_address: row.get(4)?,
            created_at: row.get(5)?,
        })
    };

    let cube_iter = match user_id {
        Some(uid) => stmt.query_map(params![uid], mapper)?,
        None => stmt.query_map([], mapper)?,
    };

    let mut cubes = Vec::new();
    for cube in cube_iter {
        cubes.push(cube?);
    }
    Ok(cubes)
}

pub fn delete_cube(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM cubes WHERE id = ?", params![id])?;
    Ok(())
}
