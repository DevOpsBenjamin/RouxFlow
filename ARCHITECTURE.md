# Architecture RouxFlow (PWA + WASM unifié)

## Vue d'ensemble

```
┌─────────────────────────────────────────────────────────────┐
│                         Browser                              │
│  ┌─────────────┐  ┌────────────────────────────────────────┐│
│  │ Service     │  │  Vue 3 + Pinia (UI)                    ││
│  │ Worker      │  │  ├── CubeBridge (bridge.ts)            ││
│  │ (Workbox)   │  │  │   └── Web Bluetooth API             ││
│  │ Cache:      │  │  ├── Stores (auth, session, bluetooth) ││
│  │ - assets    │  │  └── Components                        ││
│  │ - .wasm     │  └────────────────────────────────────────┘│
│  │ - API       │           │                                 │
│  └─────────────┘           ▼ (single import)                │
│  ┌──────────────────────────────────────────────────────────┐│
│  │              rouxflow-wasm (cdylib)                       ││
│  │  #[wasm_bindgen] wrappers — re-exports everything        ││
│  │                                                           ││
│  │  ┌────────────────┐  ┌────────────────────────────────┐  ││
│  │  │ rouxflow-core  │  │ rouxflow-render                │  ││
│  │  │ - BLE decrypt  │  │ - 3D cube (three-d + WebGL2)   │  ││
│  │  │ - SessionMgr   │  │ - Animations                   │  ││
│  │  │ - Phases       │  │ - Gyro rotation                │  ││
│  │  └────────────────┘  └────────────────────────────────┘  ││
│  │  ┌─────────────────────┐  ┌───────────────────────────┐  ││
│  │  │ rouxflow-            │  │ rouxflow-storage          │  ││
│  │  │ bluetoothcube       │  │ - IndexedDB (rexie)       │  ││
│  │  │ - 28 cube models    │  │ - Supabase REST (reqwest) │  ││
│  │  │ - 9 protocols       │  │ - Sync (offline-first)    │  ││
│  │  │ - UUIDs & keys      │  │ - HMAC signing (TODO)     │  ││
│  │  └─────────────────────┘  └───────────────────────────┘  ││
│  └──────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
```

## Flux de données

### Connexion BLE
```
Web Bluetooth requestDevice() → GATT connect → startNotifications
    → characteristicvaluechanged event
    → CubeBridge.processRawPacket(bytes, deviceId)
    → WASM handle_ble_packet(data, deviceId, session)
    → Retourne CoreAction JSON (Move, Pickup, FlowStateChanged, etc.)
    → CubeBridge.handleCoreAction() → met à jour les stores Pinia
```

### Storage (offline-first)
```
Écriture:
    CubeBridge.saveCube() → WasmStorageManager.save_cube_json()
    → StorageManager.save_cube()
    → 1. IndexedDB (local, toujours)
    → 2. Supabase REST (best-effort, si réseau dispo)

Lecture:
    CubeBridge.getCubes() → WasmStorageManager.get_cubes_json()
    → StorageManager.get_cubes()
    → IndexedDB (toujours local-first)
```

## Crates Rust

| Crate | Type | Description |
|-------|------|-------------|
| `rouxflow-wasm` | `cdylib` | Point d'entrée WASM unique. Re-exporte tout via `#[wasm_bindgen]` |
| `rouxflow-core` | `rlib` | Logique cube pure Rust : protocole BLE, sessions, phases |
| `rouxflow-render` | `rlib` | Rendu 3D (three-d + WebGL2), boucle rAF |
| `rouxflow-bluetoothcube` | `rlib` | Registre de 28 cubes, 9 protocoles, UUIDs BLE |
| `rouxflow-storage` | `rlib` | IndexedDB + Supabase + sync (WASM only) |
| `rouxflow-ai` | `rlib` | Solveur Roux (futur) |
| `rouxflow-bitboard` | `rlib` | Représentation binaire (expérimental) |
