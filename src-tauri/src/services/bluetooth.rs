use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BluetoothDevice {
    pub name: String,
    pub address: String,
    pub device_type: String,
    pub battery_percent: Option<u8>,
    pub is_connected: bool,
    pub is_paired: bool,
    pub vendor: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BluetoothInfo {
    pub enabled: bool,
    pub discoverable: bool,
    pub address: Option<String>,
    pub devices: Vec<BluetoothDevice>,
}

pub struct BluetoothService;

impl BluetoothService {
    pub fn new() -> Self {
        Self
    }

    pub fn get_bluetooth_info(&self) -> Result<BluetoothInfo, String> {
        let output = Command::new("system_profiler")
            .args(["SPBluetoothDataType", "-json"])
            .output()
            .map_err(|e| format!("Failed to run system_profiler: {}", e))?;

        if !output.status.success() {
            return Err("system_profiler command failed".to_string());
        }

        let json_str = String::from_utf8_lossy(&output.stdout);
        let json: Value = serde_json::from_str(&json_str)
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;

        self.parse_bluetooth_data(&json)
    }

    fn parse_bluetooth_data(&self, json: &Value) -> Result<BluetoothInfo, String> {
        let bt_data = json.get("SPBluetoothDataType")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .ok_or("No Bluetooth data found")?;

        // Parse controller info
        let controller = bt_data.get("controller_properties");

        let enabled = controller
            .and_then(|c| c.get("controller_state"))
            .and_then(|s| s.as_str())
            .map(|s| s == "attrib_on")
            .unwrap_or(false);

        let discoverable = controller
            .and_then(|c| c.get("controller_discoverable"))
            .and_then(|s| s.as_str())
            .map(|s| s == "attrib_on")
            .unwrap_or(false);

        let address = controller
            .and_then(|c| c.get("controller_address"))
            .and_then(|s| s.as_str())
            .map(|s| s.to_string());

        // Parse connected devices
        let mut devices = Vec::new();

        // Check for devices under different possible keys
        let device_keys = [
            "device_connected",
            "devices_list",
            "device_title",
        ];

        for key in device_keys {
            if let Some(device_list) = bt_data.get(key) {
                self.parse_devices_from_value(device_list, &mut devices, true);
            }
        }

        // Also check for not-connected but paired devices
        if let Some(paired_devices) = bt_data.get("device_not_connected") {
            self.parse_devices_from_value(paired_devices, &mut devices, false);
        }

        Ok(BluetoothInfo {
            enabled,
            discoverable,
            address,
            devices,
        })
    }

    fn parse_devices_from_value(&self, value: &Value, devices: &mut Vec<BluetoothDevice>, connected: bool) {
        match value {
            Value::Array(arr) => {
                for item in arr {
                    if let Some(device) = self.parse_device(item, connected) {
                        devices.push(device);
                    }
                }
            }
            Value::Object(obj) => {
                // Sometimes devices are keyed by their name
                for (name, data) in obj {
                    if let Some(mut device) = self.parse_device(data, connected) {
                        if device.name.is_empty() {
                            device.name = name.clone();
                        }
                        devices.push(device);
                    }
                }
            }
            _ => {}
        }
    }

    fn parse_device(&self, data: &Value, connected: bool) -> Option<BluetoothDevice> {
        let obj = data.as_object()?;

        let name = obj.get("device_name")
            .or_else(|| obj.get("_name"))
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown Device")
            .to_string();

        let address = obj.get("device_address")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let device_type = self.detect_device_type(obj);

        let battery_percent = obj.get("device_batteryLevelMain")
            .or_else(|| obj.get("device_batteryLevel"))
            .or_else(|| obj.get("device_batteryPercent"))
            .and_then(|v| {
                match v {
                    Value::Number(n) => n.as_u64().map(|n| n as u8),
                    Value::String(s) => s.trim_end_matches('%').parse::<u8>().ok(),
                    _ => None,
                }
            });

        let is_paired = obj.get("device_paired")
            .and_then(|v| v.as_str())
            .map(|s| s == "attrib_Yes" || s == "Yes")
            .unwrap_or(true);

        let vendor = obj.get("device_manufacturer")
            .or_else(|| obj.get("device_vendorID"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Some(BluetoothDevice {
            name,
            address,
            device_type,
            battery_percent,
            is_connected: connected,
            is_paired,
            vendor,
        })
    }

    fn detect_device_type(&self, obj: &serde_json::Map<String, Value>) -> String {
        // Check explicit type fields
        if let Some(minor_type) = obj.get("device_minorType")
            .or_else(|| obj.get("device_minorClassOfDevice"))
            .and_then(|v| v.as_str())
        {
            return self.normalize_device_type(minor_type);
        }

        // Check services for hints
        if let Some(services) = obj.get("device_services").and_then(|v| v.as_array()) {
            for service in services {
                if let Some(s) = service.as_str() {
                    let lower = s.to_lowercase();
                    if lower.contains("audio") || lower.contains("headset") || lower.contains("handsfree") {
                        return "Headphones".to_string();
                    }
                    if lower.contains("keyboard") {
                        return "Keyboard".to_string();
                    }
                    if lower.contains("mouse") || lower.contains("pointing") {
                        return "Mouse".to_string();
                    }
                }
            }
        }

        // Check name for hints
        if let Some(name) = obj.get("device_name").or_else(|| obj.get("_name")).and_then(|v| v.as_str()) {
            let lower = name.to_lowercase();
            if lower.contains("airpods") || lower.contains("headphone") || lower.contains("earbuds") || lower.contains("beats") {
                return "Headphones".to_string();
            }
            if lower.contains("keyboard") || lower.contains("magic keyboard") {
                return "Keyboard".to_string();
            }
            if lower.contains("mouse") || lower.contains("magic mouse") || lower.contains("trackpad") {
                return "Mouse".to_string();
            }
            if lower.contains("watch") {
                return "Watch".to_string();
            }
            if lower.contains("iphone") || lower.contains("ipad") {
                return "iOS Device".to_string();
            }
            if lower.contains("speaker") || lower.contains("homepod") {
                return "Speaker".to_string();
            }
        }

        "Other".to_string()
    }

    fn normalize_device_type(&self, device_type: &str) -> String {
        let lower = device_type.to_lowercase();

        if lower.contains("headphone") || lower.contains("headset") || lower.contains("audio") {
            "Headphones".to_string()
        } else if lower.contains("keyboard") {
            "Keyboard".to_string()
        } else if lower.contains("mouse") || lower.contains("pointing") {
            "Mouse".to_string()
        } else if lower.contains("gamepad") || lower.contains("joystick") {
            "Controller".to_string()
        } else if lower.contains("phone") {
            "Phone".to_string()
        } else if lower.contains("computer") {
            "Computer".to_string()
        } else {
            device_type.to_string()
        }
    }
}
