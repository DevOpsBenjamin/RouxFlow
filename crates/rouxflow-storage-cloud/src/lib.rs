//! Cloud Storage implementation using Supabase REST API
//! Compiled to WASM for web platform

use wasm_bindgen::prelude::*;
use rouxflow_core::storage::{Storage, Cube, StorageError};
use reqwest::{Client, header};

use async_trait::async_trait;

use rouxflow_core::session::{Session, Solve};

#[wasm_bindgen]
pub struct SupabaseStorage {
    url: String,
    anon_key: String,
}

#[wasm_bindgen]
impl SupabaseStorage {
    #[wasm_bindgen(constructor)]
    pub fn new(url: String, anon_key: String) -> Self {
        Self { url, anon_key }
    }

    /// Get cubes as JSON string (for WASM bridge)
    pub async fn get_cubes_json(&self, user_id: String) -> Result<String, String> {
        let cubes = self.get_cubes(Some(&user_id)).await.map_err(|e| e.message)?;
        serde_json::to_string(&cubes).map_err(|e| e.to_string())
    }

    /// Save cube from JSON string (for WASM bridge)
    pub async fn save_cube_json(&self, cube_json: String) -> Result<String, String> {
        let cube: Cube = serde_json::from_str(&cube_json).map_err(|e| e.to_string())?;
        self.save_cube(&cube).await.map_err(|e| e.message)?;
        Ok("ok".into())
    }

    /// Delete cube (for WASM bridge)
    pub async fn delete_cube_json(&self, id: String, user_id: String) -> Result<String, String> {
        self.delete_cube(&id, &user_id).await.map_err(|e| e.message)?;
        Ok("ok".into())
    }
}

// Native Rust trait implementation (async)
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Storage for SupabaseStorage {
    async fn get_cubes(&self, user_id: Option<&str>) -> Result<Vec<Cube>, StorageError> {
        let user_id = user_id.ok_or(StorageError { message: "User ID required for cloud storage".to_string() })?;
        let client = Client::new();
        let url = format!("{}/rest/v1/cubes?user_id=eq.{}", self.url, user_id);
        
        let res = client.get(&url)
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", self.anon_key))
            .send()
            .await
            .map_err(|e| StorageError { message: e.to_string() })?;

        if !res.status().is_success() {
            return Err(StorageError { message: format!("Supabase error: {}", res.status()) });
        }

        res.json::<Vec<Cube>>().await.map_err(|e| StorageError { message: e.to_string() })
    }

    async fn save_cube(&self, cube: &Cube) -> Result<(), StorageError> {
        let client = Client::new();
        let url = format!("{}/rest/v1/cubes", self.url);
        
        let res = client.post(&url)
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", self.anon_key))
            .header(header::CONTENT_TYPE, "application/json")
            .header("Prefer", "resolution=merge-duplicates")
            .json(cube)
            .send()
            .await
            .map_err(|e| StorageError { message: e.to_string() })?;

        if !res.status().is_success() {
            return Err(StorageError { message: format!("Supabase error: {}", res.status()) });
        }

        Ok(())
    }

    async fn delete_cube(&self, id: &str, user_id: &str) -> Result<(), StorageError> {
        let client = Client::new();
        let url = format!("{}/rest/v1/cubes?id=eq.{}&user_id=eq.{}", self.url, id, user_id);
        
        let res = client.delete(&url)
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", self.anon_key))
            .send()
            .await
            .map_err(|e| StorageError { message: e.to_string() })?;

        if !res.status().is_success() {
            return Err(StorageError { message: format!("Supabase error: {}", res.status()) });
        }

        Ok(())
    }

    // Connect Sessions & Solves (Stubs)
    async fn get_sessions(&self) -> Result<Vec<Session>, StorageError> {
        Err(StorageError { message: "Not implemented for cloud storage".to_string() })
    }

    async fn create_session(&self, _session: &Session) -> Result<(), StorageError> {
        Err(StorageError { message: "Not implemented for cloud storage".to_string() })
    }

    async fn save_solve(&self, _session_id: &str, _solve: &Solve) -> Result<(), StorageError> {
        Err(StorageError { message: "Not implemented for cloud storage".to_string() })
    }

    async fn demote_session(&self, _session_id: &str) -> Result<(), StorageError> {
        Err(StorageError { message: "Not implemented for cloud storage".to_string() })
    }
}
