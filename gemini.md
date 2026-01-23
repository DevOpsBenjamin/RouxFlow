# Gemini Project Context: RouxFlow 🧊

This file serves as a persistent context memory for Gemini (AI Assistant) to ensure alignment with project goals and technical preferences.

## 🎯 Project Overview
**RouxFlow** is a dedicated training platform for the **Roux Method**. It bridges the gap between raw data from Bluetooth cubes and actionable, Roux-specific insights.

## 🛠️ Technical Preferences
- **Package Manager**: **pnpm** (always use pnpm over npm/yarn).
- **Rust Core**: Logic lives in `crates/roux-core`. 
- **Efficiency First**: Use `cargo check` during development to avoid full compilation overhead where possible.
- **Frontend**: Vue 3 + Tailwind + Pinia in `apps/frontend`.
- **Desktop**: Tauri wrapper for native performance/BT access.
- **WASM**: Core logic is shared with the web via WASM.

## 🧠 Brain/Context
- **The "Roux-First" mindset**: Always prioritize Roux metrics (FB, SB, CMLL, LSE) over generic speedcubing metrics.
- **LSE focus**: The platform should excel at LSE sub-phase analysis (EO, UL/UR, L4E).
- **Human-First Solver**: Solutions provided should be human-executable Roux solutions, not just shortest-path God algorithms.

## 🏃 Current Strategy
1.  **Monorepo initialization**: Standardizing the structure.
2.  **Vue 3 Foundation**: Building the timer and BLE bridge.
3.  **Rust Core**: Porting/Writing the cube engine.
