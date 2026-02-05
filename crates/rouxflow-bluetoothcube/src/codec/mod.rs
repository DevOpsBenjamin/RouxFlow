//! Cube protocol codec implementations.
//!
//! Each protocol (GAN Gen2/3/4, MoYu V3, etc.) implements the [`CubeProtocol`]
//! trait for encrypting, decrypting, and decoding BLE packets.

pub mod gan_v2;
pub mod gan_v3;
pub mod gan_v4;
pub mod moyu_v3;
pub mod giiker;
pub mod gocube;
pub mod qiyi;

use rouxflow_core::cube::{Face, Quaternion};
use crate::protocol::ProtocolVersion;

// ---------------------------------------------------------------------------
// Cube events (decoded from BLE packets)
// ---------------------------------------------------------------------------

/// Angular velocity by axes.
#[derive(Debug, Clone, Copy)]
pub struct AngularVelocity {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// An event decoded from a BLE notification packet.
#[derive(Debug, Clone)]
pub enum CubeEvent {
    /// A face turn detected by the cube.
    Move {
        serial: u16,
        face: Face,
        direction: i8,
        cube_timestamp: Option<u32>,
    },
    /// Full cube state (corner/edge permutation and orientation).
    Facelets {
        serial: u16,
        cp: [u8; 8],
        co: [u8; 8],
        ep: [u8; 12],
        eo: [u8; 12],
    },
    /// Gyroscope orientation + optional angular velocity.
    Gyro {
        quaternion: Quaternion,
        velocity: Option<AngularVelocity>,
    },
    /// Battery level (0-100%).
    Battery {
        level: u8,
    },
    /// Hardware information.
    Hardware {
        name: String,
        sw_version: String,
        hw_version: String,
        gyro_supported: bool,
    },
    /// Cube-initiated disconnect.
    Disconnect,
    /// Move history response (Gen3/Gen4) — contains recovered moves.
    /// The protocol driver handles injecting these into the buffer internally;
    /// this variant is used so the caller can see them after eviction.
    MoveHistory {
        moves: Vec<CubeEvent>,
    },
    /// Raw facelet state (54-char string, URFDLB face order).
    /// Used by protocols that provide facelets directly (MoYu, GoCube, QiYi).
    RawFacelets {
        facelet_string: String,
    },
    /// Protocol-initiated write-back response that should be sent to the cube.
    /// The data is already encrypted and ready to send (e.g. QiYi ACK).
    WriteBack {
        data: Vec<u8>,
    },
}

// ---------------------------------------------------------------------------
// Cube commands (to send to the cube)
// ---------------------------------------------------------------------------

/// A command to send to the cube.
#[derive(Debug, Clone, Copy)]
pub enum CubeCommand {
    RequestFacelets,
    RequestHardware,
    RequestBattery,
    RequestReset,
}

// ---------------------------------------------------------------------------
// CubeProtocol trait
// ---------------------------------------------------------------------------

/// Protocol handler for a specific cube model/generation.
///
/// Handles encryption, decryption, packet decoding, and command encoding.
/// `decode_event` takes `&mut self` because some protocols track serial
/// numbers as state. Returns `Vec` because Gen2 can decode up to 7 moves
/// from a single packet.
pub trait CubeProtocol: Send {
    fn name(&self) -> &str;
    fn decrypt(&self, data: &[u8]) -> Vec<u8>;
    fn encrypt(&self, data: &[u8]) -> Vec<u8>;
    fn decode_event(&mut self, decrypted: &[u8]) -> Vec<CubeEvent>;
    fn create_command(&self, cmd: CubeCommand) -> Option<Vec<u8>>;
    fn has_gyro(&self) -> bool;
    fn requires_handshake(&self) -> bool;
    fn handshake_data(&self) -> Option<Vec<u8>> { None }
}

// ---------------------------------------------------------------------------
// Factory
// ---------------------------------------------------------------------------

/// Create the appropriate protocol handler for a given protocol version and
/// device MAC address. The MAC is used for key derivation (salting).
pub fn create_protocol(protocol: ProtocolVersion, mac_address: &str) -> Box<dyn CubeProtocol> {
    match protocol {
        ProtocolVersion::GanV1 => {
            // GAN V1 uses same encryption as V2, same packet format for basic events
            Box::new(gan_v2::GanV2Codec::new(
                &crate::protocol::gan_v2::ENCRYPTION_KEYS,
                mac_address,
            ))
        }
        ProtocolVersion::GanV2 => {
            Box::new(gan_v2::GanV2Codec::new(
                &crate::protocol::gan_v2::ENCRYPTION_KEYS,
                mac_address,
            ))
        }
        ProtocolVersion::MoYuAi => {
            // MoYu AI uses GAN Gen2 packet format but different keys
            Box::new(gan_v2::GanV2Codec::new(
                &crate::protocol::moyu_ai::ENCRYPTION_KEYS,
                mac_address,
            ))
        }
        ProtocolVersion::GanV3 => {
            Box::new(gan_v3::GanV3Codec::new(
                &crate::protocol::gan_v2::ENCRYPTION_KEYS,
                mac_address,
            ))
        }
        ProtocolVersion::GanV4 => {
            Box::new(gan_v4::GanV4Codec::new(
                &crate::protocol::gan_v2::ENCRYPTION_KEYS,
                mac_address,
            ))
        }
        ProtocolVersion::MoYuV3 => {
            Box::new(moyu_v3::MoYuV3Codec::new(mac_address))
        }
        ProtocolVersion::GiikerV1 => {
            Box::new(giiker::GiikerCodec::new())
        }
        ProtocolVersion::GoCube => {
            Box::new(gocube::GoCubeCodec::new())
        }
        ProtocolVersion::QiYiSmart => {
            Box::new(qiyi::QiYiCodec::new(mac_address))
        }
    }
}

// ---------------------------------------------------------------------------
// BitView helper — extract arbitrary-length bit words from byte arrays
// ---------------------------------------------------------------------------

/// A view over a byte buffer that allows extracting bit-packed values.
///
/// Matches the `GanProtocolMessageView` from the reference JS implementation.
pub struct BitView<'a> {
    data: &'a [u8],
}

impl<'a> BitView<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    /// Extract a bit word of `bit_len` bits starting at `start_bit`.
    /// For lengths <= 8, returns big-endian. For 16/32, supports endianness.
    pub fn get(&self, start_bit: usize, bit_len: usize) -> u32 {
        self.get_endian(start_bit, bit_len, false)
    }

    /// Extract a bit word with optional little-endian byte order (for 16/32 bit).
    pub fn get_endian(&self, start_bit: usize, bit_len: usize, little_endian: bool) -> u32 {
        if bit_len <= 8 {
            // Extract bits one by one (big-endian)
            let mut result: u32 = 0;
            for i in 0..bit_len {
                let bit_pos = start_bit + i;
                let byte_idx = bit_pos / 8;
                let bit_idx = 7 - (bit_pos % 8);
                if byte_idx < self.data.len() {
                    let bit = (self.data[byte_idx] >> bit_idx) & 1;
                    result = (result << 1) | (bit as u32);
                }
            }
            result
        } else if bit_len == 16 {
            let b0 = self.get(start_bit, 8) as u8;
            let b1 = self.get(start_bit + 8, 8) as u8;
            if little_endian {
                u16::from_le_bytes([b0, b1]) as u32
            } else {
                u16::from_be_bytes([b0, b1]) as u32
            }
        } else if bit_len == 32 {
            let b0 = self.get(start_bit, 8) as u8;
            let b1 = self.get(start_bit + 8, 8) as u8;
            let b2 = self.get(start_bit + 16, 8) as u8;
            let b3 = self.get(start_bit + 24, 8) as u8;
            if little_endian {
                u32::from_le_bytes([b0, b1, b2, b3])
            } else {
                u32::from_be_bytes([b0, b1, b2, b3])
            }
        } else {
            // Fallback: extract bit by bit
            let mut result: u32 = 0;
            for i in 0..bit_len {
                let bit_pos = start_bit + i;
                let byte_idx = bit_pos / 8;
                let bit_idx = 7 - (bit_pos % 8);
                if byte_idx < self.data.len() {
                    let bit = (self.data[byte_idx] >> bit_idx) & 1;
                    result = (result << 1) | (bit as u32);
                }
            }
            result
        }
    }
}

// ---------------------------------------------------------------------------
// Shared AES-CBC encryption (used by GAN Gen2/3/4)
// ---------------------------------------------------------------------------

use aes::Aes128;
use cipher::{BlockDecrypt, BlockEncrypt, KeyInit, generic_array::GenericArray};
use crate::protocol::EncryptionKeys;

/// Parse a MAC address string into 6 bytes, reversed.
/// Supports "AA:BB:CC:DD:EE:FF" format. Returns reversed salt bytes.
pub(crate) fn parse_mac_salt(mac_address: &str) -> [u8; 6] {
    let hex_only: String = mac_address.chars()
        .filter(|c| c.is_ascii_hexdigit())
        .collect();

    let mut salt = [0u8; 6];
    if hex_only.len() >= 12 {
        for i in 0..6 {
            salt[i] = u8::from_str_radix(&hex_only[i * 2..i * 2 + 2], 16).unwrap_or(0);
        }
        salt.reverse();
    }
    salt
}

/// Derive a salted key/IV pair from base keys and MAC address.
pub(crate) fn derive_gan_keys(keys: &EncryptionKeys, mac_address: &str) -> ([u8; 16], [u8; 16]) {
    let salt = parse_mac_salt(mac_address);
    let mut key = keys.key;
    let mut iv = keys.iv;
    for i in 0..6 {
        key[i] = ((key[i] as u16 + salt[i] as u16) % 0xFF) as u8;
        iv[i] = ((iv[i] as u16 + salt[i] as u16) % 0xFF) as u8;
    }
    (key, iv)
}

/// AES-128-CBC encrypt a single 16-byte block (single-block CBC = XOR with IV then encrypt).
pub(crate) fn aes_cbc_encrypt_block(key: &[u8; 16], iv: &[u8; 16], buffer: &mut [u8], offset: usize) {
    let cipher = Aes128::new(GenericArray::from_slice(key));
    let mut block = [0u8; 16];
    for i in 0..16 {
        block[i] = buffer[offset + i] ^ iv[i];
    }
    let block_ga = GenericArray::from_mut_slice(&mut block);
    cipher.encrypt_block(block_ga);
    buffer[offset..offset + 16].copy_from_slice(&block);
}

/// AES-128-CBC decrypt a single 16-byte block (single-block CBC = decrypt then XOR with IV).
pub(crate) fn aes_cbc_decrypt_block(key: &[u8; 16], iv: &[u8; 16], buffer: &mut [u8], offset: usize) {
    let cipher = Aes128::new(GenericArray::from_slice(key));
    let mut block = [0u8; 16];
    block.copy_from_slice(&buffer[offset..offset + 16]);
    let block_ga = GenericArray::from_mut_slice(&mut block);
    cipher.decrypt_block(block_ga);
    for i in 0..16 {
        buffer[offset + i] = block[i] ^ iv[i];
    }
}

/// GAN-style encrypt: encrypt first 16 bytes, then last 16 bytes.
pub(crate) fn gan_encrypt(key: &[u8; 16], iv: &[u8; 16], data: &[u8]) -> Vec<u8> {
    if data.len() < 16 {
        return data.to_vec();
    }
    let mut res = data.to_vec();
    aes_cbc_encrypt_block(key, iv, &mut res, 0);
    if res.len() > 16 {
        let offset = res.len() - 16;
        aes_cbc_encrypt_block(key, iv, &mut res, offset);
    }
    res
}

/// GAN-style decrypt: decrypt last 16 bytes, then first 16 bytes.
pub(crate) fn gan_decrypt(key: &[u8; 16], iv: &[u8; 16], data: &[u8]) -> Vec<u8> {
    if data.len() < 16 {
        return data.to_vec();
    }
    let mut res = data.to_vec();
    if res.len() > 16 {
        let offset = res.len() - 16;
        aes_cbc_decrypt_block(key, iv, &mut res, offset);
    }
    aes_cbc_decrypt_block(key, iv, &mut res, 0);
    res
}

/// Parse a signed 16-bit quaternion component from the GAN format.
/// Bit 15 is sign, bits 0-14 are magnitude, normalized to [-1, 1].
pub(crate) fn parse_quat_component(bits: u32) -> f32 {
    let sign = if (bits >> 15) & 1 == 1 { -1.0f32 } else { 1.0f32 };
    let magnitude = (bits & 0x7FFF) as f32 / 0x7FFF as f32;
    sign * magnitude
}

/// Parse a signed 4-bit angular velocity component from the GAN format.
/// Bit 3 is sign, bits 0-2 are magnitude.
pub(crate) fn parse_velocity_component(bits: u32) -> f32 {
    let sign = if (bits >> 3) & 1 == 1 { -1.0f32 } else { 1.0f32 };
    let magnitude = (bits & 0x7) as f32;
    sign * magnitude
}

/// Map a face index (0-5) to Face enum.
pub(crate) fn face_from_index(idx: u32) -> Option<Face> {
    match idx {
        0 => Some(Face::U),
        1 => Some(Face::R),
        2 => Some(Face::F),
        3 => Some(Face::D),
        4 => Some(Face::L),
        5 => Some(Face::B),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitview_basic() {
        // 0xAB = 10101011, 0xCD = 11001101
        let data = [0xAB, 0xCD];
        let view = BitView::new(&data);

        // First 4 bits of 0xAB = 1010 = 10
        assert_eq!(view.get(0, 4), 0x0A);
        // Next 4 bits = 1011 = 11
        assert_eq!(view.get(4, 4), 0x0B);
        // Full first byte
        assert_eq!(view.get(0, 8), 0xAB);
        // Bits across byte boundary: bits 4..12 = 1011_1100 = 0xBC
        assert_eq!(view.get(4, 8), 0xBC);
    }

    #[test]
    fn test_bitview_16bit() {
        let data = [0x01, 0x02];
        let view = BitView::new(&data);

        // Big-endian: 0x0102
        assert_eq!(view.get_endian(0, 16, false), 0x0102);
        // Little-endian: bytes [0x01, 0x02] as LE = 0x0201
        assert_eq!(view.get_endian(0, 16, true), 0x0201);
    }

    #[test]
    fn test_parse_mac_salt() {
        let salt = parse_mac_salt("CF:30:16:01:C7:2F");
        // Reversed: [0x2F, 0xC7, 0x01, 0x16, 0x30, 0xCF]
        assert_eq!(salt, [0x2F, 0xC7, 0x01, 0x16, 0x30, 0xCF]);
    }

    #[test]
    fn test_gan_encrypt_decrypt_roundtrip() {
        let keys = crate::protocol::gan_v2::ENCRYPTION_KEYS;
        let (key, iv) = derive_gan_keys(&keys, "AA:BB:CC:DD:EE:FF");

        let original = [0u8; 20];
        let encrypted = gan_encrypt(&key, &iv, &original);
        let decrypted = gan_decrypt(&key, &iv, &encrypted);
        assert_eq!(&decrypted, &original);
    }
}
