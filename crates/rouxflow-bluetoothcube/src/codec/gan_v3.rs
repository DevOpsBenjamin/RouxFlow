//! GAN Gen3 protocol codec.
//!
//! Supports: GAN 356 i Carry 2.
//!
//! 16-byte messages with magic=0x55 header. Includes move history recovery
//! via FIFO buffer and serial tracking.

use super::{
    BitView, CubeCommand, CubeEvent, CubeProtocol,
    derive_gan_keys, gan_decrypt, gan_encrypt,
};
use crate::protocol::EncryptionKeys;
use rouxflow_core::cube::Face;

/// Face lookup table for Gen3 move encoding.
/// The face byte encodes as bitmask: U=1, R=2, F=4, D=8, L=16, B=32.
const GEN3_FACE_BITMASKS: [u32; 6] = [2, 32, 8, 1, 16, 4];

/// Face lookup for move history (different encoding).
/// History uses: U=0, R=1, F=2, D=3, L=4, B=5 mapped from [1,5,3,0,4,2].
const GEN3_HISTORY_FACE_MAP: [u32; 6] = [1, 5, 3, 0, 4, 2];

fn face_from_gen3_bitmask(bitmask: u32) -> Option<Face> {
    let idx = GEN3_FACE_BITMASKS.iter().position(|&m| m == bitmask)?;
    super::face_from_index(idx as u32)
}

fn face_from_gen3_history(val: u32) -> Option<Face> {
    let idx = GEN3_HISTORY_FACE_MAP.iter().position(|&m| m == val)?;
    super::face_from_index(idx as u32)
}

pub struct GanV3Codec {
    key: [u8; 16],
    iv: [u8; 16],
    serial: i32,
    last_serial: i32,
    /// FIFO buffer for move ordering with history recovery
    move_buffer: Vec<BufferedMove>,
}

#[derive(Clone)]
struct BufferedMove {
    serial: u16,
    event: CubeEvent,
}

impl GanV3Codec {
    pub fn new(keys: &EncryptionKeys, mac_address: &str) -> Self {
        let (key, iv) = derive_gan_keys(keys, mac_address);
        Self {
            key,
            iv,
            serial: -1,
            last_serial: -1,
            move_buffer: Vec::new(),
        }
    }

    /// Evict moves from the FIFO buffer in serial order.
    /// Stops when a gap is detected (missing serial).
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
                // Gap detected — need move history to fill it
                break;
            }

            let entry = self.move_buffer.remove(0);
            self.last_serial = entry.serial as i32;
            evicted.push(entry.event);
        }

        // Safety: if buffer grows too large, something went wrong
        if self.move_buffer.len() > 16 {
            evicted.push(CubeEvent::Disconnect);
            self.move_buffer.clear();
        }

        evicted
    }

    /// Check if a serial fits in the circular range (start, end).
    fn is_serial_in_range(&self, start: i32, end: i32, serial: i32, closed_start: bool, closed_end: bool) -> bool {
        let range = (end - start) & 0xFF;
        let offset = (serial - start) & 0xFF;
        range >= offset
            && (closed_start || ((start - serial) & 0xFF) > 0)
            && (closed_end || ((end - serial) & 0xFF) > 0)
    }

    /// Inject a recovered move into the FIFO buffer at the correct position.
    fn inject_missed_move(&mut self, serial: u16, event: CubeEvent) {
        // Skip if already in buffer
        if self.move_buffer.iter().any(|e| e.serial == serial) {
            return;
        }

        if !self.move_buffer.is_empty() {
            let head_serial = self.move_buffer[0].serial as i32;
            // Must fit between last evicted and buffer head
            if !self.is_serial_in_range(self.last_serial, head_serial, serial as i32, false, false) {
                return;
            }
            // Insert at head if it's the predecessor
            if serial == ((head_serial - 1) & 0xFF) as u16 {
                self.move_buffer.insert(0, BufferedMove { serial, event });
            }
        } else {
            // Empty buffer — insert if in range between last evicted and current serial
            if self.is_serial_in_range(self.last_serial, self.serial, serial as i32, false, true) {
                self.move_buffer.insert(0, BufferedMove { serial, event });
            }
        }
    }
}

impl CubeProtocol for GanV3Codec {
    fn name(&self) -> &str { "GAN Gen3" }

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
        let magic = msg.get(0, 8);
        let event_type = msg.get(8, 8);
        let data_length = msg.get(16, 8);

        if magic != 0x55 || data_length == 0 {
            return vec![];
        }

        match event_type {
            0x01 => {
                // MOVE
                if self.last_serial == -1 {
                    return vec![];
                }

                let cube_timestamp = msg.get_endian(24, 32, true);
                let serial = msg.get_endian(56, 16, true) as u16;
                self.serial = serial as i32;

                let direction = msg.get(72, 2);
                let face_bitmask = msg.get(74, 6);

                if let Some(face) = face_from_gen3_bitmask(face_bitmask) {
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
            0x06 => {
                // MOVE_HISTORY
                let start_serial = msg.get(24, 8);
                let count = ((data_length - 1) * 2) as usize;

                for i in 0..count {
                    let face_val = msg.get(32 + 4 * i, 3);
                    let direction_bit = msg.get(35 + 4 * i, 1);

                    if let Some(face) = face_from_gen3_history(face_val) {
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
            0x02 => {
                // FACELETS
                let serial = msg.get_endian(24, 16, true) as u16;
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
                    let p = msg.get(40 + i * 3, 3);
                    let o = msg.get(61 + i * 2, 2);
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
                    let p = msg.get(77 + i * 4, 4);
                    let o = msg.get(121 + i, 1);
                    ep[i] = p as u8;
                    eo[i] = o as u8;
                    ep_sum += p;
                    eo_sum += o;
                }
                ep[11] = (66 - ep_sum) as u8;
                eo[11] = ((2 - (eo_sum % 2)) % 2) as u8;

                vec![CubeEvent::Facelets { serial, cp, co, ep, eo }]
            }
            0x07 => {
                // HARDWARE
                let sw_major = msg.get(72, 4);
                let sw_minor = msg.get(76, 4);
                let hw_major = msg.get(80, 4);
                let hw_minor = msg.get(84, 4);

                let mut hw_name = String::new();
                for i in 0..5 {
                    let ch = msg.get(32 + i * 8, 8) as u8;
                    if ch != 0 {
                        hw_name.push(ch as char);
                    }
                }

                vec![CubeEvent::Hardware {
                    name: hw_name,
                    hw_version: format!("{}.{}", hw_major, hw_minor),
                    sw_version: format!("{}.{}", sw_major, sw_minor),
                    gyro_supported: false,
                }]
            }
            0x10 => {
                // BATTERY
                let level = msg.get(24, 8).min(100) as u8;
                vec![CubeEvent::Battery { level }]
            }
            0x11 => {
                // DISCONNECT
                vec![CubeEvent::Disconnect]
            }
            _ => vec![],
        }
    }

    fn create_command(&self, cmd: CubeCommand) -> Option<Vec<u8>> {
        let mut msg = vec![0u8; 16];
        match cmd {
            CubeCommand::RequestFacelets => {
                msg[0] = 0x68;
                msg[1] = 0x01;
            }
            CubeCommand::RequestHardware => {
                msg[0] = 0x68;
                msg[1] = 0x04;
            }
            CubeCommand::RequestBattery => {
                msg[0] = 0x68;
                msg[1] = 0x07;
            }
            CubeCommand::RequestReset => {
                let reset = [
                    0x68, 0x05, 0x05, 0x39, 0x77, 0x00, 0x00, 0x01,
                    0x23, 0x45, 0x67, 0x89, 0xAB, 0x00, 0x00, 0x00,
                ];
                msg.copy_from_slice(&reset);
            }
        }
        Some(msg)
    }

    fn has_gyro(&self) -> bool { false }
    fn requires_handshake(&self) -> bool { false }
}
