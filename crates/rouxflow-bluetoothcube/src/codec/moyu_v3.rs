//! MoYu V3 protocol codec.
//!
//! Supports: MoYu WeiLong V10.
//!
//! Uses the **same encryption scheme as GAN Gen2** (AES-128-CBC, decrypt
//! last-16 then first-16, MAC-salted keys). The only difference from GAN
//! is the base key/IV values and the packet format (bit-packed).
//!
//! Ref: cstimer moyu32cube.js

use super::{
    AngularVelocity, BitView, CubeCommand, CubeEvent, CubeProtocol,
    derive_gan_keys, gan_decrypt, gan_encrypt,
    parse_quat_component, parse_velocity_component,
};
use crate::protocol::moyu_v3::ENCRYPTION_KEYS;
use rouxflow_core::cube::{Face, Quaternion};

/// MoYu V3 face order: "FBUDLR" — used for move encoding and facelet colors.
const MOYU_FACES: [Face; 6] = [Face::F, Face::B, Face::U, Face::D, Face::L, Face::R];

/// Face remapping for facelet parsing: output URFDLB order from FBUDLR storage.
/// faces[i] is the index into the FBUDLR-ordered face data for output face i.
/// Output order: 0=U, 1=R, 2=F, 3=D, 4=L, 5=B
const FACELET_REMAP: [usize; 6] = [2, 5, 0, 3, 4, 1];

/// Face char for a FBUDLR color value.
fn moyu_color_char(val: u32) -> char {
    match val {
        0 => 'F',
        1 => 'B',
        2 => 'U',
        3 => 'D',
        4 => 'L',
        5 => 'R',
        _ => '?',
    }
}

/// Face char for a URFDLB face index (center sticker).
fn center_char(face_idx: usize) -> char {
    moyu_color_char(FACELET_REMAP[face_idx] as u32)
}

pub struct MoYuV3Codec {
    key: [u8; 16],
    iv: [u8; 16],
    move_count: i16,
    prev_move_count: i16,
}

impl MoYuV3Codec {
    pub fn new(mac_address: &str) -> Self {
        let (key, iv) = derive_gan_keys(&ENCRYPTION_KEYS, mac_address);
        Self {
            key,
            iv,
            move_count: -1,
            prev_move_count: -1,
        }
    }

    /// Parse 144 bits of facelet data into a 54-char facelet string (URFDLB order).
    ///
    /// Bit layout: 6 faces x 24 bits = 144 bits.
    /// Each face has 8 non-center stickers x 3 bits = 24 bits.
    /// Faces stored in FBUDLR order; output in URFDLB order via FACELET_REMAP.
    fn parse_facelets(view: &BitView, start_bit: usize) -> String {
        let mut result = String::with_capacity(54);

        for out_face in 0..6 {
            let src_face = FACELET_REMAP[out_face];
            let face_start = start_bit + src_face * 24;

            for j in 0..8 {
                let color = view.get(face_start + j * 3, 3);
                result.push(moyu_color_char(color));
                // Insert center sticker after the 4th non-center sticker
                if j == 3 {
                    result.push(center_char(out_face));
                }
            }
        }

        result
    }

    /// Decode a 5-bit move value into Face + direction.
    fn decode_move(m: u32) -> Option<(Face, i8)> {
        let face_idx = (m >> 1) as usize;
        if face_idx >= 6 {
            return None;
        }
        let face = MOYU_FACES[face_idx];
        let direction: i8 = if m & 1 == 0 { 1 } else { -1 };
        Some((face, direction))
    }
}

impl CubeProtocol for MoYuV3Codec {
    fn name(&self) -> &str { "MoYu V3" }

    fn decrypt(&self, data: &[u8]) -> Vec<u8> {
        gan_decrypt(&self.key, &self.iv, data)
    }

    fn encrypt(&self, data: &[u8]) -> Vec<u8> {
        gan_encrypt(&self.key, &self.iv, data)
    }

    fn decode_event(&mut self, decrypted: &[u8]) -> Vec<CubeEvent> {
        if decrypted.is_empty() {
            return vec![];
        }

        let view = BitView::new(decrypted);
        let msg_type = view.get(0, 8);

        match msg_type {
            161 => {
                // 0xA1 — Hardware info
                // Device name: bits 8..72 (8 chars x 8 bits)
                let mut name = String::new();
                for i in 0..8 {
                    let ch = view.get(8 + i * 8, 8) as u8;
                    if ch > 0 && ch < 128 {
                        name.push(ch as char);
                    }
                }
                let sw_major = view.get(72, 8);
                let sw_minor = view.get(80, 8);
                let hw_major = view.get(88, 8);
                let hw_minor = view.get(96, 8);

                vec![CubeEvent::Hardware {
                    name: name.trim().to_string(),
                    sw_version: format!("{}.{}", sw_major, sw_minor),
                    hw_version: format!("{}.{}", hw_major, hw_minor),
                    gyro_supported: true,
                }]
            }
            163 => {
                // 0xA3 — Full cube state (facelets)
                // Facelets: bits 8..152 (144 bits)
                // Move counter: bits 152..160
                let facelet_string = Self::parse_facelets(&view, 8);
                let mc = view.get(152, 8) as i16;

                if self.prev_move_count == -1 {
                    self.move_count = mc;
                    self.prev_move_count = mc;
                }

                vec![CubeEvent::RawFacelets { facelet_string }]
            }
            164 => {
                // 0xA4 — Battery level
                let level = view.get(8, 8) as u8;
                vec![CubeEvent::Battery { level: level.min(100) }]
            }
            165 => {
                // 0xA5 — Move event
                // 5x 16-bit timestamps: bits 8..88
                // Move counter: bits 88..96
                // 5x 5-bit moves: bits 96..121
                let mc = view.get(88, 8) as i16;
                self.move_count = mc;

                if self.prev_move_count == -1 || mc == self.prev_move_count {
                    return vec![];
                }

                let mut time_offsets = [0u16; 5];
                let mut moves = [0u32; 5];

                for i in 0..5 {
                    time_offsets[i] = view.get(8 + i * 16, 16) as u16;
                    moves[i] = view.get(96 + i * 5, 5);
                }

                // Reject if any move value is invalid
                for m in &moves {
                    if *m >= 12 {
                        return vec![];
                    }
                }

                let mut move_diff = ((mc - self.prev_move_count) & 0xFF) as usize;
                if move_diff > 5 {
                    move_diff = 5;
                }
                self.prev_move_count = mc;

                // Process moves from oldest to newest
                let mut events = Vec::new();
                for i in (0..move_diff).rev() {
                    if let Some((face, direction)) = Self::decode_move(moves[i]) {
                        events.push(CubeEvent::Move {
                            serial: mc as u16,
                            face,
                            direction,
                            cube_timestamp: Some(time_offsets[i] as u32),
                        });
                    }
                }

                events
            }
            171 => {
                // 0xAB — Gyroscope (GAN-style 16-bit sign+magnitude quaternion)
                // bits 8-23: qw, 24-39: qx, 40-55: qy, 56-71: qz
                // bits 72-83: angular velocity (3x 4-bit)
                if decrypted.len() < 10 {
                    return vec![];
                }

                let qw = view.get(8, 16);
                let qx = view.get(24, 16);
                let qy = view.get(40, 16);
                let qz = view.get(56, 16);

                let vx = view.get(72, 4);
                let vy = view.get(76, 4);
                let vz = view.get(80, 4);

                vec![CubeEvent::Gyro {
                    quaternion: Quaternion {
                        w: parse_quat_component(qw),
                        x: parse_quat_component(qx),
                        y: parse_quat_component(qy),
                        z: parse_quat_component(qz),
                    },
                    velocity: Some(AngularVelocity {
                        x: parse_velocity_component(vx),
                        y: parse_velocity_component(vy),
                        z: parse_velocity_component(vz),
                    }),
                }]
            }
            _ => vec![],
        }
    }

    fn create_command(&self, cmd: CubeCommand) -> Option<Vec<u8>> {
        let opcode = match cmd {
            CubeCommand::RequestHardware => 161, // 0xA1
            CubeCommand::RequestFacelets => 163, // 0xA3
            CubeCommand::RequestBattery => 164,  // 0xA4
            CubeCommand::RequestReset => return None,
        };
        let mut msg = vec![0u8; 20];
        msg[0] = opcode;
        Some(msg)
    }

    fn has_gyro(&self) -> bool { true }

    fn requires_handshake(&self) -> bool { false }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_move() {
        assert_eq!(MoYuV3Codec::decode_move(0), Some((Face::F, 1)));   // F CW
        assert_eq!(MoYuV3Codec::decode_move(1), Some((Face::F, -1)));  // F CCW
        assert_eq!(MoYuV3Codec::decode_move(2), Some((Face::B, 1)));   // B CW
        assert_eq!(MoYuV3Codec::decode_move(4), Some((Face::U, 1)));   // U CW
        assert_eq!(MoYuV3Codec::decode_move(11), Some((Face::R, -1))); // R CCW
        assert_eq!(MoYuV3Codec::decode_move(12), None);                // invalid
    }
}
