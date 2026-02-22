//! Giiker (Xiaomi) protocol codec.
//!
//! Supports: Giiker i3, i3S, i3Y, Mi Smart Magic Cube, Hi-Smart cubes.
//!
//! Older models send unencrypted 20-byte packets; newer models encrypt with
//! an ADD-based scheme (marker byte 18 == 0xA7).
//!
//! Ref: cstimer giikercube.js

use super::{CubeCommand, CubeEvent, CubeProtocol};
use crate::protocol::giiker::{ENCRYPTION_MARKER, KEY_TABLE};
use rouxflow_core::cube::Face;

/// Corner facelet positions (ULB numbering, maps corner index to 3 facelet positions).
#[allow(dead_code)]
const C_FACELET: [[u8; 3]; 8] = [
    [26, 15, 29],
    [20, 8, 9],
    [18, 38, 6],
    [24, 27, 44],
    [51, 35, 17],
    [45, 11, 2],
    [47, 0, 36],
    [53, 42, 33],
];

/// Edge facelet positions (maps edge index to 2 facelet positions).
#[allow(dead_code)]
const E_FACELET: [[u8; 2]; 12] = [
    [25, 28],
    [23, 12],
    [19, 7],
    [21, 41],
    [32, 16],
    [5, 10],
    [3, 37],
    [30, 43],
    [52, 34],
    [48, 14],
    [46, 1],
    [50, 39],
];

/// Corner orientation sign mask for Giiker encoding.
const CO_MASK: [i8; 8] = [-1, 1, -1, 1, 1, -1, 1, -1];

/// Giiker face order for moves: "BDLURF" (1-based in the protocol).
const GIIKER_FACES: [Face; 6] = [Face::B, Face::D, Face::L, Face::U, Face::R, Face::F];

pub struct GiikerCodec {
    last_move_nibbles: [u8; 2],
}

impl GiikerCodec {
    pub fn new() -> Self {
        Self {
            last_move_nibbles: [0, 0],
        }
    }

    /// Decrypt a Giiker packet (ADD-based encryption).
    /// If byte[18] == 0xA7, applies the ADD key table.
    pub fn decrypt_packet(data: &[u8]) -> Vec<u8> {
        let mut raw: Vec<u8> = data.to_vec();
        if raw.len() >= 20 && raw[18] == ENCRYPTION_MARKER {
            let k1 = ((raw[19] >> 4) & 0xF) as usize;
            let k2 = (raw[19] & 0xF) as usize;
            for i in 0..18 {
                raw[i] = raw[i].wrapping_add(KEY_TABLE[i + k1]).wrapping_add(KEY_TABLE[i + k2]);
            }
            raw.truncate(18);
        }
        raw
    }

    /// Convert raw bytes to hex nibbles.
    pub fn to_hex_nibbles(raw: &[u8]) -> Vec<u8> {
        let mut nibbles = Vec::with_capacity(raw.len() * 2);
        for &b in raw {
            nibbles.push((b >> 4) & 0xF);
            nibbles.push(b & 0xF);
        }
        nibbles
    }
}

impl CubeProtocol for GiikerCodec {
    fn name(&self) -> &str { "Giiker" }

    fn decrypt(&self, data: &[u8]) -> Vec<u8> {
        Self::decrypt_packet(data)
    }

    fn encrypt(&self, data: &[u8]) -> Vec<u8> {
        // Giiker is read-only from the data characteristic
        data.to_vec()
    }

    fn decode_event(&mut self, decrypted: &[u8]) -> Vec<CubeEvent> {
        if decrypted.len() < 18 {
            return vec![];
        }

        let nibbles = Self::to_hex_nibbles(decrypted);
        if nibbles.len() < 40 {
            return vec![];
        }

        // Parse corner permutation (CP): nibbles 0..8, 1-based → 0-based
        let mut cp = [0u8; 8];
        for i in 0..8 {
            if nibbles[i] == 0 || nibbles[i] > 8 {
                return vec![]; // Invalid
            }
            cp[i] = nibbles[i] - 1;
        }

        // Parse corner orientation (CO): nibbles 8..16 with coMask
        let mut co = [0u8; 8];
        for i in 0..8 {
            let val = nibbles[i + 8] as i16;
            let mask = CO_MASK[i] as i16;
            co[i] = ((3 + val * mask) % 3) as u8;
        }

        // Parse edge permutation (EP): nibbles 16..28, 1-based → 0-based
        let mut ep = [0u8; 12];
        for i in 0..12 {
            if nibbles[i + 16] == 0 || nibbles[i + 16] > 12 {
                return vec![]; // Invalid
            }
            ep[i] = nibbles[i + 16] - 1;
        }

        // Parse edge orientation (EO): nibbles 28..31, 4 bits each = 12 EO bits
        let mut eo = [0u8; 12];
        for i in 0..3 {
            for bit in 0..4 {
                let mask = 8u8 >> bit;
                eo[i * 4 + bit] = if (nibbles[i + 28] & mask) != 0 { 1 } else { 0 };
            }
        }

        let mut events = Vec::new();

        // Emit Facelets event
        events.push(CubeEvent::Facelets {
            serial: 0,
            cp,
            co,
            ep,
            eo,
        });

        // Parse last move: nibbles 32..34 (face, direction)
        let move_face = nibbles[32];
        let move_dir = nibbles[33];

        // Only emit a Move event if it differs from the last one we saw
        if (move_face != self.last_move_nibbles[0] || move_dir != self.last_move_nibbles[1])
            && move_face >= 1
            && move_face <= 6
        {
            self.last_move_nibbles = [move_face, move_dir];

            let face = GIIKER_FACES[(move_face - 1) as usize];
            let direction: i8 = match (move_dir.wrapping_sub(1)) % 7 {
                0 => 1,  // CW
                1 => 2,  // Double (180)
                2 => -1, // CCW
                _ => 1,
            };

            events.push(CubeEvent::Move {
                serial: 0,
                face,
                direction,
                cube_timestamp: None,
            });
        }

        events
    }

    fn create_command(&self, cmd: CubeCommand) -> Option<Vec<u8>> {
        match cmd {
            CubeCommand::RequestBattery => {
                // Battery is read via a separate BLE service (0xaaaa/0xaaab/0xaaac)
                // Send 0xB5 to the write characteristic of that service
                Some(vec![0xB5])
            }
            _ => None,
        }
    }

    fn has_gyro(&self) -> bool { false }
    fn requires_handshake(&self) -> bool { false }
}

