//! StorageManager — offline-first orchestrator.
//!
//! All writes go to IndexedDB first. Cloud sync happens opportunistically.
//! If Supabase is unavailable the app works normally with local data.

use rouxflow_core::session::{Session, Solve};
use rouxflow_core::storage::{Cube, Storage, StorageError};

use crate::cloud::CloudStorage;
use crate::local::LocalStorage;

pub struct StorageManager {
    local: LocalStorage,
    cloud: Option<CloudStorage>,
}

impl StorageManager {
    pub async fn new(supabase_url: Option<String>, supabase_key: Option<String>) -> Result<Self, StorageError> {
        let local = LocalStorage::new().await?;
        let cloud = match (supabase_url, supabase_key) {
            (Some(url), Some(key)) if !url.is_empty() && !key.is_empty() => {
                Some(CloudStorage::new(url, key))
            }
            _ => None,
        };
        Ok(Self { local, cloud })
    }

    /// Try to sync cubes to cloud. Silently ignores failures.
    pub async fn sync_cubes(&self, user_id: &str) {
        let cloud = match &self.cloud {
            Some(c) => c,
            None => return,
        };

        // Push local cubes to cloud
        if let Ok(local_cubes) = self.local.get_cubes(Some(user_id)).await {
            for cube in &local_cubes {
                let _ = cloud.save_cube(cube).await;
            }
        }

        // Pull cloud cubes to local
        if let Ok(cloud_cubes) = cloud.get_cubes(Some(user_id)).await {
            for cube in &cloud_cubes {
                let _ = self.local.save_cube(cube).await;
            }
        }
    }
}

#[async_trait::async_trait(?Send)]
impl Storage for StorageManager {
    async fn get_cubes(&self, user_id: Option<&str>) -> Result<Vec<Cube>, StorageError> {
        self.local.get_cubes(user_id).await
    }

    async fn save_cube(&self, cube: &Cube) -> Result<(), StorageError> {
        // Always write local first
        self.local.save_cube(cube).await?;

        // Best-effort cloud push
        if let Some(cloud) = &self.cloud {
            let _ = cloud.save_cube(cube).await;
        }

        Ok(())
    }

    async fn delete_cube(&self, id: &str, user_id: &str) -> Result<(), StorageError> {
        self.local.delete_cube(id, user_id).await?;

        if let Some(cloud) = &self.cloud {
            let _ = cloud.delete_cube(id, user_id).await;
        }

        Ok(())
    }

    async fn get_sessions(&self) -> Result<Vec<Session>, StorageError> {
        self.local.get_sessions().await
    }

    async fn create_session(&self, session: &Session) -> Result<(), StorageError> {
        self.local.create_session(session).await
    }

    async fn save_solve(&self, session_id: &str, solve: &Solve) -> Result<(), StorageError> {
        self.local.save_solve(session_id, solve).await
    }

    async fn demote_session(&self, session_id: &str) -> Result<(), StorageError> {
        self.local.demote_session(session_id).await
    }
}
