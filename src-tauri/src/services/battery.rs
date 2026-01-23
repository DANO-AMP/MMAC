use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct BatteryInfo {
    pub is_present: bool,
    pub is_charging: bool,
    pub is_fully_charged: bool,
    pub charge_percent: f32,
    pub cycle_count: u32,
    pub max_capacity: u32,
    pub design_capacity: u32,
    pub health_percent: f32,
    pub temperature: f32,
    pub voltage: f32,
    pub amperage: i32,
    pub time_remaining: Option<String>,
    pub power_source: String,
    pub condition: String,
}

pub struct BatteryService;

impl BatteryService {
    pub fn new() -> Self {
        Self
    }

    pub fn get_battery_info(&self) -> Option<BatteryInfo> {
        // Use ioreg to get battery info
        let output = Command::new("ioreg")
            .args(["-r", "-c", "AppleSmartBattery", "-d", "1"])
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Check if battery is installed
        if !stdout.contains("BatteryInstalled") || !stdout.contains("\"BatteryInstalled\" = Yes") {
            return None;
        }

        let is_charging = self.extract_bool_value(&stdout, "IsCharging");
        let is_fully_charged = self.extract_bool_value(&stdout, "FullyCharged");
        let external_connected = self.extract_bool_value(&stdout, "ExternalConnected");

        // CurrentCapacity is percentage (0-100)
        let current_capacity = self.extract_int_value(&stdout, "\"CurrentCapacity\"").unwrap_or(0);

        // For actual mAh values
        let raw_max_capacity = self.extract_int_value(&stdout, "\"AppleRawMaxCapacity\"").unwrap_or(0);
        let design_capacity = self.extract_int_value(&stdout, "\"DesignCapacity\"").unwrap_or(0);

        let cycle_count = self.extract_int_value(&stdout, "\"CycleCount\"").unwrap_or(0);

        // Temperature is in deciKelvin, convert to Celsius
        let temp_raw = self.extract_int_value(&stdout, "\"Temperature\"").unwrap_or(2932);
        let temperature = (temp_raw as f32 / 10.0) - 273.15;

        // Voltage in mV, convert to V
        let voltage_mv = self.extract_int_value(&stdout, "\"Voltage\"").unwrap_or(0);
        let voltage = voltage_mv as f32 / 1000.0;

        // Amperage in mA
        let amperage = self.extract_int_value(&stdout, "\"Amperage\"").unwrap_or(0) as i32;

        // Time remaining in minutes
        let time_remaining_mins = self.extract_int_value(&stdout, "\"TimeRemaining\"");

        let charge_percent = current_capacity as f32;

        let health_percent = if design_capacity > 0 {
            (raw_max_capacity as f32 / design_capacity as f32) * 100.0
        } else {
            100.0
        };

        // Format time remaining
        let time_remaining = time_remaining_mins.map(|mins| {
            if mins == 65535 {
                // Calculating...
                "Calculando...".to_string()
            } else {
                let hours = mins / 60;
                let minutes = mins % 60;
                if is_charging {
                    format!("{}:{:02} hasta carga completa", hours, minutes)
                } else {
                    format!("{}:{:02} restante", hours, minutes)
                }
            }
        });

        // Determine power source
        let power_source = if external_connected {
            if is_charging {
                "Cargando".to_string()
            } else {
                "Conectado (no cargando)".to_string()
            }
        } else {
            "Batería".to_string()
        };

        // Determine condition based on health
        let condition = if health_percent >= 80.0 {
            "Normal".to_string()
        } else if health_percent >= 60.0 {
            "Servicio recomendado".to_string()
        } else {
            "Reemplazar pronto".to_string()
        };

        Some(BatteryInfo {
            is_present: true,
            is_charging,
            is_fully_charged,
            charge_percent,
            cycle_count: cycle_count as u32,
            max_capacity: raw_max_capacity as u32,
            design_capacity: design_capacity as u32,
            health_percent,
            temperature,
            voltage,
            amperage,
            time_remaining,
            power_source,
            condition,
        })
    }

    fn extract_bool_value(&self, text: &str, key: &str) -> bool {
        // Look for pattern: "Key" = Yes or "Key" = No
        for line in text.lines() {
            if line.contains(&format!("\"{}\"", key)) {
                return line.contains("= Yes");
            }
        }
        false
    }

    fn extract_int_value(&self, text: &str, key: &str) -> Option<i64> {
        // Look for pattern: "Key" = 12345
        for line in text.lines() {
            if line.contains(key) && line.contains("=") {
                // Split by = and get the value part
                if let Some(value_part) = line.split('=').nth(1) {
                    let trimmed = value_part.trim();
                    // Parse the number (might have trailing content)
                    if let Ok(num) = trimmed.parse::<i64>() {
                        return Some(num);
                    }
                }
            }
        }
        None
    }
}
