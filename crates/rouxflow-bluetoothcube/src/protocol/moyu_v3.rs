//! # MoYu V3 Protocol
//!
//! Native MoYu protocol used by newer WeiLong cubes.
//!
//! ## Supported cubes
//! - MoYu WeiLong V10 (BLE prefix: `WCU_MY`)
//!
//! ## Encryption
//! AES-128-CBC with **double-pass** decryption and MAC-salted keys.
//!
//! ### Key derivation
//! The 6-byte MAC address is reversed, then added modulo 255 to the first
//! 6 bytes of the master key and IV:
//! ```text
//! device_key[i] = (MASTER_KEY[i] + mac_reversed[i]) % 255   for i in 0..6
//! device_iv[i]  = (MASTER_IV[i]  + mac_reversed[i]) % 255   for i in 0..6
//! ```
//!
//! ### Double-pass decryption
//! Unlike GAN (which decrypts last-then-first), MoYu V3 decrypts:
//! 1. **Pass 1** — Decrypt 16 bytes at offset 4..20 (tail)
//! 2. **Pass 2** — Decrypt 16 bytes at offset 0..16 (head)
//!
//! Each pass uses a fresh AES-CBC context (no IV chaining across passes).
//!
//! ## Handshake
//! After connecting, a hello payload must be sent to the write characteristic
//! before the cube will start sending notifications.
//!
//! ## Packet format
//! 20-byte encrypted packets. The first byte (after decryption) is the opcode.
//!
//! | Opcode | Description                     |
//! |--------|---------------------------------|
//! | `0xA1` | Device info response            |
//! | `0xA3` | Full cube state (facelets)      |
//! | `0xA4` | Battery level                   |
//! | `0xA5` | Move event                      |
//! | `0xAB` | Gyroscope quaternion (f32 × 4)  |
//!
//! ### Gyroscope data (0xAB)
//! Bytes 1..17 contain four little-endian `f32` values representing the
//! orientation quaternion `[q0, q1, q2, q3]`.
//!
//! ### Move event (0xA5)
//! Face codes: `0=U, 1=D, 2=L, 3=R, 4=F, 5=B`.
//! Direction: `1=CW, 2=CCW, 3=Double`.

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
