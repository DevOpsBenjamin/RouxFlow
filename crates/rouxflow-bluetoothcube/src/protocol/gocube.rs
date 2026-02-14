//! # GoCube / Rubik's Connected Protocol
//!
//! Nordic UART-based protocol. **No encryption.**
//!
//! ## Supported cubes
//! - GoCube (prefix: `GoCube`)
//! - GoCube X (prefix: `GoCubeX`)
//! - Rubik's Connected (prefix: `Rubiks`)
//!
//! ## BLE
//! Uses the standard Nordic UART Service (NUS) UUID scheme, but with a
//! different base UUID than the GAN Gen2 service (note the `ca9e` suffix
//! vs GAN's `4179`).
//!
//! ## Packet format
//! Plaintext packets. Supports:
//! - Turn detection and reporting
//! - Battery level status
//! - Rotation tracking enable/disable
//! - Reset to solved state

use super::BleProfile;

/// BLE profile for GoCube / Rubik's Connected (Nordic UART).
/// NUS convention: `..0002` = RX (host writes), `..0003` = TX (host subscribes).
pub const BLE_PROFILE: BleProfile = BleProfile {
    service_uuid:           "6e400001-b5a3-f393-e0a9-e50e24dcca9e",
    command_characteristic: "6e400002-b5a3-f393-e0a9-e50e24dcca9e",
    state_characteristic:   "6e400003-b5a3-f393-e0a9-e50e24dcca9e",
};
