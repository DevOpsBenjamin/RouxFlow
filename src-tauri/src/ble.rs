use btleplug::api::{Central, Manager as BleManager, Peripheral, ScanFilter};
use btleplug::platform::{Adapter, Manager, Peripheral as BlePeripheral};
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager as TauriManager, Runtime};
use futures::stream::StreamExt;

pub struct BleState {
    pub adapter: Adapter,
    pub connected_peripheral: Arc<Mutex<Option<BlePeripheral>>>,
}

pub async fn init_ble() -> Result<Adapter, Box<dyn Error>> {
    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;
    let adapter = adapters.into_iter().next().ok_or("No Bluetooth adapter found")?;
    Ok(adapter)
}

#[tauri::command]
pub async fn ble_check_available<R: Runtime>(
    app: AppHandle<R>,
) -> Result<bool, String> {
    // Check if BleState is already initialized
    if let Some(_) = TauriManager::try_state::<BleState>(&app) {
        return Ok(true);
    }

    // Attempt to initialize if missing
    match init_ble().await {
        Ok(adapter) => {
            app.manage(BleState {
                adapter,
                connected_peripheral: Arc::new(Mutex::new(None)),
            });
            Ok(true)
        },
        Err(_) => {
            Err("Bluetooth unavailable. Please enable Bluetooth and try again.".into())
        }
    }
}

#[tauri::command]
pub async fn ble_scan<R: Runtime>(
    state: tauri::State<'_, BleState>,
    _app: AppHandle<R>,
) -> Result<(), String> {
    state.adapter
        .start_scan(ScanFilter::default())
        .await
        .map_err(|e| format!("Failed to start Bluetooth scan: {}", e))?;
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

    for peripheral in peripherals {
        let properties = peripheral.properties().await.map_err(|e| e.to_string())?.unwrap_or_default();
        if let Some(name) = properties.local_name {
            devices.push(DeviceInfo {
                id: peripheral.id().to_string(),
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
    let peripherals = state.adapter.peripherals().await.map_err(|e| e.to_string())?;
    
    for peripheral in peripherals {
        if peripheral.id().to_string() == id {
            peripheral.connect().await.map_err(|e| e.to_string())?;
            peripheral.discover_services().await.map_err(|e| e.to_string())?;
            
            let mut connected = state.connected_peripheral.lock().await;
            *connected = Some(peripheral.clone());

            let app_clone = app.clone();
            let peripheral_clone = peripheral.clone();
            
            tokio::spawn(async move {
                let chars = peripheral_clone.characteristics();
                let gan_char_uuid = uuid::uuid!("0000fe52-0000-1000-8000-00805f9b34fb");
                
                if let Some(c) = chars.iter().find(|c| c.uuid == gan_char_uuid) {
                    if let Ok(_) = peripheral_clone.subscribe(c).await {
                        if let Ok(mut notification_stream) = peripheral_clone.notifications().await {
                            while let Some(data) = notification_stream.next().await {
                                let _ = app_clone.emit("ble-packet", data.value);
                            }
                        }
                    }
                }
            });

            return Ok(());
        }
    }

    Err("Device not found".into())
}
