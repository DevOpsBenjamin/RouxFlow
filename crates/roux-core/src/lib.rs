use wasm_bindgen::prelude::*;

pub mod cube;
pub mod session;

#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! This is RouxFlow Core logic speaking.", name)
}

use crate::cube::moyu::MoyuProtocol;
use crate::cube::CubeProtocol;
use crate::session::SessionManager;

#[wasm_bindgen]
pub fn handle_ble_packet(data: &[u8], session: &mut SessionManager) -> String {
    let protocol = MoyuProtocol::new([0u8; 16]);
    let decrypted = match protocol.decrypt(data) {
        Ok(d) => d,
        Err(_) => return "".into(),
    };

    if decrypted.get(0) == Some(&0x01) {
        if let Ok(m) = protocol.decode_move(&decrypted) {
            let face_names = ["U", "R", "F", "D", "L", "B"];
            let amount_str = if m.amount == 1 { "" } else if m.amount == -1 { "'" } else { "2" };
            let move_str = format!("{}{}", face_names[m.face as usize], amount_str);
            
            // Sync with scramble validator if present
            // session.scramble_validator... (to be added)
            
            return format!("{{\"type\":\"move\",\"data\":{}}}", serde_json::to_string(&m).unwrap());
        }
    } else if decrypted.get(0) == Some(&0x05) {
        if let Ok(Some(q)) = protocol.decode_orientation(&decrypted) {
            // For now use a 0.0 timestamp or passing from JS
            return session.process_orientation(q.x, q.y, q.z, q.w, 0.0);
        }
    }

    "".into()
}
