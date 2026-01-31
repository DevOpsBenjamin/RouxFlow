use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use reqwest::{Client, header};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Cube {
    pub id: String,
    pub user_id: Option<String>,
    pub name: String,
    pub device_type: String,
    pub mac_address: String,
    pub created_at: i64,
}

#[wasm_bindgen]
pub struct CloudStorage {
    url: String,
    anon_key: String,
}

#[wasm_bindgen]
impl CloudStorage {
    #[wasm_bindgen(constructor)]
    pub fn new(url: String, anon_key: String) -> Self {
        Self { url, anon_key }
    }

    // Helper to get raw JSON string (better for cross-language bridge)
    pub async fn get_cubes_json(&self, user_id: String) -> Result<String, String> {
        let client = Client::new();
        let url = format!("{}/rest/v1/cubes?user_id=eq.{}", self.url, user_id);
        
        let res = client.get(&url)
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", self.anon_key))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let text = res.text().await.map_err(|e| e.to_string())?;
        Ok(text)
    }

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

// Native-only methods (standard Rust)
impl CloudStorage {
    pub async fn get_cubes(&self, user_id: &str) -> Result<Vec<Cube>, String> {
        let json = self.get_cubes_json(user_id.to_string()).await?;
        serde_json::from_str(&json).map_err(|e| e.to_string())
    }

    pub async fn save_cube(&self, cube: &Cube) -> Result<(), String> {
        let json = serde_json::to_string(cube).map_err(|e| e.to_string())?;
        self.save_cube_json(json).await?;
        Ok(())
    }
}
