//! # QiYi Smart Protocol
//!
//! Protocol used by QiYi smart cubes, using AES-ECB encryption.
//!
//! ## Supported cubes
//! - QiYi Tornado V4 SCS (prefix: `QY-QYSC`)
//! - QiYi Tornado V4 AI (prefix: `XMD-TornadoV4-i-`)
//!
//! ## Encryption
//! AES-128-ECB with a fixed 16-byte key (no IV needed for ECB mode).
//! Messages are padded to 16-byte alignment before encryption.
//!
//! ## Handshake
//! Requires an `appHello()` greeting after connection, with model-specific
//! greeting bytes derived from the device ID:
//! - SCS model: `[0xCC, 0xA3, 0x00, 0x00, device_id_hi, device_id_lo, ...]`
//! - AI model:  `[0xCC, 0xA6, 0x00, 0x00, device_id_hi, device_id_lo, ...]`
//!
//! The device ID is parsed from the last 4 hex characters of the BLE name.
//!
//! ## Packet format
//! Messages start with `0xFE`, followed by length and message type.
//!
//! | Message type | Description                      |
//! |--------------|----------------------------------|
//! | `0x02`       | CubeHello response (38 bytes)    |
//! | `0x03`       | State event                      |
//! | `0x04`       | Turn/move event                  |
//! | `0x0A`       | Battery / ACK                    |
//!
//! Certain messages require an ACK response from the host.
//!
//! ## Move encoding
//! Face: `L=0, R=1, D=2, U=3, F=4, B=5`.
//! Direction: `CW=1, CCW=-1, 180°=2`.

use super::BleProfile;

/// BLE profile for QiYi cubes.
pub const BLE_PROFILE: BleProfile = BleProfile {
    service_uuid:           "0000fff0-0000-1000-8000-00805f9b34fb",
    state_characteristic:   "0000fff6-0000-1000-8000-00805f9b34fb",
    command_characteristic: "0000fff6-0000-1000-8000-00805f9b34fb",
};

/// AES-128-ECB encryption key (no IV for ECB mode).
pub const ENCRYPTION_KEY: [u8; 16] = [87, 177, 249, 171, 205, 90, 232, 167, 156, 185, 140, 231, 87, 140, 81, 8];

/// Message start marker.
pub const MESSAGE_START: u8 = 0xFE;

/// Greeting key for SCS model.
pub const GREETING_SCS: [u8; 4] = [0xCC, 0xA3, 0x00, 0x00];

/// Greeting key for AI model.
pub const GREETING_AI: [u8; 4] = [0xCC, 0xA6, 0x00, 0x00];

// Message types
pub const MSG_CUBE_HELLO: u8 = 0x02;
pub const MSG_STATE: u8 = 0x03;
pub const MSG_TURN: u8 = 0x04;
pub const MSG_BATTERY_ACK: u8 = 0x0A;
