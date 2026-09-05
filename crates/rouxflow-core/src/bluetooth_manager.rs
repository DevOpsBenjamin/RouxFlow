use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub name: String,
    pub mac_address: String,
    pub protocol_name: String,
    pub has_gyro: bool,
    pub battery_level: Option<u8>,
    pub sw_version: Option<String>,
    pub hw_version: Option<String>,
}

pub struct BluetoothManager {
    connection_state: ConnectionState,
    device_info: Option<DeviceInfo>,
}

impl BluetoothManager {
    pub fn new() -> Self {
        Self {
            connection_state: ConnectionState::Disconnected,
            device_info: None,
        }
    }

    pub fn connect(&mut self, name: String, mac_address: String, protocol_name: String, has_gyro: bool) {
        self.connection_state = ConnectionState::Connected;
        self.device_info = Some(DeviceInfo {
            name,
            mac_address,
            protocol_name,
            has_gyro,
            battery_level: None,
            sw_version: None,
            hw_version: None,
        });
    }

    pub fn disconnect(&mut self) {
        self.connection_state = ConnectionState::Disconnected;
        self.device_info = None;
    }

    pub fn is_connected(&self) -> bool {
        self.connection_state == ConnectionState::Connected
    }

    pub fn get_device_info_json(&self) -> Option<String> {
        self.device_info.as_ref().and_then(|info| serde_json::to_string(info).ok())
    }

    pub fn update_battery(&mut self, level: u8) {
        if let Some(device_info) = &mut self.device_info {
            device_info.battery_level = Some(level);
        }
    }

    pub fn update_hardware(&mut self, sw_version: String, hw_version: String) {
        if let Some(device_info) = &mut self.device_info {
            device_info.sw_version = Some(sw_version);
            device_info.hw_version = Some(hw_version);
        }
    }

    /// Check if a protocol requires MAC address for encryption
    pub fn protocol_requires_mac(protocol: &str) -> bool {
        matches!(protocol, "MoYuAi" | "MoYuV3" | "GanV2" | "GanV3" | "GanV4")
    }

    /// Check if device_id is a valid MAC address format (XX:XX:XX:XX:XX:XX)
    pub fn is_valid_mac_format(device_id: &str) -> bool {
        if device_id.len() != 17 {
            return false;
        }

        let parts: Vec<&str> = device_id.split(':').collect();
        if parts.len() != 6 {
            return false;
        }

        parts.iter().all(|part| {
            part.len() == 2 && part.chars().all(|c| c.is_ascii_hexdigit())
        })
    }

    /// Determine if we need to prompt user for MAC address
    pub fn needs_mac_input(device_id: &str, protocol: &str) -> bool {
        Self::protocol_requires_mac(protocol) && !Self::is_valid_mac_format(device_id)
    }
}

impl Default for BluetoothManager {
    fn default() -> Self {
        Self::new()
    }
}
