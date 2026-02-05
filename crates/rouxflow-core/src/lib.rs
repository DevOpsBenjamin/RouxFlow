use wasm_bindgen::prelude::*;

pub mod cube;
pub mod session;
pub mod storage;

#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! This is RouxFlow Core logic speaking.", name)
}

use crate::cube::gan_v2::GanV2Protocol;
use crate::cube::CubeProtocol;
use crate::session::{SessionManager, CoreAction};

// Re-export keys for standalone tester
pub const GAN_KEY: [u8; 16] = [0x01, 0x02, 0x42, 0x28, 0x31, 0x91, 0x16, 0x07, 0x20, 0x05, 0x18, 0x54, 0x42, 0x11, 0x12, 0x53];
pub const GAN_IV: [u8; 16] = [0x11, 0x03, 0x32, 0x28, 0x21, 0x01, 0x76, 0x27, 0x20, 0x95, 0x78, 0x14, 0x32, 0x12, 0x02, 0x43];
pub const MOYU_KEY: [u8; 16] = [0x05, 0x12, 0x02, 0x45, 0x02, 0x01, 0x29, 0x56, 0x12, 0x78, 0x12, 0x76, 0x81, 0x01, 0x08, 0x03];
pub const MOYU_IV: [u8; 16] = [0x01, 0x44, 0x28, 0x06, 0x86, 0x21, 0x22, 0x28, 0x51, 0x05, 0x08, 0x31, 0x82, 0x02, 0x21, 0x06];

#[wasm_bindgen]
pub fn encode_cube_command(command_id: u8, device_id: &str, use_moyu_key: bool) -> Vec<u8> {
    let key = if use_moyu_key { MOYU_KEY } else { GAN_KEY };
    let iv = if use_moyu_key { MOYU_IV } else { GAN_IV };
    let proto = GanV2Protocol::new(key, iv, device_id);
    
    let mut msg = [0u8; 20];
    msg[0] = command_id; // 0x04 for Request State, 0x05 for Hardware Info
    
    // In GAN protocol, we MUST use encrypt for commands
    proto.encrypt(&msg)
}

#[wasm_bindgen]
pub fn handle_ble_packet(data: &[u8], device_id: &str, session: &mut SessionManager) -> String {
    // 2. Try decryption with salted GAN keys
    let gan_proto = GanV2Protocol::new(GAN_KEY, GAN_IV, device_id);
    if let Ok(decrypted) = gan_proto.decrypt(data) {
        if decrypted.len() >= 16 {
             if let Ok(m) = gan_proto.decode_move(&decrypted) {
                 let now = chrono::Utc::now().timestamp_millis() as f64 / 1000.0;
                 let notation = m.notation();
                 let action = session.handle_scramble_move(&notation, now);
                 if !action.is_empty() { return action; }
                 return serde_json::to_string(&CoreAction::Move(notation)).unwrap_or_default();
             }
             if let Ok(Some(q)) = gan_proto.decode_orientation(&decrypted) {
                 let now = chrono::Utc::now().timestamp_millis() as f64 / 1000.0;
                 return session.process_orientation(q.x, q.y, q.z, q.w, now);
             }
        }
    }

    // 3. Try decryption with salted MoYu keys
    let moyu_proto = GanV2Protocol::new(MOYU_KEY, MOYU_IV, device_id);
    if let Ok(decrypted) = moyu_proto.decrypt(data) {
        if decrypted.len() >= 16 {
             if let Ok(m) = moyu_proto.decode_move(&decrypted) {
                 let now = chrono::Utc::now().timestamp_millis() as f64 / 1000.0;
                 let notation = m.notation();
                 let action = session.handle_scramble_move(&notation, now);
                 if !action.is_empty() { return action; }
                 return serde_json::to_string(&CoreAction::Move(notation)).unwrap_or_default();
             }
             if let Ok(Some(q)) = moyu_proto.decode_orientation(&decrypted) {
                 let now = chrono::Utc::now().timestamp_millis() as f64 / 1000.0;
                 return session.process_orientation(q.x, q.y, q.z, q.w, now);
             }
        }
    }

    // 4. Fallback: Unencrypted Moyu Decoder
    let raw_moves = crate::cube::moyu::MoyuDecoder::decode_packet(data);
    if let Some(m) = raw_moves.first() {
        let now = chrono::Utc::now().timestamp_millis() as f64 / 1000.0;
        let notation = m.notation();
        let action = session.handle_scramble_move(&notation, now);
        if !action.is_empty() { return action; }
        return serde_json::to_string(&CoreAction::Move(notation)).unwrap_or_default();
    }

    "".into()
}
