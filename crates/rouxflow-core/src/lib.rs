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

#[wasm_bindgen]
pub fn handle_ble_packet(data: &[u8], session: &mut SessionManager) -> String {
    let protocol = GanV2Protocol::new([0u8; 16]);
    let decrypted = match protocol.decrypt(data) {
        Ok(d) => d,
        Err(_) => return "".into(),
    };

    if decrypted.get(0) == Some(&0x01) {
        if let Ok(m) = protocol.decode_move(&decrypted) {
            let move_str = m.notation();
            
            // Core logic manages state transitions internally
            let action = session.handle_scramble_move(&move_str, 0.0);
            if !action.is_empty() {
                return action;
            }
            
            return serde_json::to_string(&CoreAction::Move(move_str)).unwrap_or_default();
        }
    } else if decrypted.get(0) == Some(&0x05) {
        if let Ok(Some(q)) = protocol.decode_orientation(&decrypted) {
            return session.process_orientation(q.x, q.y, q.z, q.w, 0.0);
        }
    }

    "".into()
}
