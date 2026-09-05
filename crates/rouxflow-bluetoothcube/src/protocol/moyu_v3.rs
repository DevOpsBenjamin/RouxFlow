//! # MoYu V3 Protocol
//!
//! Native MoYu protocol used by newer WeiLong cubes.
//!
//! ## Supported cubes
//! - MoYu WeiLong V10 (BLE prefix: `WCU_MY3`)
//!
//! ## Encryption
//! Uses the **same encryption as GAN Gen2**: AES-128-CBC with MAC-salted keys,
//! decrypt last-16 then first-16. Different base key/IV from GAN.
//!
//! ### Key derivation (same formula as GAN)
//! ```text
//! device_key[i] = (MASTER_KEY[i] + mac_reversed[i]) % 255   for i in 0..6
//! device_iv[i]  = (MASTER_IV[i]  + mac_reversed[i]) % 255   for i in 0..6
//! ```
//!
//! ## Packet format
//! 20-byte encrypted, bit-packed packets. Bits 0..8 = message type.
//!
//! | Type  | Description                     |
//! |-------|---------------------------------|
//! | `161` | Device info (0xA1)              |
//! | `163` | Full cube state / facelets (0xA3) |
//! | `164` | Battery level (0xA4)            |
//! | `165` | Move event (0xA5)               |
//! | `171` | Gyroscope quaternion (0xAB)     |
//!
//! ### Move event (0xA5, bit-packed)
//! - Bits 8..88: 5x 16-bit timestamps
//! - Bits 88..96: move counter (u8)
//! - Bits 96..121: 5x 5-bit moves
//! - Move encoding: `m >> 1` → face in "FBUDLR", `m & 1` → 0=CW, 1=CCW
//!
//! ### State (0xA3, bit-packed)
//! - Bits 8..152: 48x 3-bit stickers (6 faces in FBUDLR, 8 stickers each)
//! - Bits 152..160: move counter
//!
//! ### Gyroscope data (0xAB)
//! Bytes 1..17: four little-endian `f32` quaternion values.

use super::{BleProfile, EncryptionKeys};

/// BLE profile for MoYu V3 cubes.
pub const BLE_PROFILE: BleProfile = BleProfile {
    service_uuid:           "0783b03e-7735-b5a0-1760-a305d2795cb0",
    state_characteristic:   "0783b03e-7735-b5a0-1760-a305d2795cb1",
    command_characteristic: "0783b03e-7735-b5a0-1760-a305d2795cb2",
};

/// MoYu V3 master encryption keys (before MAC salting).
pub const ENCRYPTION_KEYS: EncryptionKeys = EncryptionKeys {
    key: [21, 119, 58, 92, 103, 14, 45, 31, 23, 103, 42, 19, 155, 103, 82, 87],
    iv:  [17, 35, 38, 37, 134, 42, 44, 59, 85, 6, 127, 49, 126, 103, 33, 87],
};

// Opcodes (byte 0 of decrypted packet)
pub const OPCODE_INFO: u8 = 0xA1;
pub const OPCODE_STATE: u8 = 0xA3;
pub const OPCODE_BATTERY: u8 = 0xA4;
pub const OPCODE_MOVE: u8 = 0xA5;
pub const OPCODE_GYRO: u8 = 0xAB;
