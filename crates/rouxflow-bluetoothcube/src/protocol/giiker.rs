//! # Giiker (Xiaomi) Protocol
//!
//! Proprietary protocol used by Xiaomi Giiker smart cubes.
//!
//! ## Supported cubes
//! - Giiker i3 (prefix: `GiC`)
//! - Giiker i3S (prefix: `GiS`)
//! - Giiker i3Y (prefix: `Gi`)
//! - Mi Smart Magic Cube (prefix: `Mi Smart Magic Cube`)
//!
//! ## Encryption
//! Proprietary XOR/ADD scheme using a 36-byte lookup table.
//!
//! ### Decryption algorithm
//! 1. Check byte 18 == `0xA7` (167) to confirm the packet is encrypted.
//! 2. Extract two 4-bit nibbles from byte 19 as key offsets.
//! 3. For each of the first 18 bytes:
//!    `decrypted[i] = encrypted[i] + table[i + offset1] + table[i + offset2]`
//!
//! ## Packet format
//! 20-byte packets via the Turn characteristic.
//!
//! ### Move encoding
//! | Value | Face+Dir |
//! |-------|----------|
//! | 0     | D CW     |
//! | 1     | D CCW    |
//! | 2     | U CW     |
//! | 3     | U CCW    |
//! | 4     | B CW     |
//! | 5     | B CCW    |
//! | 6     | F CW     |
//! | 7     | F CCW    |
//! | 8     | L CW     |
//! | 9     | L CCW    |
//! | 10    | R CW     |
//! | 11    | R CCW    |

use super::BleProfile;

/// BLE profile for Giiker cubes.
pub const BLE_PROFILE: BleProfile = BleProfile {
    service_uuid:           "0000aadb-0000-1000-8000-00805f9b34fb",
    state_characteristic:   "0000aadc-0000-1000-8000-00805f9b34fb",
    command_characteristic: "0000aaab-0000-1000-8000-00805f9b34fb",
};

/// Additional request/response service UUID.
pub const REQUEST_SERVICE: &str = "0000aaaa-0000-1000-8000-00805f9b34fb";

/// XOR/ADD decryption lookup table (36 bytes).
pub const KEY_TABLE: [u8; 36] = [
    176, 81, 104, 224, 86, 137, 237, 119, 38, 26, 193, 161, 210, 126, 150, 81,
    93, 13, 236, 249, 89, 235, 88, 24, 113, 81, 214, 131, 130, 199, 2, 169,
    39, 165, 171, 41,
];

/// Marker byte indicating the packet is encrypted (byte 18).
pub const ENCRYPTION_MARKER: u8 = 0xA7;
