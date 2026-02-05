//! # GAN v1 Protocol
//!
//! Original protocol used by the first generation of GAN smart cubes.
//!
//! ## Supported cubes
//! - GAN 356i (China & International)
//! - GAN 356i Play
//! - GAN 356i 2 / 2 Play
//!
//! ## Encryption
//! AES-128-CBC with MAC-address-salted key/IV (same keys as Gen2/3/4).
//!
//! ## BLE characteristics
//! Uses a single service (`0000fff0`) with dedicated characteristics for
//! facelet status, state, battery, and interval control.

use super::BleProfile;

/// BLE profile for GAN v1 cubes.
pub const BLE_PROFILE: BleProfile = BleProfile {
    service_uuid:           "0000fff0-0000-1000-8000-00805f9b34fb",
    state_characteristic:   "0000fff5-0000-1000-8000-00805f9b34fb",
    command_characteristic: "0000fff5-0000-1000-8000-00805f9b34fb",
};

/// Additional characteristic: facelet status (notify).
pub const FACELET_STATUS_CHARACTERISTIC: &str = "0000fff2-0000-1000-8000-00805f9b34fb";

/// Additional characteristic: battery level (notify).
pub const BATTERY_CHARACTERISTIC: &str = "0000fff7-0000-1000-8000-00805f9b34fb";

/// Device information service UUID.
pub const DEVICE_INFO_SERVICE: &str = "0000180a-0000-1000-8000-00805f9b34fb";

/// Encryption keys (shared with GAN Gen2/3/4).
pub use super::gan_v2::ENCRYPTION_KEYS;
