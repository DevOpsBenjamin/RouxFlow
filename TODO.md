# 🧊 RouxFlow Roadmap

Focused on the "Core-First" architecture where Rust handles the engine and the Bridge orchestrates.

## ✅ Phase 1: Core-First Foundation
- [x] **Bridge-Orchestrated Architecture**: Pure Rust core + TypeScript orchestrator.
- [x] **Native Persistence**: SQLite backend for Tauri.
- [x] **Hybrid Bluetooth**: Unified bridge for Tauri (btleplug) and Web (Web Bluetooth).
- [x] **WCA Integrity**: 1-hour session limit and 5-solve validation in Rust.
- [x] **Scramble Tolerance**: 1-move mistake buffer and undo logic in Rust.

## 🚀 Phase 2: UX & Navigation (Next Steps)
- [x] **Landing Gate**: "Bluetooth-First" entry page.
- [ ] **Bluetooth Reliability** (Current Focus):
    - [ ] **Hardware**: Test with new antennas/dongle.
    - [ ] **Driver**: Debug `btleplug` connection stability.
    - [ ] **Protocol**: Robust subscription & auto-reconnect.
- [/] **Device Selection UI**:
    - [x] Basic UI implemented.
    - [ ] Real device scanner in Tauri (WIP).

## 🎨 Phase 4: Polish
- [x] **3D Visualization**: `rouxflow-render` crate (WASM) with `three-d`.
- [ ] **Benchmarking**: Compare solve splits against "Ideal Roux" efficiency stats.
