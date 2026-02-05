use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter, WriteType};
use btleplug::platform::Manager;
use futures::stream::StreamExt;
use std::error::Error;
use std::time::Duration;
use tokio::time;
use uuid::Uuid;

use rouxflow_bluetoothcube::{
    find_cube_by_ble_name, create_protocol,
    CubeEvent, CubeCommand,
};

// Configuration — keep hardcoded for this test tool
const DEVICE_NAME: &str = "WCU_MY32_C72F";
const MAC_ADDRESS: &str = "CF:30:16:01:C7:2F";

// Original captured handshake payloads (encrypted, for comparison)
const PAYLOADS: [[u8; 20]; 4] = [
    [0x95, 0x53, 0x0d, 0x6c, 0xdc, 0x06, 0xc3, 0x25, 0xbc, 0x21, 0xdb, 0x70, 0xa6, 0x4f, 0xe4, 0x00, 0x3d, 0x98, 0x0c, 0x5f],
    [0x92, 0x93, 0xa7, 0xd6, 0x36, 0x62, 0x51, 0x7d, 0x8d, 0xdd, 0xa7, 0x53, 0x30, 0x3b, 0x9a, 0xa4, 0x69, 0xed, 0x6a, 0xa0],
    [0xcd, 0xd8, 0x21, 0x93, 0x3e, 0x79, 0xa9, 0x6c, 0x92, 0x4f, 0x57, 0x4a, 0x1c, 0xc4, 0xa8, 0xd8, 0x09, 0xea, 0x8f, 0xee],
    [0x90, 0x5c, 0x36, 0x16, 0x3b, 0x6c, 0xbf, 0x34, 0x8c, 0x8b, 0x54, 0xf7, 0xa4, 0xf3, 0x7f, 0xca, 0xa8, 0x61, 0x10, 0xff],
];

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // --- Cube lookup via bluetoothcube registry ---
    let cube_def = find_cube_by_ble_name(DEVICE_NAME)
        .expect("Cube not found in registry");

    println!("Cube: {} (protocol: {:?}, status: {:?})",
        cube_def.name, cube_def.protocol, cube_def.status);
    println!("Features: {:?}", cube_def.features);

    // --- Protocol codec from the crate ---
    let mut codec = create_protocol(cube_def.protocol, MAC_ADDRESS);
    println!("Protocol: {} (gyro={}, handshake={})",
        codec.name(), codec.has_gyro(), codec.requires_handshake());

    // --- BLE UUIDs from protocol definition ---
    let ble = cube_def.protocol.ble_profile();
    println!("BLE service:  {}", ble.service_uuid);
    println!("BLE read:     {}", ble.state_characteristic);
    println!("BLE write:    {}", ble.command_characteristic);

    // --- BLE connection via btleplug ---
    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;
    let adapter = adapters.into_iter().next().ok_or("No BLE adapter")?;

    println!("\nScanning for {}...", DEVICE_NAME);
    adapter.start_scan(ScanFilter::default()).await?;
    time::sleep(Duration::from_secs(2)).await;

    let prefix = cube_def.ble_name_prefixes.first()
        .expect("Cube has no BLE name prefix");

    let target = adapter.peripherals().await?.into_iter().find(|p| {
        futures::executor::block_on(async {
            p.properties().await.ok().flatten().and_then(|pr| pr.local_name)
                .map(|n| n.starts_with(prefix))
                .unwrap_or(false)
        })
    }).ok_or("Cube not found on BLE scan")?;

    target.connect().await?;
    target.discover_services().await?;
    println!("Connected!");

    let chars = target.characteristics();
    let read_uuid = Uuid::parse_str(ble.state_characteristic)?;
    let write_uuid = Uuid::parse_str(ble.command_characteristic)?;
    let char_read = chars.iter().find(|c| c.uuid == read_uuid).ok_or("No read char")?;
    let char_write = chars.iter().find(|c| c.uuid == write_uuid).ok_or("No write char")?;

    target.subscribe(char_read).await?;
    let mut notification_stream = target.notifications().await?;

    // --- Handshake ---
    if codec.requires_handshake() {
        if let Some(handshake_plain) = codec.handshake_data() {
            let handshake_enc = codec.encrypt(&handshake_plain);
            println!("TX handshake: {}", hex::encode(&handshake_enc));
            target.write(char_write, &handshake_enc, WriteType::WithoutResponse).await?;
        }
    }
    time::sleep(Duration::from_millis(400)).await;

    // --- Request battery + hardware + facelets ---
    for cmd in [CubeCommand::RequestBattery, CubeCommand::RequestHardware, CubeCommand::RequestFacelets] {
        if let Some(payload) = codec.create_command(cmd) {
            let enc = codec.encrypt(&payload);
            println!("TX {:?}: {}", cmd, hex::encode(&enc));
            target.write(char_write, &enc, WriteType::WithoutResponse).await?;
            time::sleep(Duration::from_millis(200)).await;
        }
    }

    // --- Listen and decode events ---
    println!("\nListening for 10s... (do some moves!)\n");
    let start_time = std::time::Instant::now();
    let mut gyro_count: u32 = 0;
    let mut gyro_shown: u32 = 0;

    while start_time.elapsed() < Duration::from_secs(10) {
        if let Some(n) = notification_stream.next().await {
            let raw_hex = hex::encode(&n.value);
            let decrypted = codec.decrypt(&n.value);
            let dec_hex = hex::encode(&decrypted);
            let opcode = decrypted[0];
            let events = codec.decode_event(&decrypted);

            if events.is_empty() {
                // Show raw with opcode label
                let label = match opcode {
                    0xA1 => "INFO", 0xA3 => "STATE", 0xA4 => "BATT",
                    0xA5 => "MOVE", 0xAB => "GYRO", 0xAA => "INIT",
                    _ => "????",
                };
                println!("[RAW:{label}] {dec_hex}");
                continue;
            }

            for event in events {
                match &event {
                    CubeEvent::Move { face, direction, cube_timestamp, serial } => {
                        let dir_str = match direction {
                            1 => "CW",
                            -1 => "CCW",
                            2 => "2",
                            _ => "?",
                        };
                        println!("[MOVE ] {:?} {} serial={} ts={:?} | {}",
                            face, dir_str, serial,
                            cube_timestamp, dec_hex);
                    }
                    CubeEvent::Gyro { quaternion, velocity } => {
                        gyro_count += 1;
                        // Show first 5 gyro with full hex, then just count
                        if gyro_shown < 5 {
                            gyro_shown += 1;
                            println!("[GYRO ] w={:.4} x={:.4} y={:.4} z={:.4} | {}",
                                quaternion.w, quaternion.x, quaternion.y, quaternion.z, dec_hex);
                            if let Some(v) = velocity {
                                println!("        vel=({:.2}, {:.2}, {:.2})", v.x, v.y, v.z);
                            }
                        }
                    }
                    CubeEvent::Battery { level } => {
                        println!("[BATT ] {}% | {}", level, dec_hex);
                    }
                    CubeEvent::Hardware { name, sw_version, hw_version, gyro_supported } => {
                        println!("[HW   ] {} sw={} hw={} gyro={} | {}",
                            name, sw_version, hw_version, gyro_supported, dec_hex);
                    }
                    CubeEvent::Facelets { cp, co, ep, eo, .. } => {
                        println!("[FACE ] cp={:?} co={:?}", cp, co);
                        println!("        ep={:?} eo={:?}", ep, eo);
                        println!("        | {}", dec_hex);
                    }
                    CubeEvent::Disconnect => {
                        println!("[DISC ] Cube disconnected");
                    }
                    _ => {
                        println!("[EVT  ] {:?} | {}", event, dec_hex);
                    }
                }
            }
        }
    }

    if gyro_count > 0 {
        println!("\n(Total gyro packets: {}, shown: {})", gyro_count, gyro_shown);
    }

    target.disconnect().await?;
    println!("\nDone.");
    Ok(())
}
