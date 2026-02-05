use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter, WriteType};
use btleplug::platform::Manager;
use futures::stream::StreamExt;
use std::error::Error;
use std::time::Duration;
use tokio::time;
use uuid::Uuid;
use aes::Aes128;
use cbc::Decryptor;
use cipher::{BlockDecryptMut, KeyIvInit};

// --- MOYU V10 SECRETS (EXTRACTED) ---
const MASTER_KEY: [u8; 16] = [21, 119, 58, 92, 103, 14, 45, 31, 23, 103, 42, 19, 155, 103, 82, 87];
const MASTER_IV: [u8; 16] = [17, 35, 38, 37, 134, 42, 44, 59, 85, 6, 127, 49, 126, 103, 33, 87];

type Aes128CbcDec = Decryptor<Aes128>;

struct MoyuV10Cipher {
    device_key: [u8; 16],
    device_iv: [u8; 16],
}

impl MoyuV10Cipher {
    fn new(mac_str: &str) -> Self {
        // Parse MAC CF:30:16:01:C7:2F -> [207, 48, 22, 1, 199, 47]
        let mac_bytes: Vec<u8> = mac_str
            .split(':')
            .map(|s| u8::from_str_radix(s, 16).unwrap_or(0))
            .collect();
        
        let mut device_key = MASTER_KEY;
        let mut device_iv = MASTER_IV;

        // Apply MoYu V3 derivation logic (mod 255)
        for i in 0..6 {
            let m = mac_bytes[5 - i];
            device_key[i] = ((MASTER_KEY[i] as u16 + m as u16) % 255) as u8;
            device_iv[i] = ((MASTER_IV[i] as u16 + m as u16) % 255) as u8;
        }

        Self { device_key, device_iv }
    }

    fn decrypt(&self, data: &[u8]) -> Vec<u8> {
        let mut buffer = data.to_vec();
        
        // --- PASS 1: Decrypt 16 bytes at end (index 4 to 20) ---
        if buffer.len() >= 20 {
            let mut block = [0u8; 16];
            block.copy_from_slice(&buffer[4..20]);
            let dec = Aes128CbcDec::new(&self.device_key.into(), &self.device_iv.into());
            dec.decrypt_padded_mut::<cipher::block_padding::NoPadding>(&mut block).unwrap();
            buffer[4..20].copy_from_slice(&block);
        }

        // --- PASS 2: Decrypt 16 bytes at start (index 0 to 16) ---
        if buffer.len() >= 16 {
            let mut block = [0u8; 16];
            block.copy_from_slice(&buffer[0..16]);
            let dec = Aes128CbcDec::new(&self.device_key.into(), &self.device_iv.into());
            dec.decrypt_padded_mut::<cipher::block_padding::NoPadding>(&mut block).unwrap();
            buffer[0..16].copy_from_slice(&block);
        }

        buffer
    }
}

// Configuration
const DEVICE_NAME: &str = "WCU_MY32_C72F"; // Change to your cube name suffix
const MAC_ADDRESS: &str = "CF:30:16:01:C7:2F";

// Handshake Playloads (Raw as captured)
const PAYLOADS: [[u8; 20]; 4] = [
    [0x95, 0x53, 0x0d, 0x6c, 0xdc, 0x06, 0xc3, 0x25, 0xbc, 0x21, 0xdb, 0x70, 0xa6, 0x4f, 0xe4, 0x00, 0x3d, 0x98, 0x0c, 0x5f],
    [0x92, 0x93, 0xa7, 0xd6, 0x36, 0x62, 0x51, 0x7d, 0x8d, 0xdd, 0xa7, 0x53, 0x30, 0x3b, 0x9a, 0xa4, 0x69, 0xed, 0x6a, 0xa0],
    [0xcd, 0xd8, 0x21, 0x93, 0x3e, 0x79, 0xa9, 0x6c, 0x92, 0x4f, 0x57, 0x4a, 0x1c, 0xc4, 0xa8, 0xd8, 0x09, 0xea, 0x8f, 0xee],
    [0x90, 0x5c, 0x36, 0x16, 0x3b, 0x6c, 0xbf, 0x34, 0x8c, 0x8b, 0x54, 0xf7, 0xa4, 0xf3, 0x7f, 0xca, 0xa8, 0x61, 0x10, 0xff],
];

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🚀 RouxFlow - MoYu V10 Protocol Driver (Rust)");

    let cipher = MoyuV10Cipher::new(MAC_ADDRESS);
    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;
    let adapter = adapters.into_iter().next().ok_or("No adapter")?;
    
    println!("Scanning for {}...", DEVICE_NAME);
    adapter.start_scan(ScanFilter::default()).await?;
    time::sleep(Duration::from_secs(2)).await;

    let target = adapter.peripherals().await?.into_iter().find(|p| {
        futures::executor::block_on(async {
            p.properties().await.ok().flatten().and_then(|pr| pr.local_name)
                .map(|n| n.contains("WCU_MY32"))
                .unwrap_or(false)
        })
    }).ok_or("Cube not found")?;

    target.connect().await?;
    target.discover_services().await?;

    let chars = target.characteristics();
    let read_uuid = Uuid::parse_str("0783b03e-7735-b5a0-1760-a305d2795cb1")?;
    let write_uuid = Uuid::parse_str("0783b03e-7735-b5a0-1760-a305d2795cb2")?;
    let char_read = chars.iter().find(|c| c.uuid == read_uuid).ok_or("No read char")?;
    let char_write = chars.iter().find(|c| c.uuid == write_uuid).ok_or("No write char")?;

    target.subscribe(char_read).await?;
    let mut notification_stream = target.notifications().await?;

    println!("Performing MINIMAL Handshake (P1 Only)...");
    // Send only P1 (A0 00...)
    target.write(char_write, &PAYLOADS[0], WriteType::WithoutResponse).await?;
    println!("  TX: P1 sent (Hello)");
    time::sleep(Duration::from_millis(400)).await;

    println!("Listening... (Gyro logged to 'gyro_log.csv')");
    let start_time = std::time::Instant::now();
    let mut gyro_file = std::fs::File::create("gyro_log.csv")?;
    use std::io::Write;
    writeln!(gyro_file, "Timestamp,Q0,Q1,Q2,Q3,RawHex")?;
    
    while start_time.elapsed() < Duration::from_secs(10) {
        if let Some(n) = notification_stream.next().await {
            let dec = cipher.decrypt(&n.value);
            let opcode = dec[0];
            
            if opcode == 0xAB {
                // Log Gyro to file ONLY
                if dec.len() >= 17 {
                    let floats: Vec<f32> = (0..4).map(|i| {
                        f32::from_le_bytes(dec[1+i*4..5+i*4].try_into().unwrap())
                    }).collect();
                    writeln!(gyro_file, "{:?},{:.4},{:.4},{:.4},{:.4},{}", 
                        start_time.elapsed(), floats[0], floats[1], floats[2], floats[3], hex::encode(&dec))?;
                }
            } else {
                // Show other important packets in Console
                let label = match opcode {
                    0xA1 => "INFO ",
                    0xA3 => "STATE",
                    0xA4 => "BATT ",
                    0xA5 => "MOVE ",
                    _    => "DATA ",
                };
                println!("[{}] {}", label, hex::encode(&dec));
            }
        }
    }

    target.disconnect().await?;
    println!("Test finished.");
    Ok(())
}
