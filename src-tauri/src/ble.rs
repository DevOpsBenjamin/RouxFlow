use tauri::{AppHandle, Manager as TauriManager, Runtime, Emitter};
use futures::stream::StreamExt;
use btleplug::api::{Central, Manager as BtleplugManager, Peripheral, ScanFilter, WriteType};
use btleplug::platform::{Adapter, Peripheral as PlatformPeripheral, Manager as PlatformManager};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashSet;

pub struct BleState {
    pub adapter: Adapter,
    pub connected_peripheral: Arc<Mutex<Option<PlatformPeripheral>>>,
    pub logged_device_ids: Arc<Mutex<HashSet<String>>>,
    pub scan_started_logged: Arc<Mutex<bool>>,
}

#[tauri::command]
pub async fn ble_check_available(_state: tauri::State<'_, BleState>) -> Result<bool, String> {
    Ok(true)
}

#[tauri::command]
pub async fn ble_scan<R: Runtime>(
    state: tauri::State<'_, BleState>,
    _app: AppHandle<R>,
) -> Result<(), String> {
    let mut logged = state.scan_started_logged.lock().await;
    if !*logged {
        println!("[BLE] Starting scan...");
        *logged = true;
    }
    
    state.adapter
        .start_scan(ScanFilter::default())
        .await
        .map_err(|e| {
            println!("[BLE] Scan start failed: {}", e);
            format!("Failed to start Bluetooth scan: {}", e)
        })?;
    Ok(())
}

#[derive(serde::Serialize)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    pub rssi: i16,
}

#[tauri::command]
pub async fn ble_list_devices(
    state: tauri::State<'_, BleState>,
) -> Result<Vec<DeviceInfo>, String> {
    let peripherals = state.adapter.peripherals().await.map_err(|e| e.to_string())?;
    let mut devices = Vec::new();
    let mut logged = state.logged_device_ids.lock().await;

    for peripheral in peripherals {
        let properties = peripheral.properties().await.map_err(|e| e.to_string())?.unwrap_or_default();
        let id = peripheral.id().to_string();

        if let Some(name) = properties.local_name {
            if !logged.contains(&id) {
                println!("[BLE] Found device: {} (MAC: {})", name, id);
                logged.insert(id.clone());
            }

            devices.push(DeviceInfo {
                id: id.clone(),
                name,
                rssi: properties.rssi.unwrap_or(0),
            });
        }
    }

    Ok(devices)
}

#[tauri::command]
pub async fn ble_connect<R: Runtime>(
    state: tauri::State<'_, BleState>,
    app: AppHandle<R>,
    id: String,
) -> Result<(), String> {
    println!("[BLE] Attempting to connect to: {}", id);
    let peripherals = state.adapter.peripherals().await.map_err(|e| e.to_string())?;
    
    for peripheral in peripherals {
        if peripheral.id().to_string() == id {
            let mut connected_slot = state.connected_peripheral.lock().await;
            
            // If already connected, we'll try to re-init instead of skipping
            if !peripheral.is_connected().await.unwrap_or(false) {
                println!("[BLE] Connecting to peripheral...");
                peripheral.connect().await.map_err(|e| {
                    println!("[BLE] Connection failed: {}", e);
                    e.to_string()
                })?;
            } else {
                println!("[BLE] Already connected. Re-initializing services/notifications...");
            }
            
            peripheral.discover_services().await.map_err(|e| {
                println!("[BLE] Service discovery failed: {}", e);
                e.to_string()
            })?;
            
            *connected_slot = Some(peripheral.clone());

            let app_clone = app.clone();
            let peripheral_clone = peripheral.clone();
            let chars = peripheral.characteristics();
            
            tokio::spawn(async move {
                let gan_char_uuid = uuid::uuid!("0000fe52-0000-1000-8000-00805f9b34fb");
                let moyu_write_uuid = uuid::uuid!("02f00000-0000-0000-0000-00000000ff01");
                let moyu_notify_prefix = "02f00000-0000-0000-0000-00000000ff";
                
                let target_char = chars.iter().find(|c| c.uuid == gan_char_uuid)
                    .or_else(|| {
                        chars.iter().find(|c| {
                            let uuid_str = c.uuid.to_string();
                            uuid_str.starts_with(moyu_notify_prefix) && 
                            (c.properties.contains(btleplug::api::CharPropFlags::NOTIFY) || 
                             c.properties.contains(btleplug::api::CharPropFlags::INDICATE))
                        })
                    });

                let write_char = chars.iter().find(|c| c.uuid == moyu_write_uuid)
                    .or_else(|| {
                        chars.iter().find(|c| c.properties.contains(btleplug::api::CharPropFlags::WRITE))
                    });

                if let Some(c) = target_char {
                    // Always try to subscribe (it will just return Ok if already subscribed)
                    if let Err(e) = peripheral_clone.subscribe(c).await {
                        println!("[BLE] Subscription info (might already be active): {}", e);
                    }
                    
                    println!("[BLE] Subscribed to {}!", c.uuid);
                    
                    // Force Wake-up Sequence
                    if let Some(wc) = write_char {
                        println!("[BLE] Sending MoYu activation sequence...");
                        let _ = peripheral_clone.write(wc, &[0x05], WriteType::WithoutResponse).await;
                        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                        let _ = peripheral_clone.write(wc, &[0x01], WriteType::WithoutResponse).await;
                        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                        let _ = peripheral_clone.write(wc, &[0x03], WriteType::WithoutResponse).await;
                        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                        let _ = peripheral_clone.write(wc, &[0x04], WriteType::WithoutResponse).await;
                        let _ = peripheral_clone.write(wc, &[0x02], WriteType::WithoutResponse).await;
                    }

                    if let Ok(mut notification_stream) = peripheral_clone.notifications().await {
                        println!("[BLE] Listener loop active.");
                        while let Some(data) = notification_stream.next().await {
                            #[derive(serde::Serialize, Clone)]
                            struct PacketPayload { id: String, data: Vec<u8> }
                            let _ = app_clone.emit("ble-packet", PacketPayload {
                                id: peripheral_clone.id().to_string(),
                                data: data.value,
                            });
                        }
                    }
                }
            });

            return Ok(());
        }
    }

    println!("[BLE] Device {} not found", id);
    Err("Device not found".into())
}

pub async fn init_ble() -> Result<BleState, String> {
    let manager = PlatformManager::new().await.map_err(|e| e.to_string())?;
    let adapters = manager.adapters().await.map_err(|e| e.to_string())?;
    let adapter = adapters.into_iter().next().ok_or("No Bluetooth adapter found")?;

    Ok(BleState {
        adapter,
        connected_peripheral: Arc::new(Mutex::new(None)),
        logged_device_ids: Arc::new(Mutex::new(HashSet::new())),
        scan_started_logged: Arc::new(Mutex::new(false)),
    })
}
