# Gemini Project Context: RouxFlow 🧊

This file serves as a persistent context memory for Gemini (AI Assistant) to ensure alignment with project goals and technical preferences.

## 🎯 Project Overview
**RouxFlow** is a dedicated training platform for the **Roux Method**. It bridges the gap between raw data from Bluetooth cubes and actionable, Roux-specific insights.

## 🛠️ Technical Preferences
- **Package Manager**: **pnpm** (always use pnpm over npm/yarn).
- **Rust-First Architecture**: WASM/Rust is the source of truth. Move decryption, state tracking, and protocol decoding into `roux-core`.
- **Bluetooth Abstraction**: Use Rust `Trait` definitions for different cube types (MoYu, GAN, GoCube).
- **Protocol Specifics**:
  - **Frequency**: ~50Hz (20ms updates).
  - **Moves**: Discrete only (U, U', U2). No partial turn streaming.
  - **IMU**: 6-axis (Gyro/Accel) at ~50Hz. Essential for 3D state and M-slice inference.
  - **M-Slice Logic**:
    - **With Gyro**: Correlation of `L/R` moves + `IMU X-axis rotation` within ~30ms. Most accurate.
    - **Without Gyro**: Heuristic-only. `L/R` moves within ~15ms window are treated as `M`.
  - **Session Timing (Optional Gyro)**:
    - **Pick-up detection**: Trigger when IMU delta exceeds a threshold (start inspection -> start pick-up timer).
    - **Put-down detection**: Trigger when IMU delta stays below a threshold for >X ms (stop put-down timer).
- **WASM-First Architecture**: 100% of business logic (Timing, Roux Phases, Session Rules, Scramble Validation) MUST live in Rust/WASM.
- **Minimal TypeScript**: Frontend stores and components are thin presentation layers. Move processing into `roux-core`.
- **Atomic State**: All state transitions are calculated in Rust. TS only syncs the resulting UI model.
- **WASM**: Core logic is shared with the web via WASM.

## 🧠 Brain/Context
- **The "Roux-First" mindset**: Always prioritize Roux metrics (FB, SB, CMLL, LSE) over generic speedcubing metrics.
- **LSE focus**: The platform should excel at LSE sub-phase analysis (EO, UL/UR, L4E).
- **Human-First Solver**: Solutions provided should be human-executable Roux solutions, not just shortest-path God algorithms.

## 🏃 Current Strategy
1.  **Monorepo initialization**: Standardizing the structure.
2.  **Vue 3 Foundation**: Building the timer and BLE bridge.
3.  **Rust Core**: Porting/Writing the cube engine.
