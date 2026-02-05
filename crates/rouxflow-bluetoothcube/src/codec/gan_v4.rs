//! GAN Gen4 protocol codec.
//!
//! Supports: GAN 12 ui Maglev, GAN 14 ui FreePlay, GAN 356 i Carry 4.
//!
//! 20-byte messages, no magic byte. Multi-part hardware info. Move history
//! recovery similar to Gen3.

use std::collections::HashMap;

use super::{
    AngularVelocity, BitView, CubeCommand, CubeEvent, CubeProtocol,
    derive_gan_keys, gan_decrypt, gan_encrypt,
    parse_quat_component, parse_velocity_component,
};
use crate::protocol::EncryptionKeys;
use rouxflow_core::cube::{Face, Quaternion};

/// Face lookup table for Gen4 move encoding (same as Gen3).
const GEN4_FACE_BITMASKS: [u32; 6] = [2, 32, 8, 1, 16, 4];

/// Face lookup for move history (same as Gen3).
const GEN4_HISTORY_FACE_MAP: [u32; 6] = [1, 5, 3, 0, 4, 2];

fn face_from_gen4_bitmask(bitmask: u32) -> Option<Face> {
    let idx = GEN4_FACE_BITMASKS.iter().position(|&m| m == bitmask)?;
    super::face_from_index(idx as u32)
}

fn face_from_gen4_history(val: u32) -> Option<Face> {
    let idx = GEN4_HISTORY_FACE_MAP.iter().position(|&m| m == val)?;
    super::face_from_index(idx as u32)
}

pub struct GanV4Codec {
    key: [u8; 16],
    iv: [u8; 16],
    serial: i32,
    last_serial: i32,
    move_buffer: Vec<BufferedMove>,
    /// Partial hardware info (multi-part: 0xFA, 0xFC, 0xFD, 0xFE)
    hw_info: HashMap<u8, String>,
}

#[derive(Clone)]
struct BufferedMove {
    serial: u16,
    event: CubeEvent,
}

impl GanV4Codec {
    pub fn new(keys: &EncryptionKeys, mac_address: &str) -> Self {
        let (key, iv) = derive_gan_keys(keys, mac_address);
        Self {
            key,
            iv,
            serial: -1,
            last_serial: -1,
            move_buffer: Vec::new(),
            hw_info: HashMap::new(),
        }
    }

    fn evict_move_buffer(&mut self) -> Vec<CubeEvent> {
        let mut evicted = Vec::new();
        while !self.move_buffer.is_empty() {
            let head_serial = self.move_buffer[0].serial;
            let diff = if self.last_serial == -1 {
                1
            } else {
                ((head_serial as i32 - self.last_serial) & 0xFF) as i32
            };

            if diff > 1 {
                break;
            }

            let entry = self.move_buffer.remove(0);
            self.last_serial = entry.serial as i32;
            evicted.push(entry.event);
        }

        if self.move_buffer.len() > 16 {
            evicted.push(CubeEvent::Disconnect);
            self.move_buffer.clear();
        }

        evicted
    }

    fn is_serial_in_range(&self, start: i32, end: i32, serial: i32, closed_start: bool, closed_end: bool) -> bool {
        let range = (end - start) & 0xFF;
        let offset = (serial - start) & 0xFF;
        range >= offset
            && (closed_start || ((start - serial) & 0xFF) > 0)
            && (closed_end || ((end - serial) & 0xFF) > 0)
    }

    fn inject_missed_move(&mut self, serial: u16, event: CubeEvent) {
        if self.move_buffer.iter().any(|e| e.serial == serial) {
            return;
        }

        if !self.move_buffer.is_empty() {
            let head_serial = self.move_buffer[0].serial as i32;
            if !self.is_serial_in_range(self.last_serial, head_serial, serial as i32, false, false) {
                return;
            }
            if serial == ((head_serial - 1) & 0xFF) as u16 {
                self.move_buffer.insert(0, BufferedMove { serial, event });
            }
        } else {
            if self.is_serial_in_range(self.last_serial, self.serial, serial as i32, false, true) {
                self.move_buffer.insert(0, BufferedMove { serial, event });
            }
        }
    }
}

impl CubeProtocol for GanV4Codec {
    fn name(&self) -> &str { "GAN Gen4" }

    fn decrypt(&self, data: &[u8]) -> Vec<u8> {
        gan_decrypt(&self.key, &self.iv, data)
    }

    fn encrypt(&self, data: &[u8]) -> Vec<u8> {
        gan_encrypt(&self.key, &self.iv, data)
    }

    fn decode_event(&mut self, decrypted: &[u8]) -> Vec<CubeEvent> {
        if decrypted.len() < 16 {
            return vec![];
        }

        let msg = BitView::new(decrypted);
        let event_type = msg.get(0, 8);
        let data_length = msg.get(8, 8);

        match event_type {
            0x01 => {
                // MOVE
                if self.last_serial == -1 {
                    return vec![];
                }

                let cube_timestamp = msg.get_endian(16, 32, true);
                let serial = msg.get_endian(48, 16, true) as u16;
                self.serial = serial as i32;

                let direction = msg.get(64, 2);
                let face_bitmask = msg.get(66, 6);

                if let Some(face) = face_from_gen4_bitmask(face_bitmask) {
                    let dir: i8 = if direction == 0 { 1 } else { -1 };
                    self.move_buffer.push(BufferedMove {
                        serial,
                        event: CubeEvent::Move {
                            serial,
                            face,
                            direction: dir,
                            cube_timestamp: Some(cube_timestamp),
                        },
                    });
                }

                self.evict_move_buffer()
            }
            0xD1 => {
                // MOVE_HISTORY
                let start_serial = msg.get(16, 8);
                let count = ((data_length - 1) * 2) as usize;

                for i in 0..count {
                    let face_val = msg.get(24 + 4 * i, 3);
                    let direction_bit = msg.get(27 + 4 * i, 1);

                    if let Some(face) = face_from_gen4_history(face_val) {
                        let dir: i8 = if direction_bit == 0 { 1 } else { -1 };
                        let serial = ((start_serial as i32 - i as i32) & 0xFF) as u16;
                        self.inject_missed_move(serial, CubeEvent::Move {
                            serial,
                            face,
                            direction: dir,
                            cube_timestamp: None,
                        });
                    }
                }

                self.evict_move_buffer()
            }
            0xED => {
                // FACELETS
                let serial = msg.get_endian(16, 16, true) as u16;
                self.serial = serial as i32;

                if self.last_serial == -1 {
                    self.last_serial = serial as i32;
                }

                let mut cp = [0u8; 8];
                let mut co = [0u8; 8];
                let mut ep = [0u8; 12];
                let mut eo = [0u8; 12];

                let mut cp_sum: u32 = 0;
                let mut co_sum: u32 = 0;
                for i in 0..7 {
                    let p = msg.get(32 + i * 3, 3);
                    let o = msg.get(53 + i * 2, 2);
                    cp[i] = p as u8;
                    co[i] = o as u8;
                    cp_sum += p;
                    co_sum += o;
                }
                cp[7] = (28 - cp_sum) as u8;
                co[7] = ((3 - (co_sum % 3)) % 3) as u8;

                let mut ep_sum: u32 = 0;
                let mut eo_sum: u32 = 0;
                for i in 0..11 {
                    let p = msg.get(69 + i * 4, 4);
                    let o = msg.get(113 + i, 1);
                    ep[i] = p as u8;
                    eo[i] = o as u8;
                    ep_sum += p;
                    eo_sum += o;
                }
                ep[11] = (66 - ep_sum) as u8;
                eo[11] = ((2 - (eo_sum % 2)) % 2) as u8;

                vec![CubeEvent::Facelets { serial, cp, co, ep, eo }]
            }
            0xEC => {
                // GYRO
                let qw = msg.get(16, 16);
                let qx = msg.get(32, 16);
                let qy = msg.get(48, 16);
                let qz = msg.get(64, 16);

                let vx = msg.get(80, 4);
                let vy = msg.get(84, 4);
                let vz = msg.get(88, 4);

                vec![CubeEvent::Gyro {
                    quaternion: Quaternion {
                        x: parse_quat_component(qx),
                        y: parse_quat_component(qy),
                        z: parse_quat_component(qz),
                        w: parse_quat_component(qw),
                    },
                    velocity: Some(AngularVelocity {
                        x: parse_velocity_component(vx),
                        y: parse_velocity_component(vy),
                        z: parse_velocity_component(vz),
                    }),
                }]
            }
            0xEF => {
                // BATTERY
                let level = msg.get(8 + (data_length as usize) * 8, 8).min(100) as u8;
                vec![CubeEvent::Battery { level }]
            }
            et if et >= 0xFA && et <= 0xFE => {
                // HARDWARE (multi-part)
                match et {
                    0xFA => {
                        // Product date
                        let year = msg.get_endian(24, 16, true);
                        let month = msg.get(40, 8);
                        let day = msg.get(48, 8);
                        self.hw_info.insert(0xFA, format!(
                            "{:04}-{:02}-{:02}", year, month, day
                        ));
                    }
                    0xFC => {
                        // Hardware name
                        let len = (data_length - 1) as usize;
                        let mut name = String::new();
                        for i in 0..len {
                            let ch = msg.get(24 + i * 8, 8) as u8;
                            if ch != 0 {
                                name.push(ch as char);
                            }
                        }
                        self.hw_info.insert(0xFC, name);
                    }
                    0xFD => {
                        // Software version
                        let major = msg.get(24, 4);
                        let minor = msg.get(28, 4);
                        self.hw_info.insert(0xFD, format!("{}.{}", major, minor));
                    }
                    0xFE => {
                        // Hardware version
                        let major = msg.get(24, 4);
                        let minor = msg.get(28, 4);
                        self.hw_info.insert(0xFE, format!("{}.{}", major, minor));
                    }
                    _ => {}
                }

                // Emit Hardware event only when all 4 parts received
                if self.hw_info.len() == 4 {
                    let name = self.hw_info.get(&0xFC).cloned().unwrap_or_default();
                    let gyro_supported = name == "GAN12uiM";
                    let event = CubeEvent::Hardware {
                        name,
                        sw_version: self.hw_info.get(&0xFD).cloned().unwrap_or_default(),
                        hw_version: self.hw_info.get(&0xFE).cloned().unwrap_or_default(),
                        gyro_supported,
                    };
                    self.hw_info.clear();
                    vec![event]
                } else {
                    vec![]
                }
            }
            0xEA => {
                // DISCONNECT
                vec![CubeEvent::Disconnect]
            }
            _ => vec![],
        }
    }

    fn create_command(&self, cmd: CubeCommand) -> Option<Vec<u8>> {
        let mut msg = vec![0u8; 20];
        match cmd {
            CubeCommand::RequestFacelets => {
                msg[..6].copy_from_slice(&[0xDD, 0x04, 0x00, 0xED, 0x00, 0x00]);
            }
            CubeCommand::RequestHardware => {
                self.hw_info.len(); // reset happens on next decode
                msg[..5].copy_from_slice(&[0xDF, 0x03, 0x00, 0x00, 0x00]);
            }
            CubeCommand::RequestBattery => {
                msg[..6].copy_from_slice(&[0xDD, 0x04, 0x00, 0xEF, 0x00, 0x00]);
            }
            CubeCommand::RequestReset => {
                let reset = [
                    0xD2, 0x0D, 0x05, 0x39, 0x77, 0x00, 0x00, 0x01,
                    0x23, 0x45, 0x67, 0x89, 0xAB, 0x00, 0x00, 0x00,
                ];
                msg[..16].copy_from_slice(&reset);
            }
        }
        Some(msg)
    }

    fn has_gyro(&self) -> bool { true }
    fn requires_handshake(&self) -> bool { false }
}
