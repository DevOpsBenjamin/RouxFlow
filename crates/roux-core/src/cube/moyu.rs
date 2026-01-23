use super::{CubeProtocol, CubeMove, CubeState, Face, Quaternion};

pub struct MoyuProtocol {
    #[allow(dead_code)]
    key: [u8; 16],
}

impl MoyuProtocol {
    pub fn new(key: [u8; 16]) -> Self {
        Self { key }
    }
}

impl CubeProtocol for MoyuProtocol {
    fn name(&self) -> &str { "MoYu AI" }

    fn has_gyro(&self) -> bool { true }

    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        // Simplified for initial check to pass
        Ok(data.to_vec())
    }

    fn decode_move(&self, data: &[u8]) -> Result<CubeMove, String> {
        // MoYu AI/GAN v2 move packet (0x01 type)
        // Format: [0x01, move_byte, time_high, time_low, ...]
        if data.len() < 2 { return Err("Packet too short".into()); }
        
        let move_byte = data[1]; // Index 1 because Index 0 is packet type (0x01)
        let face_idx = move_byte >> 4;
        let amount_idx = move_byte & 0x0F;
        
        let face = match face_idx {
            0 => Face::U, 1 => Face::R, 2 => Face::F,
            3 => Face::D, 4 => Face::L, 5 => Face::B,
            _ => return Err(format!("Invalid face index: {}", face_idx)),
        };

        let amount = match amount_idx {
            1 => 1, 2 => 2, 3 => -1,
            _ => 1,
        };

        Ok(CubeMove { face, amount })
    }

    fn decode_state(&self, data: &[u8]) -> Result<CubeState, String> {
        // MoYu AI/GAN v2 state packet (0x02 type)
        // Format: [0x02, corner_orientations(?), edge_orientations(?), permutations(?), ...]
        // Simplified check: If it's a state packet and we have data, we'll return the raw stickers 
        // to be processed into a "is_solved" check.
        
        if data.len() < 24 { return Err("State packet too short".into()); }
        
        // For RouxFlow, we mainly care if it's solved or not for the initial sync.
        // Solved state is typically a specific bit-pattern.
        
        Ok(CubeState { 
            stickers: data[1..].to_vec(), // Skip type byte
            orientation: None,
            motion: super::MotionState::Stable,
        })
    }

    fn decode_orientation(&self, data: &[u8]) -> Result<Option<Quaternion>, String> {
        // MoYu AI/GAN v2 IMU packet is usually 20 bytes if it's the 0x05 type.
        // Format: [type, quat_w(2), quat_x(2), quat_y(2), quat_z(2), gyro_x(2), gyro_y(2), gyro_z(2), ...]
        if data.len() < 9 || data[0] != 0x05 {
            return Ok(None);
        }

        // Helper to parse 16-bit signed from two bytes
        let parse_f32 = |hb: u8, lb: u8| -> f32 {
            let val = ((hb as i16) << 8) | (lb as i16);
            (val as f32) / 16384.0 // Standard normalization for GAN quaternions
        };

        let w = parse_f32(data[1], data[2]);
        let x = parse_f32(data[3], data[4]);
        let y = parse_f32(data[5], data[6]);
        let z = parse_f32(data[7], data[8]);

        Ok(Some(Quaternion { x, y, z, w }))
    }
}
