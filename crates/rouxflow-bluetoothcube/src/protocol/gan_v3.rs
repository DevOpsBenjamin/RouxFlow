//! # GAN Gen3 Protocol
//!
//! Third generation, introduced with the GAN 356 i Carry 2.
//!
//! ## Supported cubes
//! - GAN 356 i Carry 2
//!
//! ## Encryption
//! Identical to Gen2 (AES-128-CBC, same keys, same MAC salting).
//!
//! ## Packet format
//! 16-byte messages with a header: `[magic, event_type, data_length, ...]`.
//! Magic byte is `0x55`.
//!
//! | Event type | Opcode | Description               |
//! |------------|--------|---------------------------|
//! | MOVE       | `0x01` | Single move + timestamp   |
//! | FACELETS   | `0x02` | Full cube state           |
//! | MOVE_HIST  | `0x06` | Move history (recovery)   |
//! | HARDWARE   | `0x07` | HW/SW version             |
//! | BATTERY    | `0x10` | Battery level             |
//! | DISCONNECT | `0x11` | Cube-initiated disconnect |
//!
//! ## Move history recovery
//! Gen3 adds a move history buffer. When a gap is detected in serial numbers,
//! the host can request missed moves via `[0x68, 0x03, serial, 0, count, 0]`.
//! The cube responds with a MOVE_HISTORY packet containing 2 moves per byte.
//!
//! ## Known firmware bug
//! Requesting move history that crosses the serial 255 -> 0 boundary may
//! return spoofed 'D' moves (zero bytes). The implementation clamps the
//! request window to avoid this.

use super::BleProfile;

/// BLE profile for GAN Gen3 cubes.
pub const BLE_PROFILE: BleProfile = BleProfile {
    service_uuid:           "8653000a-43e6-47b7-9cb0-5fc21d4ae340",
    command_characteristic: "8653000c-43e6-47b7-9cb0-5fc21d4ae340",
    state_characteristic:   "8653000b-43e6-47b7-9cb0-5fc21d4ae340",
};

/// Encryption keys (same as Gen2).
pub use super::gan_v2::ENCRYPTION_KEYS;

/// Magic byte at start of every Gen3 packet.
pub const MAGIC: u8 = 0x55;

// Event opcodes (byte 1 after magic)
pub const EVENT_MOVE: u8 = 0x01;
pub const EVENT_FACELETS: u8 = 0x02;
pub const EVENT_MOVE_HISTORY: u8 = 0x06;
pub const EVENT_HARDWARE: u8 = 0x07;
pub const EVENT_BATTERY: u8 = 0x10;
pub const EVENT_DISCONNECT: u8 = 0x11;

// Command prefixes
pub const CMD_PREFIX: u8 = 0x68;
pub const CMD_REQUEST_FACELETS: [u8; 2] = [0x68, 0x01];
pub const CMD_REQUEST_MOVE_HISTORY: [u8; 2] = [0x68, 0x03];
pub const CMD_REQUEST_HARDWARE: [u8; 2] = [0x68, 0x04];
pub const CMD_REQUEST_RESET: [u8; 2] = [0x68, 0x05];
pub const CMD_REQUEST_BATTERY: [u8; 2] = [0x68, 0x07];
