//! Supabase cloud storage implementation.
//!
//! Uses reqwest to call the Supabase REST API.
//! Sessions and solves are stubs for now (only cubes are synced to cloud).

use reqwest::{Client, header};
use rouxflow_core::session::{Session, Solve};
use rouxflow_core::storage::{Cube, Storage, StorageError};

pub struct CloudStorage {
    url: String,
    anon_key: String,
}

impl CloudStorage {
    pub fn new(url: String, anon_key: String) -> Self {
        Self { url, anon_key }
    }
}

#[async_trait::async_trait(?Send)]
impl Storage for CloudStorage {
    async fn get_cubes(&self, user_id: Option<&str>) -> Result<Vec<Cube>, StorageError> {
        let user_id = user_id.ok_or(StorageError {
            message: "User ID required for cloud storage".to_string(),
        })?;
        let client = Client::new();
        let url = format!("{}/rest/v1/cubes?user_id=eq.{}", self.url, user_id);

        let res = client
            .get(&url)
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", self.anon_key))
            .send()
            .await
            .map_err(|e| StorageError { message: e.to_string() })?;

        if !res.status().is_success() {
            return Err(StorageError {
                message: format!("Supabase error: {}", res.status()),
            });
        }

        res.json::<Vec<Cube>>()
            .await
            .map_err(|e| StorageError { message: e.to_string() })
    }

    async fn save_cube(&self, cube: &Cube) -> Result<(), StorageError> {
        let client = Client::new();
        let url = format!("{}/rest/v1/cubes", self.url);

        let res = client
            .post(&url)
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", self.anon_key))
            .header(header::CONTENT_TYPE, "application/json")
            .header("Prefer", "resolution=merge-duplicates")
            .json(cube)
            .send()
            .await
            .map_err(|e| StorageError { message: e.to_string() })?;

        if !res.status().is_success() {
            return Err(StorageError {
                message: format!("Supabase error: {}", res.status()),
            });
        }

        Ok(())
    }

    async fn delete_cube(&self, id: &str, user_id: &str) -> Result<(), StorageError> {
        let client = Client::new();
        let url = format!(
            "{}/rest/v1/cubes?id=eq.{}&user_id=eq.{}",
            self.url, id, user_id
        );

        let res = client
            .delete(&url)
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", self.anon_key))
            .send()
            .await
            .map_err(|e| StorageError { message: e.to_string() })?;

        if !res.status().is_success() {
            return Err(StorageError {
                message: format!("Supabase error: {}", res.status()),
            });
        }

        Ok(())
    }

    async fn get_sessions(&self) -> Result<Vec<Session>, StorageError> {
        // Cloud sessions not yet implemented
        Ok(Vec::new())
    }

    async fn create_session(&self, _session: &Session) -> Result<(), StorageError> {
        // Cloud sessions not yet implemented
        Ok(())
    }

    async fn save_solve(&self, _session_id: &str, _solve: &Solve) -> Result<(), StorageError> {
        // Cloud solves not yet implemented
        Ok(())
    }

    async fn demote_session(&self, _session_id: &str) -> Result<(), StorageError> {
        // Cloud sessions not yet implemented
        Ok(())
    }
}
