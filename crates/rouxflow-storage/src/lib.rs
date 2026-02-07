//! Unified storage for RouxFlow
//!
//! Provides offline-first local storage (IndexedDB) with optional
//! cloud sync (Supabase). All writes go to IndexedDB first;
//! Supabase sync happens opportunistically when network is available.
//!
//! This crate is WASM-only (uses browser APIs). Dependency gating in Cargo.toml
//! ensures it only compiles for wasm32 targets.

pub mod local;
pub mod cloud;
pub mod sync;
pub mod signing;

// Re-export the Storage trait and types from core
pub use rouxflow_core::storage::{Cube, Storage, StorageError};
pub use rouxflow_core::session::{Session, SessionType, Solve};

pub use sync::StorageManager;
