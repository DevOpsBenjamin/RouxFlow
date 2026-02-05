# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

RouxFlow is a Bluetooth-connected training platform for the Roux method of speedcubing. The project targets both web and desktop platforms using a shared Rust core compiled to WASM for web and native for desktop (via Tauri).

**Tech Stack:**
- **Core Logic:** Rust (compiled to WASM for web, native for desktop)
- **Frontend:** Vue 3 + TypeScript + TailwindCSS v4
- **Desktop:** Tauri v2
- **Storage:** SQLite (desktop local), Supabase (cloud sync)
- **Bluetooth:** Web Bluetooth API (web), btleplug (desktop native)

## Development Commands

### WASM Building

Build WASM modules from Rust crates for web use:

```bash
# Build individual WASM modules
pnpm --filter frontend run build:wasm:core    # rouxflow-core
pnpm --filter frontend run build:wasm:cloud   # rouxflow-storage-cloud
pnpm --filter frontend run build:wasm:render  # rouxflow-render

# Build all WASM modules
pnpm --filter frontend run build:wasm
```

WASM outputs are written to `apps/frontend/src/wasm/<crate-name>/` and should not be committed (see .gitignore).

### Frontend Development

```bash
# Development server (auto-builds WASM first)
pnpm --filter frontend dev

# Production build
pnpm --filter frontend build

# Type checking
pnpm --filter frontend exec vue-tsc -b
```

### Tauri (Desktop) Development

```bash
# Run desktop app in development
pnpm tauri dev

# Build desktop app for production
pnpm tauri build

# Build Rust backend only (no frontend rebuild)
cd src-tauri && cargo build
```

### Rust Development

```bash
# Build all Rust crates
cargo build --workspace

# Build specific crate
cargo build -p rouxflow-core

# Run tests
cargo test --workspace

# Run tests for specific crate
cargo test -p rouxflow-core
```

## Architecture

### Crate Organization

RouxFlow uses a **pnpm workspace** for frontend packages and a **Cargo workspace** (implicitly via path dependencies) for Rust crates.

**Rust Crates** (`crates/`):
- `rouxflow-core`: Core cube logic (binary state, move parsing, phase detection, BLE protocol handlers)
- `rouxflow-storage-sqlite`: SQLite storage implementation (local desktop storage)
- `rouxflow-storage-cloud`: Cloud storage implementation (Supabase API client)
- `rouxflow-render`: 3D cube rendering logic (WASM for web)
- `rouxflow-bitboard`: Experimental binary cube representation optimizations
- `rouxflow-ai`: AI/solver logic for phase-optimal solutions
- `rouxflow-standalone`: Standalone test utilities
- `rouxflow-bt-test`: Bluetooth testing utilities

**Applications**:
- `apps/frontend`: Vue 3 web app (also embedded in Tauri desktop)
- `src-tauri`: Tauri desktop wrapper with native Bluetooth (btleplug) and SQLite

### Key Architectural Patterns

**Storage Trait Pattern:**
Both `rouxflow-storage-sqlite` and `rouxflow-storage-cloud` implement a `Storage` trait defined in `rouxflow-core`. The desktop app uses SQLite for local storage and syncs with cloud storage. The web app uses cloud storage directly.

**WASM Compilation:**
The frontend depends on WASM-compiled versions of:
- `rouxflow-core` (cube logic, BLE protocol)
- `rouxflow-storage-cloud` (Supabase client)
- `rouxflow-render` (3D rendering)

These must be rebuilt via `wasm-pack` after modifying Rust code. The build outputs TypeScript bindings automatically.

**Bluetooth Protocol Handling:**
`rouxflow-core/src/lib.rs` exports:
- `handle_ble_packet()`: Unified BLE packet decoder supporting GAN and MoYu cubes (encrypted and unencrypted)
- `encode_cube_command()`: Command encoder for requesting cube state/hardware info
- Protocol constants: `GAN_KEY`, `GAN_IV`, `MOYU_KEY`, `MOYU_IV`

The core uses salted AES-CBC encryption with device-specific IVs for GAN/MoYu protocol v2.

**Session Management:**
`SessionManager` in `rouxflow-core/src/session.rs` handles:
- Scramble validation (tracks expected moves vs actual moves)
- Timer state (idle → scrambling → solving → complete)
- Orientation processing (quaternion from gyroscope)
- Move recording and phase transitions

## Important Notes for Claude Code

### WASM Build Requirements

**After modifying any Rust crate used by the web frontend**, you MUST rebuild WASM:
```bash
pnpm --filter frontend run build:wasm
```

The WASM build uses `wasm-pack build --target web`, which outputs:
- `.wasm` binary
- TypeScript bindings (`.d.ts`)
- JavaScript glue code

These files are in `.gitignore` under `**/wasm/*/` (except `.ts` files).

### Bluetooth Protocol Specifics

- **GAN cubes**: Use AES-CBC encrypted protocol with device-salted keys
- **MoYu cubes**: Support both encrypted (v2) and raw unencrypted packets
- Device ID is used to salt the encryption IV (see `GanV2Protocol::new()`)
- All BLE packets go through `handle_ble_packet()` which tries decryption cascades

### Tauri Desktop Integration

The Tauri app (`src-tauri/`) depends on:
- `rouxflow-core` (as Rust library, not WASM)
- `rouxflow-storage-sqlite` (native rusqlite)
- `rouxflow-storage-cloud` (native reqwest)

It embeds the same Vue frontend from `apps/frontend/dist/` after building.

### Frontend Service Layer

The frontend uses:
- **Stores** (Pinia): `apps/frontend/src/stores/` for app state
- **Services**: `apps/frontend/src/services/` for WASM bridge and Bluetooth
- **Components**: `apps/frontend/src/components/` organized by feature (cube/, layout/, session/)

### TailwindCSS v4

The project uses **TailwindCSS v4** (via `@tailwindcss/vite` plugin). Configuration is in CSS imports, not `tailwind.config.js`.

### Development Workflow

1. **Rust changes** → rebuild WASM → restart Vite dev server
2. **Vue/TS changes** → Vite HMR (no rebuild needed)
3. **Tauri changes** → `pnpm tauri dev` (rebuilds Rust + frontend)

### Testing

Currently tests are primarily in Rust crates. Frontend has minimal test coverage. When adding features, prioritize Rust unit tests for core logic.

## Current Development Focus

See `ROADMAP.md` for full feature list. Current focus areas:
- Phase detection (FB, SB, CMLL, LSE sub-phases)
- Roux-optimal solver (phase-by-phase, not God-mode)
- LSE granular tracking (EO, UL/UR, L4E separately)

## File Structure

```
RouxFlow/
├── apps/
│   └── frontend/              # Vue 3 web app
│       ├── src/
│       │   ├── components/    # Vue components
│       │   ├── stores/        # Pinia state stores
│       │   ├── services/      # WASM bridge, Bluetooth services
│       │   └── wasm/          # WASM outputs (gitignored, built from crates)
│       └── package.json
├── crates/                    # Rust workspace
│   ├── rouxflow-core/         # Core cube logic, BLE protocol
│   ├── rouxflow-storage-sqlite/
│   ├── rouxflow-storage-cloud/
│   ├── rouxflow-render/
│   ├── rouxflow-bitboard/
│   ├── rouxflow-ai/
│   └── ...
├── src-tauri/                 # Tauri desktop app
│   ├── src/                   # Rust desktop entry point
│   └── Cargo.toml
├── package.json               # Workspace root
├── pnpm-workspace.yaml
└── ARCHITECTURE.md            # Detailed architecture (in French)
```
