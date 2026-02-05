//! GoCube / Rubik's Connected protocol codec.
//!
//! Supports: GoCube, GoCube X, Rubik's Connected.
//!
//! No encryption. Framed serial protocol over Nordic UART.
//! Frame: [0x2A] [len] [msgType] [payload...] [0x0D] [0x0A]
//!
//! Ref: cstimer gocube.js

use super::{CubeCommand, CubeEvent, CubeProtocol};
use rouxflow_core::cube::Face;

/// GoCube axis permutation: maps GoCube face indices to standard URFDLB.
/// GoCube 0→B(5), 1→F(2), 2→U(0), 3→D(3), 4→R(1), 5→L(4)
const AXIS_PERM: [usize; 6] = [5, 2, 0, 3, 1, 4];

/// GoCube sticker position mapping within a face (excluding center).
/// Maps GoCube sticker order to standard face positions.
const FACE_PERM: [usize; 8] = [0, 1, 2, 5, 8, 7, 6, 3];

/// Rotation offset per face for sticker mapping.
const FACE_OFFSET: [usize; 6] = [0, 0, 6, 2, 0, 0];

/// Standard face order for lookups: U, R, F, D, L, B
const STD_FACES: [Face; 6] = [Face::U, Face::R, Face::F, Face::D, Face::L, Face::B];

/// GoCube color order: "BFUDRL" → face chars.
fn gocube_color_char(val: u8) -> char {
    match val {
        0 => 'B',
        1 => 'F',
        2 => 'U',
        3 => 'D',
        4 => 'R',
        5 => 'L',
        _ => '?',
    }
}

pub struct GoCubeCodec;

impl GoCubeCodec {
    pub fn new() -> Self {
        Self
    }
}

impl CubeProtocol for GoCubeCodec {
    fn name(&self) -> &str { "GoCube" }

    fn decrypt(&self, data: &[u8]) -> Vec<u8> {
        // No encryption
        data.to_vec()
    }

    fn encrypt(&self, data: &[u8]) -> Vec<u8> {
        data.to_vec()
    }

    fn decode_event(&mut self, data: &[u8]) -> Vec<CubeEvent> {
        if data.len() < 4 {
            return vec![];
        }

        // Validate frame markers
        if data[0] != 0x2A
            || data[data.len() - 2] != 0x0D
            || data[data.len() - 1] != 0x0A
        {
            return vec![];
        }

        let msg_type = data[2];
        let msg_len = data.len() - 6; // payload length (excluding frame bytes)

        match msg_type {
            1 => {
                // MOVE: pairs of bytes [move_byte, timestamp_byte]
                let mut events = Vec::new();
                let mut i = 0;
                while i < msg_len {
                    let move_byte = data[3 + i];
                    let gc_face = (move_byte >> 1) as usize;
                    let gc_dir = move_byte & 1;

                    if gc_face < 6 {
                        let axis = AXIS_PERM[gc_face];
                        let face = STD_FACES[axis];
                        let direction: i8 = if gc_dir == 0 { 1 } else { -1 };

                        events.push(CubeEvent::Move {
                            serial: 0,
                            face,
                            direction,
                            cube_timestamp: None,
                        });
                    }

                    i += 2; // Each move is 2 bytes (move + timestamp)
                }
                events
            }
            2 => {
                // CUBE STATE: 54 bytes of color values
                // Layout: 6 faces × (1 center + 8 surrounding) = 54 bytes
                if data.len() < 3 + 54 {
                    return vec![];
                }

                let mut facelet = vec!['?'; 54];

                for a in 0..6 {
                    let axis = AXIS_PERM[a] * 9;
                    let aoff = FACE_OFFSET[a];

                    // Center sticker
                    facelet[axis + 4] = gocube_color_char(data[3 + a * 9]);

                    // 8 surrounding stickers
                    for i in 0..8 {
                        let pos = FACE_PERM[(i + aoff) % 8];
                        facelet[axis + pos] = gocube_color_char(data[3 + a * 9 + i + 1]);
                    }
                }

                let facelet_string: String = facelet.into_iter().collect();
                vec![CubeEvent::RawFacelets { facelet_string }]
            }
            3 => {
                // QUATERNION — not implemented, would need gyro parsing
                vec![]
            }
            5 => {
                // BATTERY LEVEL
                if data.len() < 4 {
                    return vec![];
                }
                let level = data[3].min(100);
                vec![CubeEvent::Battery { level }]
            }
            7 => {
                // OFFLINE STATS — logged only in cstimer
                vec![]
            }
            8 => {
                // CUBE TYPE — logged only in cstimer
                vec![]
            }
            _ => vec![],
        }
    }

    fn create_command(&self, cmd: CubeCommand) -> Option<Vec<u8>> {
        match cmd {
            CubeCommand::RequestBattery => Some(vec![50]),  // 0x32
            CubeCommand::RequestFacelets => Some(vec![51]), // 0x33
            _ => None,
        }
    }

    fn has_gyro(&self) -> bool { false }
    fn requires_handshake(&self) -> bool { false }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_validation() {
        let mut codec = GoCubeCodec::new();

        // Valid frame with battery type
        let frame = vec![0x2A, 0x01, 5, 85, 0x0D, 0x0A];
        let events = codec.decode_event(&frame);
        assert_eq!(events.len(), 1);
        match &events[0] {
            CubeEvent::Battery { level } => assert_eq!(*level, 85),
            _ => panic!("Expected Battery event"),
        }

        // Invalid frame (wrong start)
        let bad = vec![0x00, 0x01, 5, 85, 0x0D, 0x0A];
        assert!(codec.decode_event(&bad).is_empty());
    }

    #[test]
    fn test_move_decode() {
        let mut codec = GoCubeCodec::new();

        // Move byte: face=2 (U in GoCube), dir=0 (CW)
        // GoCube face 2 → AXIS_PERM[2] = 0 → U in standard
        let frame = vec![0x2A, 0x01, 1, 4, 0, 0x0D, 0x0A]; // move_byte=4 = face 2, dir 0
        let events = codec.decode_event(&frame);
        assert_eq!(events.len(), 1);
        match &events[0] {
            CubeEvent::Move { face, direction, .. } => {
                assert_eq!(*face, Face::U);
                assert_eq!(*direction, 1); // CW
            }
            _ => panic!("Expected Move event"),
        }
    }
}
