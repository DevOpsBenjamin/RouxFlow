//! Cloud Storage implementation using Supabase REST API
//! Compiled to WASM for web platform

use wasm_bindgen::prelude::*;
use roux_core::storage::{Storage, Cube, StorageError};
use reqwest::{Client, header};

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
        let client = Client::new();
        let url = format!("{}/rest/v1/cubes?user_id=eq.{}", self.url, user_id);
        
        let res = client.get(&url)
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", self.anon_key))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        res.text().await.map_err(|e| e.to_string())
    }

    /// Save cube from JSON string (for WASM bridge)
    pub async fn save_cube_json(&self, cube_json: String) -> Result<String, String> {
        let client = Client::new();
        let url = format!("{}/rest/v1/cubes", self.url);
        
        let res = client.post(&url)
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", self.anon_key))
            .header(header::CONTENT_TYPE, "application/json")
            .header("Prefer", "resolution=merge-duplicates")
            .body(cube_json)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            return Err(format!("Supabase error: {}", res.status()));
        }

        Ok("ok".into())
    }

    /// Delete cube (for WASM bridge)
    pub async fn delete_cube_json(&self, id: String, user_id: String) -> Result<String, String> {
        let client = Client::new();
        let url = format!("{}/rest/v1/cubes?id=eq.{}&user_id=eq.{}", self.url, id, user_id);
        
        let res = client.delete(&url)
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", self.anon_key))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            return Err(format!("Supabase error: {}", res.status()));
        }

        Ok("ok".into())
    }
}

// Native Rust trait implementation (non-WASM, for Tauri sync)
impl Storage for SupabaseStorage {
    fn get_cubes(&self, _user_id: &str) -> Result<Vec<Cube>, StorageError> {
        // For sync we need a blocking runtime, handled in Tauri command
        unimplemented!("Use async get_cubes_json for native")
    }

    fn save_cube(&self, _cube: &Cube) -> Result<(), StorageError> {
        unimplemented!("Use async save_cube_json for native")
    }

    fn delete_cube(&self, _id: &str, _user_id: &str) -> Result<(), StorageError> {
        unimplemented!("Use async delete_cube_json for native")
    }
}
