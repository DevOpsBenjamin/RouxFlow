//! # GAN Gen4 Protocol
//!
//! Fourth generation, used by the latest flagship GAN cubes.
//!
//! ## Supported cubes
//! - GAN 12 ui Maglev
//! - GAN 14 ui FreePlay
//!
//! ## Encryption
//! Identical to Gen2 (AES-128-CBC, same keys, same MAC salting).
//!
//! ## Packet format
//! 20-byte messages. Header: `[event_type, data_length, ...]`.
//! No magic byte (unlike Gen3).
//!
//! | Event type | Opcode     | Description                        |
//! |------------|------------|------------------------------------|
//! | MOVE       | `0x01`     | Single move + 32-bit timestamp     |
//! | MOVE_HIST  | `0xD1`     | Move history (recovery)            |
//! | FACELETS   | `0xED`     | Full cube state                    |
//! | GYRO       | `0xEC`     | Quaternion + angular velocity      |
//! | BATTERY    | `0xEF`     | Battery level                      |
//! | HARDWARE   | `0xFA-0xFE`| Multi-part hardware info           |
//! | DISCONNECT | `0xEA`     | Cube-initiated disconnect          |
//!
//! ## Multi-part hardware info
//! Hardware info is split across 4 event types:
//! - `0xFA` - Production date (year, month, day)
//! - `0xFC` - Hardware name (ASCII string, e.g. "GAN12uiM")
//! - `0xFD` - Software version (major.minor)
//! - `0xFE` - Hardware version (major.minor)
//!
//! Gyroscope support is determined by checking hardware name == `"GAN12uiM"`.
//!
//! ## Move history recovery
//! Same concept as Gen3, but using opcode `0xD1` and command `[0xD1, 0x04, serial, 0, count, 0]`.

use super::BleProfile;

/// BLE profile for GAN Gen4 cubes.
pub const BLE_PROFILE: BleProfile = BleProfile {
    service_uuid:           "00000010-0000-fff7-fff6-fff5fff4fff0",
    command_characteristic: "0000fff5-0000-1000-8000-00805f9b34fb",
    state_characteristic:   "0000fff6-0000-1000-8000-00805f9b34fb",
};

/// Encryption keys (same as Gen2).
pub use super::gan_v2::ENCRYPTION_KEYS;

// Event opcodes
pub const EVENT_MOVE: u8 = 0x01;
pub const EVENT_MOVE_HISTORY: u8 = 0xD1;
pub const EVENT_FACELETS: u8 = 0xED;
pub const EVENT_GYRO: u8 = 0xEC;
pub const EVENT_BATTERY: u8 = 0xEF;
pub const EVENT_DISCONNECT: u8 = 0xEA;

// Hardware info sub-opcodes
pub const EVENT_HW_PRODUCT_DATE: u8 = 0xFA;
pub const EVENT_HW_NAME: u8 = 0xFC;
pub const EVENT_HW_SW_VERSION: u8 = 0xFD;
pub const EVENT_HW_HW_VERSION: u8 = 0xFE;

// Commands
pub const CMD_REQUEST_FACELETS: [u8; 6] = [0xDD, 0x04, 0x00, 0xED, 0x00, 0x00];
pub const CMD_REQUEST_HARDWARE: [u8; 5] = [0xDF, 0x03, 0x00, 0x00, 0x00];
pub const CMD_REQUEST_BATTERY: [u8; 6] = [0xDD, 0x04, 0x00, 0xEF, 0x00, 0x00];
pub const CMD_REQUEST_MOVE_HISTORY_PREFIX: [u8; 2] = [0xD1, 0x04];
