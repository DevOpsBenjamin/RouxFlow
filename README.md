# 🧊 RouxFlow

> A modern Bluetooth smart cube training platform dedicated to the **Roux Method** for speedcubing.

[![Build Check](https://github.com/DevOpsBenjamin/RouxFlow/actions/workflows/ci.yml/badge.svg)](https://github.com/DevOpsBenjamin/RouxFlow/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

---

### 📢 Project Status: Paused ATM — Real Users, Please Say Hello!

RouxFlow is currently on pause while the maintainer focuses on other priorities. However, **Cloudflare analytics show steady recurring connections and traffic**!

If you are a **real human cuber** using RouxFlow (and not just an automated scraper or bot), **please let us know**:
- ⭐ **Star this repository** on GitHub.
- 💬 **[Open a GitHub Issue or Discussion](https://github.com/DevOpsBenjamin/RouxFlow/issues)** to share your feedback, report a bug, or request cube support.
- ✉️ **Send an email to [contact@rouxflow.app](mailto:contact@rouxflow.app)**.

Knowing that real people are actively training with RouxFlow will directly help reprioritize and resume active development during spare time!

---

## 🚀 Key Features

- **Phase Split Analysis**: Real-time detection of First Block (FB), Second Block (SB), CMLL, and Last Six Edges (LSE).
- **LSE Breakdown**: Sub-phase granularity across EO, UL/UR, and L4E.
- **Smart Cube Ecosystem**: Support for multiple protocols (QiYi, MoYu AI, GiiKER, GoCube, GAN BLE).
- **Offline-First PWA**: Runs fully in the browser via WebAssembly (Rust) and IndexedDB, with optional cloud sync.
- **Hardware Integration**: Web Bluetooth API with gyro orientation tracking and snap detection.

## 🛠️ Tech Stack

- **Core Logic & Engine**: Rust compiled to WASM (`crates/rouxflow-wasm`)
- **Frontend**: Vue 3 + TypeScript + TailwindCSS v4 + Vite PWA
- **Database & Auth**: Supabase (PostgreSQL) + IndexedDB (offline-first via `rexie`)
- **3D Graphics**: `three-d` in WebAssembly

## 🗺️ Roadmap & Documentation

- [Roadmap & Todo List](TODO.md)
- [Architecture Overview](ARCHITECTURE.md)
- [Supported Smart Cubes](crates/rouxflow-bluetoothcube/SUPPORTED_CUBES.md)
- [Terms of Service](CGU.md)
- [Database Schema](DATABASE.md)

## 📬 Contact & Support

- **Email**: [contact@rouxflow.app](mailto:contact@rouxflow.app)
- **Issues & Bug Reports**: [GitHub Issues](https://github.com/DevOpsBenjamin/RouxFlow/issues)

---

*License: [MIT](./LICENSE) — Built with 🦀 and 🧊 by DevOpsBen.*
