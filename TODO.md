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
- [/] **Device Selection UI**: Replace mocked connection with a real device scanner in Tauri.
    - [ ] Add `ble_list_devices` command to Rust backend.
    - [ ] Create `DeviceSelectionModal.vue` for the frontend.
    - [ ] Update `bridge.ts` to orchestrate multi-device selection.
- [ ] **WCA vs Free Session Flow**:
  - [ ] WCA: Strict scramble lock & teaser moves.
  - [ ] Free: Post-solve decision between "Analyze" or "Next".
- [ ] **Analysis Hub**:
  - [ ] Wire `SolveAnalysis.vue` to real bridge data.
  - [ ] History browser for previous sessions.

## � Phase 3: Roux Intelligence
- [ ] **Cube State Tracking**: Full bitpacked stickers representation in Rust.
- [ ] **M-Slice Inference**: Correctly mapping gym moves to Roux-specific phases.
- [ ] **Phase Detection**:
  - [ ] FB & SB boolean checks.
  - [ ] CMLL recognition.
  - [ ] LSE sub-phase timing (EO, UL/UR, L4E).

## 🎨 Phase 4: Polish
- [ ] **3D Visualization**: Integrating a three.js cube preview.
- [ ] **Benchmarking**: Compare solve splits against "Ideal Roux" efficiency stats.
