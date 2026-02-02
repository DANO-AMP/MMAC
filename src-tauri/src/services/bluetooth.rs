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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ---- Struct serialization / deserialization tests ----

    #[test]
    fn test_bluetooth_device_serialization_roundtrip() {
        let device = BluetoothDevice {
            name: "AirPods Pro".to_string(),
            address: "AA:BB:CC:DD:EE:FF".to_string(),
            device_type: "Headphones".to_string(),
            battery_percent: Some(85),
            is_connected: true,
            is_paired: true,
            vendor: Some("Apple".to_string()),
        };

        let json = serde_json::to_string(&device).expect("serialize");
        let deserialized: BluetoothDevice = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.name, "AirPods Pro");
        assert_eq!(deserialized.address, "AA:BB:CC:DD:EE:FF");
        assert_eq!(deserialized.device_type, "Headphones");
        assert_eq!(deserialized.battery_percent, Some(85));
        assert!(deserialized.is_connected);
        assert!(deserialized.is_paired);
        assert_eq!(deserialized.vendor, Some("Apple".to_string()));
    }

    #[test]
    fn test_bluetooth_device_serialization_with_none_fields() {
        let device = BluetoothDevice {
            name: "Unknown".to_string(),
            address: "".to_string(),
            device_type: "Other".to_string(),
            battery_percent: None,
            is_connected: false,
            is_paired: false,
            vendor: None,
        };

        let json = serde_json::to_string(&device).expect("serialize");
        let deserialized: BluetoothDevice = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.battery_percent, None);
        assert_eq!(deserialized.vendor, None);
        assert!(!deserialized.is_connected);
    }

    #[test]
    fn test_bluetooth_info_serialization_roundtrip() {
        let info = BluetoothInfo {
            enabled: true,
            discoverable: false,
            address: Some("11:22:33:44:55:66".to_string()),
            devices: vec![],
        };

        let json = serde_json::to_string(&info).expect("serialize");
        let deserialized: BluetoothInfo = serde_json::from_str(&json).expect("deserialize");

        assert!(deserialized.enabled);
        assert!(!deserialized.discoverable);
        assert_eq!(deserialized.address, Some("11:22:33:44:55:66".to_string()));
        assert!(deserialized.devices.is_empty());
    }

    // ---- parse_bluetooth_data tests ----

    fn sample_bt_json() -> Value {
        json!({
            "SPBluetoothDataType": [{
                "controller_properties": {
                    "controller_state": "attrib_on",
                    "controller_discoverable": "attrib_on",
                    "controller_address": "AA:BB:CC:DD:EE:FF"
                },
                "device_connected": [
                    {
                        "device_name": "AirPods Pro",
                        "device_address": "11:22:33:44:55:66",
                        "device_batteryLevelMain": 85,
                        "device_paired": "attrib_Yes",
                        "device_manufacturer": "Apple"
                    }
                ],
                "device_not_connected": [
                    {
                        "device_name": "Magic Mouse",
                        "device_address": "77:88:99:AA:BB:CC",
                        "device_paired": "attrib_Yes",
                        "device_manufacturer": "Apple"
                    }
                ]
            }]
        })
    }

    #[test]
    fn test_parse_bluetooth_data_happy_path() {
        let service = BluetoothService::new();
        let json = sample_bt_json();

        let result = service.parse_bluetooth_data(&json).expect("should parse");

        assert!(result.enabled);
        assert!(result.discoverable);
        assert_eq!(result.address, Some("AA:BB:CC:DD:EE:FF".to_string()));
        assert_eq!(result.devices.len(), 2);

        let connected = result.devices.iter().find(|d| d.is_connected).expect("connected device");
        assert_eq!(connected.name, "AirPods Pro");
        assert_eq!(connected.battery_percent, Some(85));
        assert!(connected.is_paired);

        let disconnected = result.devices.iter().find(|d| !d.is_connected).expect("disconnected device");
        assert_eq!(disconnected.name, "Magic Mouse");
        assert!(!disconnected.is_connected);
    }

    #[test]
    fn test_parse_bluetooth_data_disabled_controller() {
        let service = BluetoothService::new();
        let json = json!({
            "SPBluetoothDataType": [{
                "controller_properties": {
                    "controller_state": "attrib_off",
                    "controller_discoverable": "attrib_off"
                }
            }]
        });

        let result = service.parse_bluetooth_data(&json).expect("should parse");

        assert!(!result.enabled);
        assert!(!result.discoverable);
        assert_eq!(result.address, None);
        assert!(result.devices.is_empty());
    }

    #[test]
    fn test_parse_bluetooth_data_no_controller_properties() {
        let service = BluetoothService::new();
        let json = json!({
            "SPBluetoothDataType": [{}]
        });

        let result = service.parse_bluetooth_data(&json).expect("should parse");

        assert!(!result.enabled);
        assert!(!result.discoverable);
        assert_eq!(result.address, None);
    }

    #[test]
    fn test_parse_bluetooth_data_missing_data() {
        let service = BluetoothService::new();
        let json = json!({});

        let result = service.parse_bluetooth_data(&json);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_bluetooth_data_empty_array() {
        let service = BluetoothService::new();
        let json = json!({ "SPBluetoothDataType": [] });

        let result = service.parse_bluetooth_data(&json);
        assert!(result.is_err());
    }

    // ---- parse_devices_from_value tests ----

    #[test]
    fn test_parse_devices_from_array() {
        let service = BluetoothService::new();
        let value = json!([
            {
                "device_name": "Device A",
                "device_address": "AA:AA:AA:AA:AA:AA"
            },
            {
                "device_name": "Device B",
                "device_address": "BB:BB:BB:BB:BB:BB"
            }
        ]);

        let mut devices = Vec::new();
        service.parse_devices_from_value(&value, &mut devices, true);

        assert_eq!(devices.len(), 2);
        assert!(devices[0].is_connected);
        assert!(devices[1].is_connected);
    }

    #[test]
    fn test_parse_devices_from_object_keyed_by_name_with_device_name() {
        let service = BluetoothService::new();
        let value = json!({
            "My Keyboard": {
                "device_name": "Actual Name",
                "device_address": "CC:CC:CC:CC:CC:CC",
                "device_minorType": "Keyboard"
            }
        });

        let mut devices = Vec::new();
        service.parse_devices_from_value(&value, &mut devices, false);

        assert_eq!(devices.len(), 1);
        // When device_name IS present inside the object, it is used (not the key)
        assert_eq!(devices[0].name, "Actual Name");
        assert!(!devices[0].is_connected);
    }

    #[test]
    fn test_parse_devices_from_object_key_not_used_when_default_name() {
        let service = BluetoothService::new();
        // When no device_name or _name is present, parse_device returns "Unknown Device".
        // The key-name override only fires when device.name.is_empty(), which does NOT happen
        // with the default "Unknown Device" string. This tests the actual behavior.
        let value = json!({
            "My Keyboard": {
                "device_address": "CC:CC:CC:CC:CC:CC",
                "device_minorType": "Keyboard"
            }
        });

        let mut devices = Vec::new();
        service.parse_devices_from_value(&value, &mut devices, false);

        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].name, "Unknown Device");
    }

    #[test]
    fn test_parse_devices_from_non_array_non_object() {
        let service = BluetoothService::new();
        let value = json!("just a string");

        let mut devices = Vec::new();
        service.parse_devices_from_value(&value, &mut devices, true);

        assert!(devices.is_empty());
    }

    // ---- parse_device tests ----

    #[test]
    fn test_parse_device_battery_as_number() {
        let service = BluetoothService::new();
        let data = json!({
            "device_name": "Test",
            "device_address": "AA:BB:CC:DD:EE:FF",
            "device_batteryLevelMain": 72
        });

        let device = service.parse_device(&data, true).expect("should parse");
        assert_eq!(device.battery_percent, Some(72));
    }

    #[test]
    fn test_parse_device_battery_as_string_with_percent() {
        let service = BluetoothService::new();
        let data = json!({
            "device_name": "Test",
            "device_address": "AA:BB:CC:DD:EE:FF",
            "device_batteryPercent": "90%"
        });

        let device = service.parse_device(&data, true).expect("should parse");
        assert_eq!(device.battery_percent, Some(90));
    }

    #[test]
    fn test_parse_device_fallback_name_key() {
        let service = BluetoothService::new();
        let data = json!({
            "_name": "FallbackName",
            "device_address": "AA:BB:CC:DD:EE:FF"
        });

        let device = service.parse_device(&data, true).expect("should parse");
        assert_eq!(device.name, "FallbackName");
    }

    #[test]
    fn test_parse_device_unknown_defaults() {
        let service = BluetoothService::new();
        let data = json!({});

        let device = service.parse_device(&data, false).expect("should parse");
        assert_eq!(device.name, "Unknown Device");
        assert_eq!(device.address, "");
        assert_eq!(device.battery_percent, None);
        assert!(!device.is_connected);
        // No device_paired field means default true
        assert!(device.is_paired);
    }

    #[test]
    fn test_parse_device_not_an_object() {
        let service = BluetoothService::new();
        let data = json!("not an object");

        let device = service.parse_device(&data, true);
        assert!(device.is_none());
    }

    #[test]
    fn test_parse_device_paired_yes() {
        let service = BluetoothService::new();
        let data = json!({
            "device_name": "D",
            "device_paired": "Yes"
        });

        let device = service.parse_device(&data, true).unwrap();
        assert!(device.is_paired);
    }

    #[test]
    fn test_parse_device_paired_no() {
        let service = BluetoothService::new();
        let data = json!({
            "device_name": "D",
            "device_paired": "attrib_No"
        });

        let device = service.parse_device(&data, true).unwrap();
        assert!(!device.is_paired);
    }

    // ---- detect_device_type tests ----

    #[test]
    fn test_detect_device_type_from_minor_type_field() {
        let service = BluetoothService::new();
        let mut obj = serde_json::Map::new();
        obj.insert("device_minorType".to_string(), json!("Headset"));

        assert_eq!(service.detect_device_type(&obj), "Headphones");
    }

    #[test]
    fn test_detect_device_type_from_minor_class() {
        let service = BluetoothService::new();
        let mut obj = serde_json::Map::new();
        obj.insert("device_minorClassOfDevice".to_string(), json!("Keyboard"));

        assert_eq!(service.detect_device_type(&obj), "Keyboard");
    }

    #[test]
    fn test_detect_device_type_from_services_audio() {
        let service = BluetoothService::new();
        let mut obj = serde_json::Map::new();
        obj.insert("device_services".to_string(), json!(["Audio Sink", "AVRCP"]));

        assert_eq!(service.detect_device_type(&obj), "Headphones");
    }

    #[test]
    fn test_detect_device_type_from_services_keyboard() {
        let service = BluetoothService::new();
        let mut obj = serde_json::Map::new();
        obj.insert("device_services".to_string(), json!(["Keyboard Input"]));

        assert_eq!(service.detect_device_type(&obj), "Keyboard");
    }

    #[test]
    fn test_detect_device_type_from_services_mouse() {
        let service = BluetoothService::new();
        let mut obj = serde_json::Map::new();
        obj.insert("device_services".to_string(), json!(["Mouse Input", "Pointing Device"]));

        assert_eq!(service.detect_device_type(&obj), "Mouse");
    }

    #[test]
    fn test_detect_device_type_from_name_airpods() {
        let service = BluetoothService::new();
        let mut obj = serde_json::Map::new();
        obj.insert("device_name".to_string(), json!("John's AirPods Pro"));

        assert_eq!(service.detect_device_type(&obj), "Headphones");
    }

    #[test]
    fn test_detect_device_type_from_name_beats() {
        let service = BluetoothService::new();
        let mut obj = serde_json::Map::new();
        obj.insert("device_name".to_string(), json!("Beats Solo3"));

        assert_eq!(service.detect_device_type(&obj), "Headphones");
    }

    #[test]
    fn test_detect_device_type_from_name_magic_keyboard() {
        let service = BluetoothService::new();
        let mut obj = serde_json::Map::new();
        obj.insert("device_name".to_string(), json!("Magic Keyboard"));

        assert_eq!(service.detect_device_type(&obj), "Keyboard");
    }

    #[test]
    fn test_detect_device_type_from_name_magic_mouse() {
        let service = BluetoothService::new();
        let mut obj = serde_json::Map::new();
        obj.insert("device_name".to_string(), json!("Magic Mouse 2"));

        assert_eq!(service.detect_device_type(&obj), "Mouse");
    }

    #[test]
    fn test_detect_device_type_from_name_trackpad() {
        let service = BluetoothService::new();
        let mut obj = serde_json::Map::new();
        obj.insert("device_name".to_string(), json!("Magic Trackpad"));

        assert_eq!(service.detect_device_type(&obj), "Mouse");
    }

    #[test]
    fn test_detect_device_type_from_name_watch() {
        let service = BluetoothService::new();
        let mut obj = serde_json::Map::new();
        obj.insert("device_name".to_string(), json!("Apple Watch"));

        assert_eq!(service.detect_device_type(&obj), "Watch");
    }

    #[test]
    fn test_detect_device_type_from_name_iphone() {
        let service = BluetoothService::new();
        let mut obj = serde_json::Map::new();
        obj.insert("device_name".to_string(), json!("John's iPhone"));

        assert_eq!(service.detect_device_type(&obj), "iOS Device");
    }

    #[test]
    fn test_detect_device_type_from_name_ipad() {
        let service = BluetoothService::new();
        let mut obj = serde_json::Map::new();
        obj.insert("device_name".to_string(), json!("iPad Pro"));

        assert_eq!(service.detect_device_type(&obj), "iOS Device");
    }

    #[test]
    fn test_detect_device_type_from_name_speaker() {
        let service = BluetoothService::new();
        let mut obj = serde_json::Map::new();
        obj.insert("device_name".to_string(), json!("HomePod mini"));

        assert_eq!(service.detect_device_type(&obj), "Speaker");
    }

    #[test]
    fn test_detect_device_type_unknown() {
        let service = BluetoothService::new();
        let obj = serde_json::Map::new();

        assert_eq!(service.detect_device_type(&obj), "Other");
    }

    // ---- normalize_device_type tests ----

    #[test]
    fn test_normalize_headphones() {
        let service = BluetoothService::new();
        assert_eq!(service.normalize_device_type("Headphone"), "Headphones");
        assert_eq!(service.normalize_device_type("headset"), "Headphones");
        assert_eq!(service.normalize_device_type("Audio Device"), "Headphones");
    }

    #[test]
    fn test_normalize_keyboard() {
        let service = BluetoothService::new();
        assert_eq!(service.normalize_device_type("Keyboard"), "Keyboard");
    }

    #[test]
    fn test_normalize_mouse() {
        let service = BluetoothService::new();
        assert_eq!(service.normalize_device_type("Mouse"), "Mouse");
        assert_eq!(service.normalize_device_type("Pointing Device"), "Mouse");
    }

    #[test]
    fn test_normalize_controller() {
        let service = BluetoothService::new();
        assert_eq!(service.normalize_device_type("Gamepad"), "Controller");
        assert_eq!(service.normalize_device_type("Joystick"), "Controller");
    }

    #[test]
    fn test_normalize_phone() {
        let service = BluetoothService::new();
        assert_eq!(service.normalize_device_type("Phone"), "Phone");
    }

    #[test]
    fn test_normalize_computer() {
        let service = BluetoothService::new();
        assert_eq!(service.normalize_device_type("Computer"), "Computer");
    }

    #[test]
    fn test_normalize_unknown_passthrough() {
        let service = BluetoothService::new();
        assert_eq!(service.normalize_device_type("SomethingElse"), "SomethingElse");
    }

    // ---- Multiple device keys test ----

    #[test]
    fn test_parse_devices_from_devices_list_key() {
        let service = BluetoothService::new();
        let json = json!({
            "SPBluetoothDataType": [{
                "controller_properties": {
                    "controller_state": "attrib_on"
                },
                "devices_list": [
                    {
                        "device_name": "Listed Device",
                        "device_address": "AA:AA:AA:AA:AA:AA"
                    }
                ]
            }]
        });

        let result = service.parse_bluetooth_data(&json).expect("should parse");
        assert_eq!(result.devices.len(), 1);
        assert_eq!(result.devices[0].name, "Listed Device");
        assert!(result.devices[0].is_connected);
    }

    #[test]
    fn test_parse_devices_from_device_title_key() {
        let service = BluetoothService::new();
        let json = json!({
            "SPBluetoothDataType": [{
                "controller_properties": {
                    "controller_state": "attrib_on"
                },
                "device_title": {
                    "My Device": {
                        "device_name": "My Device",
                        "device_address": "BB:BB:BB:BB:BB:BB"
                    }
                }
            }]
        });

        let result = service.parse_bluetooth_data(&json).expect("should parse");
        assert_eq!(result.devices.len(), 1);
        assert_eq!(result.devices[0].name, "My Device");
    }
}
