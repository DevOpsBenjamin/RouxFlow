//! QiYi Smart protocol codec.
//!
//! Supports: QiYi Tornado V4 (SCS and AI), QiYi AI 3x3.
//!
//! AES-128-ECB encryption with a static key. CRC-16 Modbus for integrity.
//! Requires a handshake (sendHello with MAC) after BLE connection.
//!
//! Ref: cstimer qiyicube.js

use super::{CubeCommand, CubeEvent, CubeProtocol};
use crate::protocol::qiyi::ENCRYPTION_KEY;
use aes::Aes128;
use cipher::{BlockDecrypt, BlockEncrypt, KeyInit, generic_array::GenericArray};
use rouxflow_core::cube::Face;

/// QiYi move axis mapping: (move_byte - 1) >> 1 → standard URFDLB index.
const AXIS_MAP: [usize; 6] = [4, 1, 3, 0, 2, 5]; // L, R, D, U, F, B

/// Standard faces for URFDLB indexing.
const STD_FACES: [Face; 6] = [Face::U, Face::R, Face::F, Face::D, Face::L, Face::B];

/// QiYi facelet color order: "LRDUFB".
fn qiyi_color_char(val: u8) -> char {
    match val {
        0 => 'L',
        1 => 'R',
        2 => 'D',
        3 => 'U',
        4 => 'F',
        5 => 'B',
        _ => '?',
    }
}

/// CRC-16 Modbus.
fn crc16_modbus(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &b in data {
        crc ^= b as u16;
        for _ in 0..8 {
            if (crc & 1) != 0 {
                crc = (crc >> 1) ^ 0xA001;
            } else {
                crc >>= 1;
            }
        }
    }
    crc
}

pub struct QiYiCodec {
    cipher: Aes128,
    mac_bytes: [u8; 6],
    last_ts: u32,
}

impl QiYiCodec {
    pub fn new(mac_address: &str) -> Self {
        let cipher = Aes128::new(GenericArray::from_slice(&ENCRYPTION_KEY));

        let mut mac_bytes = [0u8; 6];
        let parts: Vec<&str> = mac_address.split(':').collect();
        if parts.len() == 6 {
            for (i, part) in parts.iter().enumerate() {
                mac_bytes[i] = u8::from_str_radix(part, 16).unwrap_or(0);
            }
        }

        Self {
            cipher,
            mac_bytes,
            last_ts: 0,
        }
    }

    /// AES-128-ECB decrypt: decrypt each 16-byte block independently.
    fn ecb_decrypt(&self, data: &[u8]) -> Vec<u8> {
        let mut result = data.to_vec();
        let mut i = 0;
        while i + 16 <= result.len() {
            let block = GenericArray::from_mut_slice(&mut result[i..i + 16]);
            self.cipher.decrypt_block(block);
            i += 16;
        }
        result
    }

    /// AES-128-ECB encrypt: encrypt each 16-byte block independently.
    fn ecb_encrypt(&self, data: &[u8]) -> Vec<u8> {
        let mut result = data.to_vec();
        let mut i = 0;
        while i + 16 <= result.len() {
            let block = GenericArray::from_mut_slice(&mut result[i..i + 16]);
            self.cipher.encrypt_block(block);
            i += 16;
        }
        result
    }

    /// Build a framed message: [0xFE][len][content...][crc16_lo][crc16_hi],
    /// padded to 16-byte alignment, then encrypted.
    fn build_message(&self, content: &[u8]) -> Vec<u8> {
        let total_len = 4 + content.len(); // FE + len_byte + content + 2 CRC bytes
        let mut msg = Vec::with_capacity(total_len);
        msg.push(0xFE);
        msg.push(total_len as u8);
        msg.extend_from_slice(content);

        let crc = crc16_modbus(&msg);
        msg.push((crc & 0xFF) as u8);
        msg.push((crc >> 8) as u8);

        // Pad to 16-byte alignment
        let pad = (16 - msg.len() % 16) % 16;
        msg.resize(msg.len() + pad, 0);

        self.ecb_encrypt(&msg)
    }

    /// Build the hello handshake message with MAC address.
    fn build_hello(&self) -> Vec<u8> {
        let mut content = vec![0x00, 0x6B, 0x01, 0x00, 0x00, 0x22, 0x06, 0x00, 0x02, 0x08, 0x00];
        // Append MAC bytes in reverse order
        for i in (0..6).rev() {
            content.push(self.mac_bytes[i]);
        }
        self.build_message(&content)
    }

    /// Build an ACK response from the opcode+timestamp portion of a received message.
    fn build_ack(&self, msg: &[u8]) -> Vec<u8> {
        if msg.len() >= 7 {
            self.build_message(&msg[2..7])
        } else {
            vec![]
        }
    }

    /// Parse 27 bytes of nibble-packed facelets into a 54-char facelet string.
    /// Even facelets: low nibble, odd facelets: high nibble.
    fn parse_facelets(face_msg: &[u8]) -> String {
        let mut result = String::with_capacity(54);
        for i in 0..54 {
            let byte = face_msg[i >> 1];
            let nibble = if i % 2 == 0 {
                byte & 0x0F        // low nibble for even
            } else {
                (byte >> 4) & 0x0F // high nibble for odd
            };
            result.push(qiyi_color_char(nibble));
        }
        result
    }

    /// Decode a move byte into Face + direction.
    fn decode_move(move_byte: u8) -> Option<(Face, i8)> {
        if move_byte == 0 || move_byte > 12 {
            return None;
        }
        let idx = ((move_byte - 1) >> 1) as usize;
        if idx >= 6 {
            return None;
        }
        let axis = AXIS_MAP[idx];
        let face = STD_FACES[axis];
        let direction: i8 = if move_byte & 1 == 1 { 1 } else { -1 }; // odd=CW, even=CCW
        Some((face, direction))
    }
}

impl CubeProtocol for QiYiCodec {
    fn name(&self) -> &str { "QiYi Smart" }

    fn decrypt(&self, data: &[u8]) -> Vec<u8> {
        self.ecb_decrypt(data)
    }

    fn encrypt(&self, data: &[u8]) -> Vec<u8> {
        self.ecb_encrypt(data)
    }

    fn decode_event(&mut self, decrypted: &[u8]) -> Vec<CubeEvent> {
        if decrypted.len() < 3 {
            return vec![];
        }

        // Validate frame start
        if decrypted[0] != 0xFE {
            return vec![];
        }

        // Truncate to declared length
        let msg_len = decrypted[1] as usize;
        if msg_len > decrypted.len() || msg_len < 3 {
            return vec![];
        }
        let msg = &decrypted[..msg_len];

        // Verify CRC
        if crc16_modbus(msg) != 0 {
            return vec![];
        }

        let opcode = msg[2];
        let ts = if msg.len() >= 7 {
            (msg[3] as u32) << 24
                | (msg[4] as u32) << 16
                | (msg[5] as u32) << 8
                | (msg[6] as u32)
        } else {
            0
        };

        let mut events = Vec::new();

        match opcode {
            0x02 => {
                // CUBE HELLO — initial state response
                // Battery: msg[35]
                // Facelets: msg[7..34] (27 bytes)
                // ACK: send back msg[2..7]
                if msg.len() >= 36 {
                    events.push(CubeEvent::Battery { level: msg[35].min(100) });
                }
                if msg.len() >= 34 {
                    let facelet_string = Self::parse_facelets(&msg[7..34]);
                    events.push(CubeEvent::RawFacelets { facelet_string });
                }

                // ACK response
                let ack = self.build_ack(msg);
                if !ack.is_empty() {
                    events.push(CubeEvent::WriteBack { data: ack });
                }

                self.last_ts = ts;
            }
            0x03 => {
                // STATE CHANGE — move event with history
                // ACK: send back msg[2..7]
                let ack = self.build_ack(msg);
                if !ack.is_empty() {
                    events.push(CubeEvent::WriteBack { data: ack });
                }

                // Current move: msg[34], timestamp: ts
                if msg.len() >= 35 {
                    // Build move list: current + history
                    let mut todo_moves: Vec<(u8, u32)> = vec![(msg[34], ts)];

                    // History: working backwards from offset 91 - 5*n
                    while todo_moves.len() < 10 {
                        let n = todo_moves.len();
                        let off = 91usize.saturating_sub(5 * n);
                        if off + 5 > msg.len() {
                            break;
                        }
                        let his_ts = (msg[off] as u32) << 24
                            | (msg[off + 1] as u32) << 16
                            | (msg[off + 2] as u32) << 8
                            | (msg[off + 3] as u32);
                        let his_mv = msg[off + 4];
                        if his_ts <= self.last_ts {
                            break;
                        }
                        todo_moves.push((his_mv, his_ts));
                    }

                    // Process from oldest to newest
                    for i in (0..todo_moves.len()).rev() {
                        let (mv, move_ts) = todo_moves[i];
                        if let Some((face, direction)) = Self::decode_move(mv) {
                            events.push(CubeEvent::Move {
                                serial: 0,
                                face,
                                direction,
                                cube_timestamp: Some(move_ts),
                            });
                        }
                    }

                    // Facelets for verification
                    if msg.len() >= 34 {
                        let facelet_string = Self::parse_facelets(&msg[7..34]);
                        events.push(CubeEvent::RawFacelets { facelet_string });
                    }

                    // Battery update
                    if msg.len() >= 36 {
                        events.push(CubeEvent::Battery { level: msg[35].min(100) });
                    }
                }

                self.last_ts = ts;
            }
            _ => {}
        }

        events
    }

    fn create_command(&self, cmd: CubeCommand) -> Option<Vec<u8>> {
        match cmd {
            // QiYi commands are sent as framed messages
            // The build_message handles framing, CRC, and encryption
            CubeCommand::RequestFacelets => {
                let content = vec![0x04]; // Request state
                Some(self.build_message(&content))
            }
            CubeCommand::RequestBattery => {
                let content = vec![0x0A]; // Battery/ACK type
                Some(self.build_message(&content))
            }
            _ => None,
        }
    }

    fn has_gyro(&self) -> bool { false }

    fn requires_handshake(&self) -> bool { true }

    fn handshake_data(&self) -> Option<Vec<u8>> {
        Some(self.build_hello())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc16_modbus() {
        // Known test vector
        let data = [0xFE, 0x05, 0x00];
        let crc = crc16_modbus(&data);
        assert_ne!(crc, 0); // Just verify it computes something

        // CRC of a full frame should be 0
        let mut frame = vec![0xFE, 0x05, 0x00];
        let crc = crc16_modbus(&frame);
        frame.push((crc & 0xFF) as u8);
        frame.push((crc >> 8) as u8);
        assert_eq!(crc16_modbus(&frame), 0);
    }

    #[test]
    fn test_parse_facelets() {
        // All zeros = all 'L' (color 0)
        let data = [0u8; 27];
        let result = QiYiCodec::parse_facelets(&data);
        assert_eq!(result.len(), 54);
        assert!(result.chars().all(|c| c == 'L'));
    }

    #[test]
    fn test_decode_move() {
        // move_byte 1: idx=0, axis_map[0]=4=L, odd=CW
        assert_eq!(QiYiCodec::decode_move(1), Some((Face::L, 1)));
        // move_byte 2: idx=0, axis_map[0]=4=L, even=CCW
        assert_eq!(QiYiCodec::decode_move(2), Some((Face::L, -1)));
        // move_byte 3: idx=1, axis_map[1]=1=R, odd=CW
        assert_eq!(QiYiCodec::decode_move(3), Some((Face::R, 1)));
        // move_byte 7: idx=3, axis_map[3]=0=U, odd=CW
        assert_eq!(QiYiCodec::decode_move(7), Some((Face::U, 1)));
        // move_byte 0: invalid
        assert_eq!(QiYiCodec::decode_move(0), None);
        // move_byte 13: invalid
        assert_eq!(QiYiCodec::decode_move(13), None);
    }

    #[test]
    fn test_ecb_roundtrip() {
        let codec = QiYiCodec::new("AA:BB:CC:DD:EE:FF");
        let data = vec![0u8; 16]; // One block
        let encrypted = codec.ecb_encrypt(&data);
        let decrypted = codec.ecb_decrypt(&encrypted);
        assert_eq!(decrypted, data);
    }
}
