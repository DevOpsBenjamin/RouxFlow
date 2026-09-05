//! # MoYu AI Protocol
//!
//! Used by MoYu cubes that connect via the GAN Gen2 BLE service but with
//! different encryption keys.
//!
//! ## Supported cubes
//! - MoYu AI 2023 (BLE prefix: `AiCube`)
//! - MoYu AI v2 (BLE prefix: `MHC`)
//!
//! ## Encryption
//! Same AES-128-CBC scheme as GAN Gen2 (MAC-salted, first/last 16-byte chunks),
//! but using MoYu-specific base keys.
//!
//! ## BLE
//! Reuses the GAN Gen2 BLE service and characteristics.
//! Detected at connection time by the BLE device name prefix: if the name
//! starts with `"AiCube"` or `"MHC"`, these keys are used instead of the
//! standard GAN keys.
//!
//! ## Packet format
//! Identical to GAN Gen2 (same opcodes, same bit layout).

use super::EncryptionKeys;

/// MoYu AI AES-128-CBC encryption keys.
pub const ENCRYPTION_KEYS: EncryptionKeys = EncryptionKeys {
    key: [0x05, 0x12, 0x02, 0x45, 0x02, 0x01, 0x29, 0x56,
          0x12, 0x78, 0x12, 0x76, 0x81, 0x01, 0x08, 0x03],
    iv:  [0x01, 0x44, 0x28, 0x06, 0x86, 0x21, 0x22, 0x28,
          0x51, 0x05, 0x08, 0x31, 0x82, 0x02, 0x21, 0x06],
};

/// BLE profile: reuses GAN Gen2, re-exported for convenience.
pub use super::gan_v2::BLE_PROFILE;
