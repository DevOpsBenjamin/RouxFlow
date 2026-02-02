use serde::{Deserialize, Serialize};
use crate::session::{Session, Solve};

/// Cube Bluetooth device stored in database
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Cube {
    pub id: String,
    pub user_id: Option<String>,
    pub name: String,
    pub device_type: String,
    pub mac_address: String,
    pub created_at: i64,
}

/// Error type for storage operations
#[derive(Debug, Clone)]
pub struct StorageError {
    pub message: String,
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<String> for StorageError {
    fn from(s: String) -> Self {
        StorageError { message: s }
    }
}

use async_trait::async_trait;

/// Trait defining storage operations for cubes and sessions.
/// Implemented by SQLite (Tauri) and Supabase (Cloud/WASM).
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait Storage: Send + Sync {
    // Cubes
    async fn get_cubes(&self, user_id: Option<&str>) -> Result<Vec<Cube>, StorageError>;
    async fn save_cube(&self, cube: &Cube) -> Result<(), StorageError>;
    async fn delete_cube(&self, id: &str, user_id: &str) -> Result<(), StorageError>;

    // Sessions & Solves
    async fn get_sessions(&self) -> Result<Vec<Session>, StorageError>;
    async fn create_session(&self, session: &Session) -> Result<(), StorageError>;
    async fn save_solve(&self, session_id: &str, solve: &Solve) -> Result<(), StorageError>;
    async fn demote_session(&self, session_id: &str) -> Result<(), StorageError>;
}
