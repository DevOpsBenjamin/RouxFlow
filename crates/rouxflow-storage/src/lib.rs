//! Unified storage for RouxFlow
//!
//! Provides offline-first local storage (IndexedDB) with optional
//! cloud sync (Supabase). All writes go to IndexedDB first;
//! Supabase sync happens opportunistically when network is available.

#[cfg(target_arch = "wasm32")]
pub mod local;
#[cfg(target_arch = "wasm32")]
pub mod cloud;
#[cfg(target_arch = "wasm32")]
pub mod sync;
#[cfg(target_arch = "wasm32")]
pub mod signing;

// Re-export the Storage trait and types from core
pub use rouxflow_core::storage::{Cube, Storage, StorageError};
pub use rouxflow_core::session::{Session, SessionType, Solve};

#[cfg(target_arch = "wasm32")]
pub use sync::StorageManager;
