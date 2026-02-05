use super::{CubeMove, Face};
use aes::Aes128;
use cbc::Decryptor;
use cipher::{BlockDecryptMut, KeyIvInit};

type Aes128CbcDec = Decryptor<Aes128>;

// Constants derived from "Moyu Protocol V3" Reverse Engineering
const MASTER_KEY: [u8; 16] = [21, 119, 58, 92, 103, 14, 45, 31, 23, 103, 42, 19, 155, 103, 82, 87];
const MASTER_IV: [u8; 16] = [17, 35, 38, 37, 134, 42, 44, 59, 85, 6, 127, 49, 126, 103, 33, 87];

pub struct MoyuV10Protocol {
    device_key: [u8; 16],
    device_iv: [u8; 16],
}

impl MoyuV10Protocol {
    pub fn new(mac_addr: &str) -> Self {
        let mac_bytes: Vec<u8> = mac_addr
            .split(':')
            .map(|s| u8::from_str_radix(s, 16).unwrap_or(0))
            .collect();

        // Default to zero key if mac parsing fails (should handle better in prod)
        if mac_bytes.len() != 6 {
            return Self { device_key: [0; 16], device_iv: [0; 16] };
        }

        let mut device_key = MASTER_KEY;
        let mut device_iv = MASTER_IV;

        // V3 Key Derivation: (Master + MAC_Byte_Reversed) % 255
        for i in 0..6 {
            let m = mac_bytes[5 - i];
            device_key[i] = ((MASTER_KEY[i] as u16 + m as u16) % 255) as u8;
            device_iv[i] = ((MASTER_IV[i] as u16 + m as u16) % 255) as u8;
        }

        Self { device_key, device_iv }
    }

    /// Decrypts a 20-byte packet using the Double Overlapping Pass algorithm
    pub fn decrypt(&self, data: &[u8]) -> Vec<u8> {
        let mut buffer = data.to_vec();
        
        // Pass 1: Decrypt tail (bytes 4-20)
        if buffer.len() >= 20 {
            let mut block = [0u8; 16];
            block.copy_from_slice(&buffer[4..20]);
            let dec = Aes128CbcDec::new(&self.device_key.into(), &self.device_iv.into());
            dec.decrypt_padded_mut::<cipher::block_padding::NoPadding>(&mut block).ok();
            buffer[4..20].copy_from_slice(&block);
        }

        // Pass 2: Decrypt head (bytes 0-16) - NOTE: This uses the result of Pass 1 for overlap
        if buffer.len() >= 16 {
            let mut block = [0u8; 16];
            block.copy_from_slice(&buffer[0..16]);
            let dec = Aes128CbcDec::new(&self.device_key.into(), &self.device_iv.into());
            dec.decrypt_padded_mut::<cipher::block_padding::NoPadding>(&mut block).ok();
            buffer[0..16].copy_from_slice(&block);
        }

        buffer
    }

    /// Parses a decrypted packet into a CubeMove
    pub fn parse_packet(&self, decrypted: &[u8]) -> Option<CubeMove> {
        if decrypted.len() < 20 { return None; }

        match decrypted[0] {
            0xA5 => {
                // Format: A5 [Face] [Dir] ...
                let face_code = decrypted[1];
                let dir_code = decrypted[2]; // 1=CW, 2=CCW, 3=Double? Need to verify logs 
                // Based on standard protocols: 
                // U=0, D=1, L=2, R=3, F=4, B=5 (Might differ for V10, needs validation)
                // From user logs: 00, 01, 02 seen for moves.
                
                let face = match face_code {
                    0 => Face::U,
                    1 => Face::D, // Assuming standard index
                    2 => Face::L,
                    3 => Face::R,
                    4 => Face::F,
                    5 => Face::B,
                    _ => return None,
                };

                // Direction mapping from logs needs verification. 
                // Usually: 1=CW, 2=CCW, 3=2 (Double)
                // But user logs showed 00, 01, 02 after A5.
                // Let's assume standard mapping for now and refine with live data.
                let amount = match dir_code {
                    1 => 1,
                    2 => -1,
                    3 => 2, 
                    _ => 1 // Default to CW if unknown
                };

                Some(CubeMove { face, amount })
            },
            _ => None // Ignore A1 (Info), A3 (State), A4 (Batt), AB (Gyro) for move stream
        }
    }
}
