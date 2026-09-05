//! GAN Gen2 protocol codec.
//!
//! Supports: GAN Mini ui FreePlay, GAN 12 ui FreePlay, GAN 12 ui,
//! GAN 356 i Carry/Carry S/Carry E, GAN 356 i 3, Monster Go 3Ai,
//! MoYu AI 2023/v2 (same packet format, different keys).

use super::{
    AngularVelocity, BitView, CubeCommand, CubeEvent, CubeProtocol,
    derive_gan_keys, face_from_index, gan_decrypt, gan_encrypt,
    parse_quat_component, parse_velocity_component,
};
use crate::protocol::EncryptionKeys;
use rouxflow_core::cube::Quaternion;

pub struct GanV2Codec {
    key: [u8; 16],
    iv: [u8; 16],
    last_serial: i16,
    last_move_timestamp: f64,
    cube_timestamp: f64,
}

impl GanV2Codec {
    pub fn new(keys: &EncryptionKeys, mac_address: &str) -> Self {
        let (key, iv) = derive_gan_keys(keys, mac_address);
        Self {
            key,
            iv,
            last_serial: -1,
            last_move_timestamp: 0.0,
            cube_timestamp: 0.0,
        }
    }
}

impl CubeProtocol for GanV2Codec {
    fn name(&self) -> &str { "GAN Gen2" }

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
        let event_type = msg.get(0, 4);
        let mut events = Vec::new();

        match event_type {
            0x01 => {
                // GYRO
                let qw = msg.get(4, 16);
                let qx = msg.get(20, 16);
                let qy = msg.get(36, 16);
                let qz = msg.get(52, 16);

                let vx = msg.get(68, 4);
                let vy = msg.get(72, 4);
                let vz = msg.get(76, 4);

                events.push(CubeEvent::Gyro {
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
                });
            }
            0x02 => {
                // MOVE — only accept after first facelets event
                if self.last_serial != -1 {
                    let serial = msg.get(4, 8);
                    let diff = ((serial as i32 - self.last_serial as i32) & 0xFF).min(7) as usize;
                    self.last_serial = serial as i16;

                    if diff > 0 {
                        for i in (0..diff).rev() {
                            let face_idx = msg.get(12 + 5 * i as usize, 4);
                            let direction_bit = msg.get(16 + 5 * i as usize, 1);
                            let elapsed = msg.get(47 + 16 * i as usize, 16);

                            let elapsed_f = if elapsed == 0 {
                                // Timestamp register overflow — use wall clock estimate
                                0.0 // Caller should substitute with local time delta
                            } else {
                                elapsed as f64
                            };
                            self.cube_timestamp += elapsed_f;

                            if let Some(face) = face_from_index(face_idx) {
                                let direction: i8 = if direction_bit == 0 { 1 } else { -1 };
                                events.push(CubeEvent::Move {
                                    serial: ((serial as i32 - i as i32) & 0xFF) as u16,
                                    face,
                                    direction,
                                    cube_timestamp: Some(self.cube_timestamp as u32),
                                });
                            }
                        }
                        self.last_move_timestamp = self.cube_timestamp;
                    }
                }
            }
            0x04 => {
                // FACELETS
                let serial = msg.get(4, 8);

                if self.last_serial == -1 {
                    self.last_serial = serial as i16;
                }

                let mut cp = [0u8; 8];
                let mut co = [0u8; 8];
                let mut ep = [0u8; 12];
                let mut eo = [0u8; 12];

                // Corners (7 explicit, 8th derived by parity)
                let mut cp_sum: u32 = 0;
                let mut co_sum: u32 = 0;
                for i in 0..7 {
                    let p = msg.get(12 + i * 3, 3);
                    let o = msg.get(33 + i * 2, 2);
                    cp[i] = p as u8;
                    co[i] = o as u8;
                    cp_sum += p;
                    co_sum += o;
                }
                cp[7] = (28 - cp_sum) as u8;
                co[7] = ((3 - (co_sum % 3)) % 3) as u8;

                // Edges (11 explicit, 12th derived by parity)
                let mut ep_sum: u32 = 0;
                let mut eo_sum: u32 = 0;
                for i in 0..11 {
                    let p = msg.get(47 + i * 4, 4);
                    let o = msg.get(91 + i, 1);
                    ep[i] = p as u8;
                    eo[i] = o as u8;
                    ep_sum += p;
                    eo_sum += o;
                }
                ep[11] = (66 - ep_sum) as u8;
                eo[11] = ((2 - (eo_sum % 2)) % 2) as u8;

                events.push(CubeEvent::Facelets {
                    serial: serial as u16,
                    cp,
                    co,
                    ep,
                    eo,
                });
            }
            0x05 => {
                // HARDWARE
                let hw_major = msg.get(8, 8);
                let hw_minor = msg.get(16, 8);
                let sw_major = msg.get(24, 8);
                let sw_minor = msg.get(32, 8);
                let gyro_supported = msg.get(104, 1) != 0;

                let mut hw_name = String::new();
                for i in 0..8 {
                    let ch = msg.get(40 + i * 8, 8) as u8;
                    if ch != 0 {
                        hw_name.push(ch as char);
                    }
                }

                events.push(CubeEvent::Hardware {
                    name: hw_name,
                    hw_version: format!("{}.{}", hw_major, hw_minor),
                    sw_version: format!("{}.{}", sw_major, sw_minor),
                    gyro_supported,
                });
            }
            0x09 => {
                // BATTERY
                let level = msg.get(8, 8).min(100) as u8;
                events.push(CubeEvent::Battery { level });
            }
            0x0D => {
                // DISCONNECT
                events.push(CubeEvent::Disconnect);
            }
            _ => {}
        }

        events
    }

    fn create_command(&self, cmd: CubeCommand) -> Option<Vec<u8>> {
        let mut msg = vec![0u8; 20];
        match cmd {
            CubeCommand::RequestFacelets => { msg[0] = 0x04; }
            CubeCommand::RequestHardware => { msg[0] = 0x05; }
            CubeCommand::RequestBattery => { msg[0] = 0x09; }
            CubeCommand::RequestReset => {
                msg.copy_from_slice(&[
                    0x0A, 0x05, 0x39, 0x77, 0x00, 0x00, 0x01, 0x23,
                    0x45, 0x67, 0x89, 0xAB, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00,
                ]);
            }
        }
        Some(msg)
    }

    fn has_gyro(&self) -> bool { true }
    fn requires_handshake(&self) -> bool { false }
}
