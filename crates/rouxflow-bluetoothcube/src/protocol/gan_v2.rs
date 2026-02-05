//! # GAN Gen2 Protocol
//!
//! Second generation protocol, used by the majority of current GAN cubes.
//!
//! ## Supported cubes
//! - GAN Mini ui FreePlay
//! - GAN 12 ui FreePlay / GAN 12 ui
//! - GAN 356 i Carry / Carry S
//! - GAN 356 i 3
//! - Monster Go 3Ai (with MoYu AI keys, see `moyu_ai` module)
//!
//! ## Encryption
//! AES-128-CBC. The first 6 bytes of the base key and IV are salted by adding
//! the 6 bytes of the device MAC address (reversed) modulo 255:
//! ```text
//! salted_key[i] = (base_key[i] + mac_reversed[i]) % 255   for i in 0..6
//! salted_iv[i]  = (base_iv[i]  + mac_reversed[i]) % 255   for i in 0..6
//! ```
//!
//! Only the first and last 16-byte chunks of each message are encrypted.
//! Decryption order: last chunk first, then first chunk.
//!
//! ## Packet format
//! 20-byte bit-packed messages. The first 4 bits identify the event type.
//!
//! | Event type | Opcode | Description                      |
//! |------------|--------|----------------------------------|
//! | GYRO       | `0x01` | Quaternion + angular velocity    |
//! | MOVE       | `0x02` | Up to 7 moves + timestamps       |
//! | FACELETS   | `0x04` | Full cube state (CP/CO/EP/EO)    |
//! | HARDWARE   | `0x05` | HW/SW version, gyro support flag |
//! | BATTERY    | `0x09` | Battery level (0-100%)           |
//! | DISCONNECT | `0x0D` | Cube-initiated disconnect        |
//!
//! ## Commands
//! | Command          | Byte 0 |
//! |------------------|--------|
//! | Request facelets | `0x04` |
//! | Request hardware | `0x05` |
//! | Request battery  | `0x09` |
//! | Reset cube       | `0x0A` |

use super::{BleProfile, EncryptionKeys};

/// BLE profile for GAN Gen2 cubes.
pub const BLE_PROFILE: BleProfile = BleProfile {
    service_uuid:           "6e400001-b5a3-f393-e0a9-e50e24dc4179",
    command_characteristic: "28be4a4a-cd67-11e9-a32f-2a2ae2dbcce4",
    state_characteristic:   "28be4cb6-cd67-11e9-a32f-2a2ae2dbcce4",
};

/// AES-128-CBC encryption keys (shared with Gen3 and Gen4).
pub const ENCRYPTION_KEYS: EncryptionKeys = EncryptionKeys {
    key: [0x01, 0x02, 0x42, 0x28, 0x31, 0x91, 0x16, 0x07,
          0x20, 0x05, 0x18, 0x54, 0x42, 0x11, 0x12, 0x53],
    iv:  [0x11, 0x03, 0x32, 0x28, 0x21, 0x01, 0x76, 0x27,
          0x20, 0x95, 0x78, 0x14, 0x32, 0x12, 0x02, 0x43],
};

// Event opcodes (first 4 bits of decrypted message)
pub const EVENT_GYRO: u8 = 0x01;
pub const EVENT_MOVE: u8 = 0x02;
pub const EVENT_FACELETS: u8 = 0x04;
pub const EVENT_HARDWARE: u8 = 0x05;
pub const EVENT_BATTERY: u8 = 0x09;
pub const EVENT_DISCONNECT: u8 = 0x0D;

// Command opcodes (byte 0 of command message)
pub const CMD_REQUEST_FACELETS: u8 = 0x04;
pub const CMD_REQUEST_HARDWARE: u8 = 0x05;
pub const CMD_REQUEST_BATTERY: u8 = 0x09;
pub const CMD_REQUEST_RESET: u8 = 0x0A;
