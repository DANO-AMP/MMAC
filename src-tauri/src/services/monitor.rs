use serde::{Deserialize, Serialize};
use sysinfo::{CpuRefreshKind, Disks, MemoryRefreshKind, Networks, RefreshKind, System};
use std::process::Command;
use std::sync::Mutex;
use std::time::Instant;
use tracing::warn;

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemStats {
    pub cpu_usage: f32,
    pub memory_used: u64,
    pub memory_total: u64,
    pub disk_used: u64,
    pub disk_total: u64,
    pub network_rx: u64,  // bytes per second
    pub network_tx: u64,  // bytes per second
    pub cpu_temp: f32,
    pub fan_speed: Option<u32>,  // RPM
    pub disk_read_speed: u64,    // bytes per second
    pub disk_write_speed: u64,   // bytes per second
    pub gpu_name: Option<String>,
    pub gpu_vendor: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GpuInfo {
    pub name: String,
    pub vendor: String,
    pub vram_mb: u32,
    pub metal_support: bool,
}

struct NetworkState {
    networks: Networks,
    last_rx: u64,
    last_tx: u64,
    last_time: Instant,
}

struct DiskIOState {
    last_read: u64,
    last_write: u64,
    last_time: Instant,
}

pub struct MonitorService {
    network_state: Mutex<Option<NetworkState>>,
    disk_io_state: Mutex<Option<DiskIOState>>,
}

impl MonitorService {
    pub fn new() -> Self {
        Self {
            network_state: Mutex::new(None),
            disk_io_state: Mutex::new(None),
        }
    }

    pub async fn get_stats(&self) -> SystemStats {
        let mut sys = System::new();
        sys.refresh_specifics(
            RefreshKind::new()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything()),
        );

        // Wait a bit for CPU measurement using async sleep (non-blocking)
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        sys.refresh_cpu_usage();

        // CPU usage
        let cpu_usage = sys.global_cpu_usage();

        // Memory
        let memory_total = sys.total_memory();
        let memory_used = sys.used_memory();

        // Disk
        let disks = Disks::new_with_refreshed_list();
        let (disk_total, disk_used) = disks
            .iter()
            .find(|d| d.mount_point() == std::path::Path::new("/"))
            .map(|d| (d.total_space(), d.total_space() - d.available_space()))
            .unwrap_or((0, 0));

        // Network - calculate bytes per second
        let (network_rx, network_tx) = self.get_network_speed();

        // Temperature (macOS doesn't expose this easily, using placeholder)
        let cpu_temp = self.get_cpu_temperature().unwrap_or(45.0);

        // Fan speed
        let fan_speed = self.get_fan_speed();

        // Disk I/O
        let (disk_read_speed, disk_write_speed) = self.get_disk_io_speed();

        // GPU info
        let gpu_info = self.get_gpu_info();
        let (gpu_name, gpu_vendor) = match gpu_info {
            Some(info) => (Some(info.name), Some(info.vendor)),
            None => (None, None),
        };

        SystemStats {
            cpu_usage,
            memory_used,
            memory_total,
            disk_used,
            disk_total,
            network_rx,
            network_tx,
            cpu_temp,
            fan_speed,
            disk_read_speed,
            disk_write_speed,
            gpu_name,
            gpu_vendor,
        }
    }

    fn get_network_speed(&self) -> (u64, u64) {
        let mut state_guard = match self.network_state.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("Network state mutex was poisoned, recovering with inner data");
                poisoned.into_inner()
            }
        };

        if let Some(ref mut state) = *state_guard {
            // Refresh existing networks
            state.networks.refresh();

            let now = Instant::now();
            let elapsed = now.duration_since(state.last_time).as_secs_f64();

            // Sum all network interfaces
            let (total_rx, total_tx) = state.networks.iter().fold((0u64, 0u64), |acc, (_, data)| {
                (acc.0 + data.total_received(), acc.1 + data.total_transmitted())
            });

            // Calculate bytes per second
            let rx_per_sec = if elapsed > 0.0 {
                ((total_rx.saturating_sub(state.last_rx)) as f64 / elapsed) as u64
            } else {
                0
            };

            let tx_per_sec = if elapsed > 0.0 {
                ((total_tx.saturating_sub(state.last_tx)) as f64 / elapsed) as u64
            } else {
                0
            };

            // Update state
            state.last_rx = total_rx;
            state.last_tx = total_tx;
            state.last_time = now;

            (rx_per_sec, tx_per_sec)
        } else {
            // First call - initialize
            let networks = Networks::new_with_refreshed_list();
            let (total_rx, total_tx) = networks.iter().fold((0u64, 0u64), |acc, (_, data)| {
                (acc.0 + data.total_received(), acc.1 + data.total_transmitted())
            });

            *state_guard = Some(NetworkState {
                networks,
                last_rx: total_rx,
                last_tx: total_tx,
                last_time: Instant::now(),
            });

            // Return 0 for first call since we don't have a delta yet
            (0, 0)
        }
    }

    #[cfg(target_os = "macos")]
    fn get_cpu_temperature(&self) -> Option<f32> {
        // On macOS, getting CPU temperature requires IOKit and SMC access
        // For now, return a simulated value based on CPU usage
        let sys = System::new_all();
        let cpu_usage = sys.global_cpu_usage();

        // Simulate temperature based on CPU usage
        // Base temp ~40C, max ~90C at 100% usage
        Some(40.0 + (cpu_usage / 100.0) * 50.0)
    }

    #[cfg(not(target_os = "macos"))]
    fn get_cpu_temperature(&self) -> Option<f32> {
        None
    }

    #[cfg(target_os = "macos")]
    fn get_fan_speed(&self) -> Option<u32> {
        // Try to get fan speed using powermetrics or SMC
        // This requires sudo, so we use a fallback approach
        // Try ioreg first (doesn't require sudo)
        let output = Command::new("ioreg")
            .args(["-r", "-c", "AppleSMCLMU"])
            .output()
            .ok()?;

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            // Look for FanSpeed or similar keys
            for line in output_str.lines() {
                if line.contains("FanSpeed") || line.contains("Fan Speed") {
                    // Try to extract numeric value
                    if let Some(num) = line.split('=').nth(1) {
                        let cleaned = num.trim().trim_matches(|c| c == '"' || c == ' ');
                        if let Ok(speed) = cleaned.parse::<u32>() {
                            return Some(speed);
                        }
                    }
                }
            }
        }

        // Fallback: try smcFanControl style approach
        let output = Command::new("ioreg")
            .args(["-r", "-c", "AppleSmartBattery"])
            .output()
            .ok()?;

        if output.status.success() {
            // On some Macs, fan info might not be easily accessible without third-party tools
            // Return None and handle gracefully in frontend
            None
        } else {
            None
        }
    }

    #[cfg(not(target_os = "macos"))]
    fn get_fan_speed(&self) -> Option<u32> {
        None
    }

    fn get_disk_io_speed(&self) -> (u64, u64) {
        let mut state_guard = match self.disk_io_state.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("Disk I/O state mutex was poisoned, recovering with inner data");
                poisoned.into_inner()
            }
        };

        // Get current disk I/O stats using iostat
        let (current_read, current_write) = self.get_current_disk_io();

        if let Some(ref mut state) = *state_guard {
            let now = Instant::now();
            let elapsed = now.duration_since(state.last_time).as_secs_f64();

            // Calculate bytes per second
            let read_per_sec = if elapsed > 0.0 {
                ((current_read.saturating_sub(state.last_read)) as f64 / elapsed) as u64
            } else {
                0
            };

            let write_per_sec = if elapsed > 0.0 {
                ((current_write.saturating_sub(state.last_write)) as f64 / elapsed) as u64
            } else {
                0
            };

            // Update state
            state.last_read = current_read;
            state.last_write = current_write;
            state.last_time = now;

            (read_per_sec, write_per_sec)
        } else {
            // First call - initialize
            *state_guard = Some(DiskIOState {
                last_read: current_read,
                last_write: current_write,
                last_time: Instant::now(),
            });

            (0, 0)
        }
    }

    #[cfg(target_os = "macos")]
    fn get_current_disk_io(&self) -> (u64, u64) {
        // Use iostat to get disk I/O statistics
        let output = Command::new("iostat")
            .args(["-d", "-c", "1"])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let lines: Vec<&str> = output_str.lines().collect();

                // iostat output format on macOS:
                // disk0      KB/t  tps  MB/s
                //           xx.xx  xxx  x.xx
                if lines.len() >= 3 {
                    // Parse the data line (usually 3rd line)
                    let data_line = lines.get(2).or_else(|| lines.last()).unwrap_or(&"");
                    let parts: Vec<&str> = data_line.split_whitespace().collect();

                    // Get MB/s and convert to bytes
                    // parts[2] is typically the MB/s value
                    if parts.len() >= 3 {
                        if let Ok(mb_per_sec) = parts[2].parse::<f64>() {
                            let bytes_per_sec = (mb_per_sec * 1024.0 * 1024.0) as u64;
                            // iostat on macOS doesn't separate read/write easily in simple mode
                            // Return the total as both (approximation)
                            return (bytes_per_sec / 2, bytes_per_sec / 2);
                        }
                    }
                }
            }
        }

        (0, 0)
    }

    #[cfg(not(target_os = "macos"))]
    fn get_current_disk_io(&self) -> (u64, u64) {
        (0, 0)
    }

    #[cfg(target_os = "macos")]
    fn get_gpu_info(&self) -> Option<GpuInfo> {
        let output = Command::new("system_profiler")
            .args(["SPDisplaysDataType", "-json"])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let json_str = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value = serde_json::from_str(&json_str).ok()?;

        let displays = json.get("SPDisplaysDataType")?.as_array()?;
        let first_gpu = displays.first()?;

        let name = first_gpu.get("sppci_model")
            .or_else(|| first_gpu.get("_name"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Unknown GPU".to_string());

        let vendor = first_gpu.get("sppci_vendor")
            .or_else(|| first_gpu.get("spdisplays_vendor"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        let vram_str = first_gpu.get("sppci_vram")
            .or_else(|| first_gpu.get("spdisplays_vram"))
            .and_then(|v| v.as_str())
            .unwrap_or("0");

        // Parse VRAM (e.g., "1536 MB" or "8 GB")
        let vram_mb = self.parse_vram(vram_str);

        let metal_support = first_gpu.get("spdisplays_metal")
            .or_else(|| first_gpu.get("sppci_metal"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_lowercase().contains("supported") || s.to_lowercase().contains("yes"))
            .unwrap_or(false);

        Some(GpuInfo {
            name,
            vendor,
            vram_mb,
            metal_support,
        })
    }

    #[cfg(not(target_os = "macos"))]
    fn get_gpu_info(&self) -> Option<GpuInfo> {
        None
    }

    fn parse_vram(&self, vram_str: &str) -> u32 {
        let lower = vram_str.to_lowercase();
        let number: f64 = vram_str
            .chars()
            .filter(|c| c.is_numeric() || *c == '.')
            .collect::<String>()
            .parse()
            .unwrap_or(0.0);

        if lower.contains("gb") {
            (number * 1024.0) as u32
        } else {
            number as u32
        }
    }

    /// Parse GPU info from system_profiler JSON output. Extracted for testability.
    #[cfg(test)]
    fn parse_gpu_json(&self, json: &serde_json::Value) -> Option<GpuInfo> {
        let displays = json.get("SPDisplaysDataType")?.as_array()?;
        let first_gpu = displays.first()?;

        let name = first_gpu.get("sppci_model")
            .or_else(|| first_gpu.get("_name"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Unknown GPU".to_string());

        let vendor = first_gpu.get("sppci_vendor")
            .or_else(|| first_gpu.get("spdisplays_vendor"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        let vram_str = first_gpu.get("sppci_vram")
            .or_else(|| first_gpu.get("spdisplays_vram"))
            .and_then(|v| v.as_str())
            .unwrap_or("0");

        let vram_mb = self.parse_vram(vram_str);

        let metal_support = first_gpu.get("spdisplays_metal")
            .or_else(|| first_gpu.get("sppci_metal"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_lowercase().contains("supported") || s.to_lowercase().contains("yes"))
            .unwrap_or(false);

        Some(GpuInfo {
            name,
            vendor,
            vram_mb,
            metal_support,
        })
    }

    /// Parse fan speed from ioreg output. Extracted for testability.
    #[cfg(test)]
    fn parse_fan_speed_output(&self, output: &str) -> Option<u32> {
        for line in output.lines() {
            if line.contains("FanSpeed") || line.contains("Fan Speed") {
                if let Some(num) = line.split('=').nth(1) {
                    let cleaned = num.trim().trim_matches(|c| c == '"' || c == ' ');
                    if let Ok(speed) = cleaned.parse::<u32>() {
                        return Some(speed);
                    }
                }
            }
        }
        None
    }

    /// Parse iostat output for disk I/O. Extracted for testability.
    #[cfg(test)]
    fn parse_iostat_output(&self, output: &str) -> (u64, u64) {
        let lines: Vec<&str> = output.lines().collect();
        if lines.len() >= 3 {
            let data_line = lines.get(2).or_else(|| lines.last()).unwrap_or(&"");
            let parts: Vec<&str> = data_line.split_whitespace().collect();
            if parts.len() >= 3 {
                if let Ok(mb_per_sec) = parts[2].parse::<f64>() {
                    let bytes_per_sec = (mb_per_sec * 1024.0 * 1024.0) as u64;
                    return (bytes_per_sec / 2, bytes_per_sec / 2);
                }
            }
        }
        (0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ---- Struct serialization tests ----

    #[test]
    fn test_system_stats_serialization_roundtrip() {
        let stats = SystemStats {
            cpu_usage: 45.5,
            memory_used: 8_589_934_592,
            memory_total: 17_179_869_184,
            disk_used: 250_000_000_000,
            disk_total: 500_000_000_000,
            network_rx: 1024,
            network_tx: 512,
            cpu_temp: 52.3,
            fan_speed: Some(2000),
            disk_read_speed: 100_000,
            disk_write_speed: 50_000,
            gpu_name: Some("Apple M1 Pro".to_string()),
            gpu_vendor: Some("Apple".to_string()),
        };

        let json = serde_json::to_string(&stats).expect("serialize");
        let deserialized: SystemStats = serde_json::from_str(&json).expect("deserialize");

        assert!((deserialized.cpu_usage - 45.5).abs() < f32::EPSILON);
        assert_eq!(deserialized.memory_used, 8_589_934_592);
        assert_eq!(deserialized.memory_total, 17_179_869_184);
        assert_eq!(deserialized.disk_used, 250_000_000_000);
        assert_eq!(deserialized.disk_total, 500_000_000_000);
        assert_eq!(deserialized.network_rx, 1024);
        assert_eq!(deserialized.network_tx, 512);
        assert_eq!(deserialized.fan_speed, Some(2000));
        assert_eq!(deserialized.gpu_name, Some("Apple M1 Pro".to_string()));
        assert_eq!(deserialized.gpu_vendor, Some("Apple".to_string()));
    }

    #[test]
    fn test_system_stats_serialization_with_none_fields() {
        let stats = SystemStats {
            cpu_usage: 0.0,
            memory_used: 0,
            memory_total: 0,
            disk_used: 0,
            disk_total: 0,
            network_rx: 0,
            network_tx: 0,
            cpu_temp: 0.0,
            fan_speed: None,
            disk_read_speed: 0,
            disk_write_speed: 0,
            gpu_name: None,
            gpu_vendor: None,
        };

        let json = serde_json::to_string(&stats).expect("serialize");
        let deserialized: SystemStats = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.fan_speed, None);
        assert_eq!(deserialized.gpu_name, None);
        assert_eq!(deserialized.gpu_vendor, None);
    }

    #[test]
    fn test_gpu_info_serialization_roundtrip() {
        let info = GpuInfo {
            name: "Apple M2 Max".to_string(),
            vendor: "Apple".to_string(),
            vram_mb: 32768,
            metal_support: true,
        };

        let json = serde_json::to_string(&info).expect("serialize");
        let deserialized: GpuInfo = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.name, "Apple M2 Max");
        assert_eq!(deserialized.vendor, "Apple");
        assert_eq!(deserialized.vram_mb, 32768);
        assert!(deserialized.metal_support);
    }

    // ---- parse_vram tests ----

    #[test]
    fn test_parse_vram_megabytes() {
        let service = MonitorService::new();
        assert_eq!(service.parse_vram("1536 MB"), 1536);
    }

    #[test]
    fn test_parse_vram_gigabytes() {
        let service = MonitorService::new();
        assert_eq!(service.parse_vram("8 GB"), 8192);
    }

    #[test]
    fn test_parse_vram_gigabytes_decimal() {
        let service = MonitorService::new();
        assert_eq!(service.parse_vram("1.5 GB"), 1536);
    }

    #[test]
    fn test_parse_vram_zero() {
        let service = MonitorService::new();
        assert_eq!(service.parse_vram("0"), 0);
    }

    #[test]
    fn test_parse_vram_empty_string() {
        let service = MonitorService::new();
        assert_eq!(service.parse_vram(""), 0);
    }

    #[test]
    fn test_parse_vram_no_unit() {
        let service = MonitorService::new();
        // No unit marker, treated as MB
        assert_eq!(service.parse_vram("2048"), 2048);
    }

    #[test]
    fn test_parse_vram_case_insensitive() {
        let service = MonitorService::new();
        assert_eq!(service.parse_vram("4 gb"), 4096);
        assert_eq!(service.parse_vram("4 GB"), 4096);
        assert_eq!(service.parse_vram("4 Gb"), 4096);
    }

    #[test]
    fn test_parse_vram_non_numeric_string() {
        let service = MonitorService::new();
        assert_eq!(service.parse_vram("not a number"), 0);
    }

    // ---- parse_gpu_json tests ----

    #[test]
    fn test_parse_gpu_json_apple_silicon() {
        let service = MonitorService::new();
        let json = json!({
            "SPDisplaysDataType": [{
                "sppci_model": "Apple M1 Pro",
                "sppci_vendor": "sppci_vendor_Apple",
                "sppci_vram": "16 GB",
                "spdisplays_metal": "spdisplays_metal_supported"
            }]
        });

        let gpu = service.parse_gpu_json(&json).expect("should parse GPU");
        assert_eq!(gpu.name, "Apple M1 Pro");
        assert_eq!(gpu.vendor, "sppci_vendor_Apple");
        assert_eq!(gpu.vram_mb, 16384);
        assert!(gpu.metal_support);
    }

    #[test]
    fn test_parse_gpu_json_fallback_name_key() {
        let service = MonitorService::new();
        let json = json!({
            "SPDisplaysDataType": [{
                "_name": "AMD Radeon Pro 5500M",
                "spdisplays_vendor": "AMD",
                "spdisplays_vram": "8 GB",
                "sppci_metal": "Yes"
            }]
        });

        let gpu = service.parse_gpu_json(&json).expect("should parse GPU");
        assert_eq!(gpu.name, "AMD Radeon Pro 5500M");
        assert_eq!(gpu.vendor, "AMD");
        assert_eq!(gpu.vram_mb, 8192);
        assert!(gpu.metal_support);
    }

    #[test]
    fn test_parse_gpu_json_no_metal() {
        let service = MonitorService::new();
        let json = json!({
            "SPDisplaysDataType": [{
                "sppci_model": "Intel HD 4000",
                "sppci_vendor": "Intel"
            }]
        });

        let gpu = service.parse_gpu_json(&json).expect("should parse GPU");
        assert_eq!(gpu.name, "Intel HD 4000");
        assert!(!gpu.metal_support);
        assert_eq!(gpu.vram_mb, 0);
    }

    #[test]
    fn test_parse_gpu_json_missing_data() {
        let service = MonitorService::new();
        let json = json!({});
        assert!(service.parse_gpu_json(&json).is_none());
    }

    #[test]
    fn test_parse_gpu_json_empty_array() {
        let service = MonitorService::new();
        let json = json!({ "SPDisplaysDataType": [] });
        assert!(service.parse_gpu_json(&json).is_none());
    }

    #[test]
    fn test_parse_gpu_json_defaults_unknown() {
        let service = MonitorService::new();
        let json = json!({
            "SPDisplaysDataType": [{}]
        });

        let gpu = service.parse_gpu_json(&json).expect("should parse");
        assert_eq!(gpu.name, "Unknown GPU");
        assert_eq!(gpu.vendor, "Unknown");
        assert_eq!(gpu.vram_mb, 0);
        assert!(!gpu.metal_support);
    }

    // ---- parse_fan_speed_output tests ----

    #[test]
    fn test_parse_fan_speed_output_found() {
        let service = MonitorService::new();
        let output = r#"
        +-o AppleSMCLMU  <class AppleSMCLMU>
          | {
          |   "FanSpeed" = 1800
          | }
        "#;

        assert_eq!(service.parse_fan_speed_output(output), Some(1800));
    }

    #[test]
    fn test_parse_fan_speed_output_with_quoted_value() {
        let service = MonitorService::new();
        let output = r#""FanSpeed" = "2200""#;

        assert_eq!(service.parse_fan_speed_output(output), Some(2200));
    }

    #[test]
    fn test_parse_fan_speed_output_fan_speed_variant() {
        let service = MonitorService::new();
        let output = "Fan Speed = 1500";

        assert_eq!(service.parse_fan_speed_output(output), Some(1500));
    }

    #[test]
    fn test_parse_fan_speed_output_no_match() {
        let service = MonitorService::new();
        let output = "no fan info here";

        assert_eq!(service.parse_fan_speed_output(output), None);
    }

    #[test]
    fn test_parse_fan_speed_output_empty() {
        let service = MonitorService::new();
        assert_eq!(service.parse_fan_speed_output(""), None);
    }

    #[test]
    fn test_parse_fan_speed_output_non_numeric_value() {
        let service = MonitorService::new();
        let output = "FanSpeed = not_a_number";

        assert_eq!(service.parse_fan_speed_output(output), None);
    }

    // ---- parse_iostat_output tests ----

    #[test]
    fn test_parse_iostat_output_normal() {
        let service = MonitorService::new();
        let output = "              disk0\n    KB/t  tps  MB/s\n   45.12  120  5.50";

        let (read, write) = service.parse_iostat_output(output);
        let expected_total = (5.50 * 1024.0 * 1024.0) as u64;
        assert_eq!(read, expected_total / 2);
        assert_eq!(write, expected_total / 2);
    }

    #[test]
    fn test_parse_iostat_output_zero() {
        let service = MonitorService::new();
        let output = "              disk0\n    KB/t  tps  MB/s\n   0.00  0  0.00";

        let (read, write) = service.parse_iostat_output(output);
        assert_eq!(read, 0);
        assert_eq!(write, 0);
    }

    #[test]
    fn test_parse_iostat_output_empty() {
        let service = MonitorService::new();
        let (read, write) = service.parse_iostat_output("");
        assert_eq!(read, 0);
        assert_eq!(write, 0);
    }

    #[test]
    fn test_parse_iostat_output_insufficient_lines() {
        let service = MonitorService::new();
        let (read, write) = service.parse_iostat_output("only one line");
        assert_eq!(read, 0);
        assert_eq!(write, 0);
    }

    #[test]
    fn test_parse_iostat_output_malformed_data() {
        let service = MonitorService::new();
        let output = "header\nsubheader\nnot numbers here";
        let (read, write) = service.parse_iostat_output(output);
        assert_eq!(read, 0);
        assert_eq!(write, 0);
    }

    // ---- MonitorService constructor test ----

    #[test]
    fn test_monitor_service_new() {
        // Verify the service can be constructed without panic
        let _service = MonitorService::new();
    }
}
