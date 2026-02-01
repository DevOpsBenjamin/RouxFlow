# Architecture RouxFlow (Trait-Based Storage)

## Vue d'ensemble

```
┌─────────────────────────────────────────────────────────────┐
│                        roux-core                            │
│  ┌─────────────────┐  ┌──────────────────────────────────┐  │
│  │ trait Storage   │  │ Logique Métier (Sessions, Moves) │  │
│  │ - get_cubes()   │  │ - SessionManager                 │  │
│  │ - save_cube()   │  │ - ScrambleValidator              │  │
│  │ - delete_cube() │  │ - PhaseDetector                  │  │
│  └─────────────────┘  └──────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
           │                              │
           ▼                              ▼
┌──────────────────────┐      ┌──────────────────────┐
│ roux-storage-sqlite  │      │ roux-storage-cloud   │
│ impl Storage for     │      │ impl Storage for     │
│ SqliteStorage        │      │ SupabaseStorage      │
│ (rusqlite)           │      │ (reqwest)            │
└──────────────────────┘      └──────────────────────┘
           │                              │
           ▼                              ▼
┌──────────────────────┐      ┌──────────────────────┐
│     src-tauri        │      │     WASM (Web)       │
│ Appelle SQLite +     │      │ Appelle Cloud        │
│ Cloud pour sync      │      │ directement          │
└──────────────────────┘      └──────────────────────┘
```

## Le Trait `Storage` (roux-core)

```rust
// crates/roux-core/src/storage.rs
use async_trait::async_trait;

#[async_trait]
pub trait Storage: Send + Sync {
    async fn get_cubes(&self, user_id: &str) -> Result<Vec<Cube>, StorageError>;
    async fn save_cube(&self, cube: &Cube) -> Result<(), StorageError>;
    async fn delete_cube(&self, id: &str) -> Result<(), StorageError>;
    
    // Sessions
    async fn get_sessions(&self, user_id: &str) -> Result<Vec<Session>, StorageError>;
    async fn save_session(&self, session: &Session) -> Result<(), StorageError>;
    async fn save_solve(&self, session_id: &str, solve: &Solve) -> Result<(), StorageError>;
}
```

## Implémentation SQLite (roux-storage-sqlite)

```rust
// crates/roux-storage-sqlite/src/lib.rs
use roux_core::{Storage, Cube, StorageError};
use rusqlite::Connection;

pub struct SqliteStorage {
    conn: Connection,
}

#[async_trait]
impl Storage for SqliteStorage {
    async fn get_cubes(&self, user_id: &str) -> Result<Vec<Cube>, StorageError> {
        // SELECT * FROM cubes WHERE user_id = ?
    }
    // ...
}
```

## Implémentation Cloud (roux-storage-cloud)

```rust
// crates/roux-storage-cloud/src/lib.rs
use roux_core::{Storage, Cube, StorageError};

pub struct SupabaseStorage {
    url: String,
    key: String,
    client: reqwest::Client,
}

#[async_trait]
impl Storage for SupabaseStorage {
    async fn get_cubes(&self, user_id: &str) -> Result<Vec<Cube>, StorageError> {
        // GET /rest/v1/cubes?user_id=eq.{user_id}
    }
    // ...
}
```

## Synchronisation (src-tauri)

```rust
// src-tauri/src/sync.rs
use roux_core::Storage;

pub async fn sync_cubes<L: Storage, R: Storage>(
    local: &L, 
    remote: &R, 
    user_id: &str
) -> Result<(), SyncError> {
    let local_cubes = local.get_cubes(user_id).await?;
    let remote_cubes = remote.get_cubes(user_id).await?;
    
    // Merge strategy: Remote wins for conflicts
    for cube in remote_cubes {
        if !local_cubes.contains(&cube) {
            local.save_cube(&cube).await?;
        }
    }
    
    // Push local-only to remote
    for cube in local_cubes {
        if !remote_cubes.contains(&cube) {
            remote.save_cube(&cube).await?;
        }
    }
    
    Ok(())
}
```

## Bridge TypeScript (Simplifié)

```typescript
// bridge.ts
const tauriDriver = {
    getCubes: (userId) => invoke('storage_get_cubes', { userId }),
    saveCube: (cube) => invoke('storage_save_cube', { cube }),
}

const wasmDriver = {
    getCubes: (userId) => wasmCloudStorage.get_cubes(userId),
    saveCube: (cube) => wasmCloudStorage.save_cube(cube),
}

export class CubeBridge {
    static getCubes(userId: string) {
        return isTauri ? tauriDriver.getCubes(userId) : wasmDriver.getCubes(userId)
    }
}
```

## Structure des Crates

```
crates/
├── roux-core/           # Traits + Logique Métier
│   └── src/
│       ├── lib.rs
│       ├── storage.rs   # trait Storage
│       └── session.rs   # SessionManager
├── roux-storage-sqlite/ # impl Storage (rusqlite)
│   └── src/lib.rs
└── roux-storage-cloud/  # impl Storage (reqwest) → WASM
    └── src/lib.rs
```
