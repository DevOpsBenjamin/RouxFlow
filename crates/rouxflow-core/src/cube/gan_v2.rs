use super::{CubeProtocol, CubeMove, CubeState, Face, Quaternion};
use aes::Aes128;
use cipher::{BlockDecrypt, BlockEncrypt, KeyInit, generic_array::GenericArray};

/// GAN Gen2 protocol implementation
/// Matches gan-web-bluetooth reference for encryption/decryption
pub struct GanV2Protocol {
    key: [u8; 16],
    iv: [u8; 16],
}

impl GanV2Protocol {
    pub fn new(base_key: [u8; 16], base_iv: [u8; 16], mac_address: &str) -> Self {
        let mut key = base_key;
        let mut iv = base_iv;

        // Salt with MAC address (reversed order)
        let hex_only: String = mac_address.chars()
            .filter(|c| c.is_ascii_hexdigit())
            .collect();
            
        if hex_only.len() >= 12 {
            let mut salt = [0u8; 6];
            for i in 0..6 {
                if let Ok(byte) = u8::from_str_radix(&hex_only[i*2..i*2+2], 16) {
                    salt[i] = byte;
                }
            }
            // Reverse salt as per GAN protocol
            salt.reverse();
            
            // DEBUG LOG for standalone troubleshooting
            println!("[Core] MAC: {} -> Salt: {:02x?}", mac_address, salt);

            for i in 0..6 {
                key[i] = key[i].wrapping_add(salt[i]);
                iv[i] = iv[i].wrapping_add(salt[i]);
            }
        }

        Self { key, iv }
    }

    pub fn encrypt(&self, data: &[u8]) -> Vec<u8> {
        if data.len() < 16 { return data.to_vec(); }
        let mut res = data.to_vec();
        
        // Encrypt first 16 bytes
        self.encrypt_chunk(&mut res, 0);
        
        // Encrypt last 16 bytes if overlapping
        if res.len() > 16 {
            let offset = res.len() - 16;
            self.encrypt_chunk(&mut res, offset);
        }
        res
    }

    fn encrypt_chunk(&self, buffer: &mut [u8], offset: usize) {
        let cipher = Aes128::new(GenericArray::from_slice(&self.key));
        let mut block = [0u8; 16];
        // CBC style: XOR with IV before encryption
        for i in 0..16 {
            block[i] = buffer[offset + i] ^ self.iv[i];
        }
        let mut block_ga = GenericArray::from_mut_slice(&mut block);
        cipher.encrypt_block(&mut block_ga);
        buffer[offset..offset+16].copy_from_slice(&block_ga);
    }

    fn decrypt_internal(&self, data: &[u8]) -> Vec<u8> {
        if data.len() < 16 { return data.to_vec(); }
        let mut res = data.to_vec();
        
        // Decrypt last 16 bytes first
        if res.len() > 16 {
            let offset = res.len() - 16;
            self.decrypt_chunk(&mut res, offset);
        }
        
        // Decrypt first 16 bytes
        self.decrypt_chunk(&mut res, 0);
        res
    }

    fn decrypt_chunk(&self, buffer: &mut [u8], offset: usize) {
        let cipher = Aes128::new(GenericArray::from_slice(&self.key));
        let mut block = [0u8; 16];
        block.copy_from_slice(&buffer[offset..offset+16]);
        
        let mut block_ga = GenericArray::from_mut_slice(&mut block);
        cipher.decrypt_block(&mut block_ga);
        
        // XOR with IV after decryption
        for i in 0..16 {
            buffer[offset + i] = block_ga[i] ^ self.iv[i];
        }
    }
}

struct BitView<'a> { data: &'a [u8] }

impl<'a> BitView<'a> {
    fn get_bit_word(&self, start_bit: usize, bit_len: usize) -> u32 {
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

impl CubeProtocol for GanV2Protocol {
    fn name(&self) -> &str { "GAN Gen2" }
    fn has_gyro(&self) -> bool { true }
    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        if data.is_empty() { return Ok(Vec::new()); }
        Ok(self.decrypt_internal(data))
    }
    fn decode_move(&self, data: &[u8]) -> Result<CubeMove, String> {
        let view = BitView { data };
        if view.get_bit_word(0, 4) != 0x02 { return Err("Not a move packet".into()); }
        let face_idx = view.get_bit_word(12, 4);
        let direction_idx = view.get_bit_word(16, 1);
        let face = match face_idx {
            0 => Face::U, 1 => Face::R, 2 => Face::F,
            3 => Face::D, 4 => Face::L, 5 => Face::B,
            _ => return Err("Invalid face".into()),
        };
        Ok(CubeMove { face, amount: if direction_idx == 0 { 1 } else { -1 } })
    }
    fn decode_state(&self, _data: &[u8]) -> Result<CubeState, String> { Ok(CubeState::new()) }
    fn decode_orientation(&self, data: &[u8]) -> Result<Option<Quaternion>, String> {
        let view = BitView { data };
        if view.get_bit_word(0, 4) != 0x01 { return Ok(None); }
        let parse_quat = |bits: u32| {
            let sign = (bits >> 15) & 1;
            let val = bits & 0x7FFF;
            (if sign == 1 { -1.0 } else { 1.0 }) * (val as f32) / 32767.0
        };
        Ok(Some(Quaternion {
            w: parse_quat(view.get_bit_word(4, 16)),
            x: parse_quat(view.get_bit_word(20, 16)),
            y: parse_quat(view.get_bit_word(36, 16)),
            z: parse_quat(view.get_bit_word(52, 16)),
        }))
    }
}
