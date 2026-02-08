# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

RouxFlow is a Bluetooth-connected PWA training platform for the Roux method of speedcubing. All logic runs in the browser via a single WASM module compiled from Rust.

**Tech Stack:**
- **Core Logic:** Rust (compiled to WASM via `rouxflow-wasm` entry point)
- **Frontend:** Vue 3 + TypeScript + TailwindCSS v4
- **Storage:** IndexedDB (local, offline-first via `rexie`) + Supabase (cloud sync)
- **Bluetooth:** Web Bluetooth API (Chrome/Edge only)
- **PWA:** `vite-plugin-pwa` with Workbox service worker

## Development Commands

### WASM Building

Single WASM module built from `rouxflow-wasm` entry point crate:

```bash
# Build WASM (outputs to apps/frontend/src/wasm/rouxflow/)
pnpm --filter frontend run build:wasm

# Or from root
pnpm build:wasm
```

WASM outputs are written to `apps/frontend/src/wasm/rouxflow/` and should not be committed (see .gitignore).

### Frontend Development

```bash
# Development server (auto-builds WASM first)
pnpm --filter frontend dev

# Production build (PWA with service worker)
pnpm --filter frontend build

# Type checking
pnpm --filter frontend exec vue-tsc -b
```

### Rust Development

```bash
# Build specific crate (no workspace Cargo.toml at root)
cd crates/rouxflow-core && cargo build

# Run tests for a crate
cd crates/rouxflow-core && cargo test
```

## Architecture

### Crate Organization

```
[Browser]
  ├── Service Worker (PWA, cache assets + .wasm)
  ├── Vue 3 + Pinia (UI only)
  ├── TypeScript (thin bridge layer)
  │   ├── Web Bluetooth API → passes packets to WASM
  │   └── CubeBridge (simplified, web-only)
  │
  └── rouxflow-wasm (SINGLE .wasm, entry point only)
      │   ↓ re-exports
      ├── rouxflow-core           — BLE decrypt, session mgmt, phases
      ├── rouxflow-render         — 3D cube (three-d + web-sys)
      ├── rouxflow-bluetoothcube  — cube registry, UUIDs, keys
      └── rouxflow-storage        — IndexedDB (local) + Supabase (cloud) + sync
```

**Rust Crates** (`crates/`):
- `rouxflow-wasm`: **Single WASM entry point** (`cdylib`). Re-exports from all other crates via `#[wasm_bindgen]` wrappers.
- `rouxflow-core`: Pure Rust lib. Core cube logic (BLE protocol, session management, phase detection). No `wasm-bindgen`.
- `rouxflow-render`: 3D cube rendering (three-d + web-sys). Pure Rust lib, WASM-specific code behind `#[cfg(target_arch = "wasm32")]`.
- `rouxflow-bluetoothcube`: Cube registry (28 models, 9 protocols, BLE UUIDs, encryption keys). Pure Rust, no deps.
- `rouxflow-storage`: Unified storage. IndexedDB (local, offline-first) + Supabase REST (cloud sync). WASM-only.
- `rouxflow-ai`: Roux solver (future, not in WASM yet)
- `rouxflow-bitboard`: Experimental cube representation (future)

### Key Architectural Patterns

**Single WASM Entry Point:**
Only `rouxflow-wasm` is compiled as `cdylib`. All other crates are `rlib` (pure Rust libraries). The frontend imports everything from one module:
```typescript
import init, { SessionManager, WasmStorageManager, handle_ble_packet, ... } from '../wasm/rouxflow/rouxflow_wasm'
```

**Offline-First Storage (StorageManager):**
`rouxflow-storage` implements the `Storage` trait from `rouxflow-core`:
- All writes go to IndexedDB first (via `rexie`)
- Cloud sync (Supabase) happens opportunistically
- If Supabase is unavailable, the app works normally with local data

**Bluetooth Protocol Handling:**
`rouxflow-core` exports `handle_ble_packet()` and `encode_cube_command()`. The WASM entry point wraps these. Protocol cascade: GAN encrypted → MoYu encrypted → MoYu raw.

**Session Management:**
`SessionManager` in `rouxflow-core/src/session.rs` handles scramble validation, timer state machine, orientation processing, and move recording.

## Important Notes for Claude Code

### WASM Build Requirements

**After modifying any Rust crate**, rebuild WASM:
```bash
pnpm --filter frontend run build:wasm
```

### No Workspace Cargo.toml

There is no root `Cargo.toml` workspace. Each crate is independent under `crates/`. Build from the crate directory: `cd crates/<name> && cargo build`.

### Frontend Service Layer

The frontend uses:
- **Stores** (Pinia): `apps/frontend/src/stores/` for app state
- **Services**: `apps/frontend/src/services/cube/bridge.ts` — single WASM bridge
- **Components**: `apps/frontend/src/components/` organized by feature

### TailwindCSS v4

Uses `@tailwindcss/vite` plugin. Configuration is in CSS imports, not `tailwind.config.js`.

### Development Workflow

1. **Rust changes** → `pnpm build:wasm` → restart Vite dev server
2. **Vue/TS changes** → Vite HMR (no rebuild needed)

### Testing

Tests are primarily in Rust crates. Frontend has minimal test coverage. Prioritize Rust unit tests for core logic.

## File Structure

```
RouxFlow/
├── apps/
│   └── frontend/              # Vue 3 PWA
│       ├── src/
│       │   ├── components/    # Vue components
│       │   ├── stores/        # Pinia state stores
│       │   ├── services/      # WASM bridge
│       │   └── wasm/          # WASM output (gitignored)
│       └── package.json
├── crates/
│   ├── rouxflow-wasm/         # WASM entry point (cdylib)
│   ├── rouxflow-core/         # Core cube logic (rlib)
│   ├── rouxflow-render/       # 3D rendering (rlib)
│   ├── rouxflow-bluetoothcube/# Cube registry (rlib)
│   ├── rouxflow-storage/      # IndexedDB + Supabase (rlib)
│   ├── rouxflow-ai/           # Solver (future)
│   ├── rouxflow-bitboard/     # Experimental (future)
├── package.json               # Workspace root
└── pnpm-workspace.yaml
```
