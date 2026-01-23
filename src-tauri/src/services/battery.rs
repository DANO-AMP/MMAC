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
            .args(["-r", "-c", "AppleSmartBattery", "-a"])
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Parse plist-like output
        let is_present = stdout.contains("BatteryInstalled");
        if !is_present {
            return None;
        }

        let is_charging = self.extract_bool(&stdout, "IsCharging").unwrap_or(false);
        let is_fully_charged = self.extract_bool(&stdout, "FullyCharged").unwrap_or(false);
        let current_capacity = self.extract_number(&stdout, "CurrentCapacity").unwrap_or(0);
        let max_capacity = self.extract_number(&stdout, "MaxCapacity").unwrap_or(100);
        let design_capacity = self.extract_number(&stdout, "DesignCapacity").unwrap_or(100);
        let cycle_count = self.extract_number(&stdout, "CycleCount").unwrap_or(0);
        let temperature = self.extract_number(&stdout, "Temperature").unwrap_or(0) as f32 / 100.0;
        let voltage = self.extract_number(&stdout, "Voltage").unwrap_or(0) as f32 / 1000.0;
        let amperage = self.extract_number(&stdout, "Amperage").unwrap_or(0) as i32;

        let charge_percent = if max_capacity > 0 {
            (current_capacity as f32 / max_capacity as f32) * 100.0
        } else {
            0.0
        };

        let health_percent = if design_capacity > 0 {
            (max_capacity as f32 / design_capacity as f32) * 100.0
        } else {
            100.0
        };

        // Get time remaining from pmset
        let time_remaining = self.get_time_remaining();

        // Get power source
        let power_source = if is_charging {
            "Conectado a corriente".to_string()
        } else {
            "Batería".to_string()
        };

        // Determine condition
        let condition = if health_percent >= 80.0 {
            "Normal".to_string()
        } else if health_percent >= 60.0 {
            "Servicio recomendado".to_string()
        } else {
            "Reemplazar pronto".to_string()
        };

        Some(BatteryInfo {
            is_present,
            is_charging,
            is_fully_charged,
            charge_percent,
            cycle_count: cycle_count as u32,
            max_capacity: max_capacity as u32,
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

    fn extract_bool(&self, text: &str, key: &str) -> Option<bool> {
        for line in text.lines() {
            if line.contains(key) {
                return Some(line.contains("Yes") || line.contains("true"));
            }
        }
        None
    }

    fn extract_number(&self, text: &str, key: &str) -> Option<i64> {
        for line in text.lines() {
            if line.contains(key) {
                // Find the number in the line
                for word in line.split_whitespace() {
                    if let Ok(num) = word.trim_matches(|c: char| !c.is_ascii_digit() && c != '-').parse() {
                        return Some(num);
                    }
                }
            }
        }
        None
    }

    fn get_time_remaining(&self) -> Option<String> {
        let output = Command::new("pmset")
            .args(["-g", "batt"])
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Look for time remaining pattern like "2:30 remaining" or "0:45 remaining"
        for line in stdout.lines() {
            if line.contains("remaining") {
                if let Some(time_part) = line.split_whitespace().find(|s| s.contains(':')) {
                    return Some(format!("{} restante", time_part));
                }
            } else if line.contains("charging") {
                if let Some(time_part) = line.split_whitespace().find(|s| s.contains(':')) {
                    return Some(format!("{} hasta carga completa", time_part));
                }
            }
        }

        None
    }
}
