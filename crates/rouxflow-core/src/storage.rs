use serde::{Deserialize, Serialize};

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

/// Trait defining storage operations for cubes and sessions.
/// Implemented by SQLite (Tauri) and Supabase (Cloud/WASM).
pub trait Storage {
    fn get_cubes(&self, user_id: &str) -> Result<Vec<Cube>, StorageError>;
    fn save_cube(&self, cube: &Cube) -> Result<(), StorageError>;
    fn delete_cube(&self, id: &str, user_id: &str) -> Result<(), StorageError>;
}
