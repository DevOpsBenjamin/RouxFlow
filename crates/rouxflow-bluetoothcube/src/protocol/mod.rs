//! Bluetooth smart cube protocol definitions.
//!
//! Each protocol is defined in its own module with BLE UUIDs, encryption
//! constants, and (future) trait implementations for packet encoding/decoding.
//!
//! See [`PROTOCOL.md`](../../PROTOCOL.md) for a human-readable description
//! of each protocol.

pub mod gan_v1;
pub mod gan_v2;
pub mod gan_v3;
pub mod gan_v4;
pub mod moyu_ai;
pub mod moyu_v3;
pub mod giiker;
pub mod gocube;
pub mod qiyi;

// Re-export per-protocol BLE profiles
pub use gan_v1::BLE_PROFILE as GAN_V1_BLE;
pub use gan_v2::BLE_PROFILE as GAN_V2_BLE;
pub use gan_v3::BLE_PROFILE as GAN_V3_BLE;
pub use gan_v4::BLE_PROFILE as GAN_V4_BLE;
pub use moyu_v3::BLE_PROFILE as MOYU_V3_BLE;
pub use giiker::BLE_PROFILE as GIIKER_BLE;
pub use gocube::BLE_PROFILE as GOCUBE_BLE;
pub use qiyi::BLE_PROFILE as QIYI_BLE;

// Re-export per-protocol encryption keys
pub use gan_v2::ENCRYPTION_KEYS as GAN_KEYS;
pub use moyu_ai::ENCRYPTION_KEYS as MOYU_AI_KEYS;
pub use moyu_v3::ENCRYPTION_KEYS as MOYU_V3_KEYS;
pub use qiyi::ENCRYPTION_KEY as QIYI_KEY;
pub use giiker::KEY_TABLE as GIIKER_KEY_TABLE;

// ---------------------------------------------------------------------------
// Shared types
// ---------------------------------------------------------------------------

/// Protocol version identifier.
///
/// Each variant maps to a specific encryption scheme, packet format,
/// and set of BLE service/characteristic UUIDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProtocolVersion {
    GanV1,
    GanV2,
    GanV3,
    GanV4,
    MoYuAi,
    MoYuV3,
    GiikerV1,
    GoCube,
    QiYiSmart,
}

/// BLE connection profile for a specific protocol.
#[derive(Debug, Clone)]
pub struct BleProfile {
    /// Primary BLE service UUID
    pub service_uuid: &'static str,
    /// Characteristic UUID for receiving notifications (state/moves)
    pub state_characteristic: &'static str,
    /// Characteristic UUID for writing commands
    pub command_characteristic: &'static str,
}

/// AES-128-CBC encryption key pair.
#[derive(Debug, Clone)]
pub struct EncryptionKeys {
    pub key: [u8; 16],
    pub iv: [u8; 16],
}

/// Encryption method used by a protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncryptionMethod {
    /// AES-128-CBC with MAC-address-salted key/IV.
    AesCbc,
    /// AES-128-CBC with double-pass (decrypt tail then head).
    AesCbcDoublePass,
    /// AES-128-ECB with a fixed key (no IV).
    AesEcb,
    /// Proprietary XOR/ADD with a lookup table.
    ProprietaryXor,
    /// No encryption (plaintext).
    None,
}

impl ProtocolVersion {
    /// Returns the BLE profile for this protocol.
    pub fn ble_profile(&self) -> &'static BleProfile {
        match self {
            Self::GanV1 => &GAN_V1_BLE,
            Self::GanV2 => &GAN_V2_BLE,
            Self::GanV3 => &GAN_V3_BLE,
            Self::GanV4 => &GAN_V4_BLE,
            Self::MoYuAi => &GAN_V2_BLE, // reuses GAN Gen2 service
            Self::MoYuV3 => &MOYU_V3_BLE,
            Self::GiikerV1 => &GIIKER_BLE,
            Self::GoCube => &GOCUBE_BLE,
            Self::QiYiSmart => &QIYI_BLE,
        }
    }

    /// Returns the AES-CBC encryption keys, if applicable.
    pub fn encryption_keys(&self) -> Option<&'static EncryptionKeys> {
        match self {
            Self::GanV1 | Self::GanV2 | Self::GanV3 | Self::GanV4 => Some(&GAN_KEYS),
            Self::MoYuAi => Some(&MOYU_AI_KEYS),
            Self::MoYuV3 => Some(&MOYU_V3_KEYS),
            Self::GiikerV1 | Self::GoCube | Self::QiYiSmart => None,
        }
    }

    /// Returns the encryption method used by this protocol.
    pub fn encryption_method(&self) -> EncryptionMethod {
        match self {
            Self::GanV1 | Self::GanV2 | Self::GanV3 | Self::GanV4 | Self::MoYuAi => EncryptionMethod::AesCbc,
            Self::MoYuV3 => EncryptionMethod::AesCbcDoublePass,
            Self::QiYiSmart => EncryptionMethod::AesEcb,
            Self::GiikerV1 => EncryptionMethod::ProprietaryXor,
            Self::GoCube => EncryptionMethod::None,
        }
    }

    /// Whether this protocol requires a handshake after BLE connection.
    pub fn requires_handshake(&self) -> bool {
        matches!(self, Self::MoYuV3 | Self::QiYiSmart)
    }

    /// Whether this protocol supports move history recovery.
    pub fn supports_move_history(&self) -> bool {
        matches!(self, Self::GanV3 | Self::GanV4)
    }
}
