# 🧊 RouxFlow: MVP 1 & 2 Checklist

Focused on Establishing the Bluetooth bridge and real-time state synchronization first.

## 🌐 MVP 1: Bluetooth & State Sync (Web)
- [ ] **Bluetooth Integration**
  - [ ] Research and implement `gan-web-bluetooth` compatibility (MoYu AI / GAN v2).
  - [ ] Implement discovery and pairing flow in the UI.
  - [ ] Handle state synchronization (detecting mixed state on connect).
- [ ] **Move & State Bridge**
  - [ ] Map raw BLE data to human-readable moves.
  - [ ] Track virtual cube state in the frontend (Pinia).
- [ ] **Basic Timer UI**
  - [ ] Auto-trigger timer start on first move.
  - [ ] Real-time move list display.
  - [ ] **Option**: Gyro-based session timing (Pick-up / Put-down detection).

## 🦀 MVP 2: Rust Core & Phase Detection (WASM)
- [ ] **Cube Representation**
  - [ ] Implement bitpacked/binary cube state in Rust.
  - [ ] WASM bridge for high-performance phase detection.
- [ ] **Roux Phase Detection Logic**
  - [ ] Implement M-slice software inference (Gyro and Timing paths).
  - [ ] Boolean checks for FB, SB, CMLL, and LSE.
  - [ ] Real-time splits triggered via WASM.

## 🎨 UI/UX Polish
- [ ] Implement Cube visualizer (3D or 2D net).
- [ ] Responsive layout for mobile/tablet.
