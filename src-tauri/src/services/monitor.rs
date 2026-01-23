use serde::{Deserialize, Serialize};
use sysinfo::{CpuRefreshKind, Disks, MemoryRefreshKind, Networks, RefreshKind, System};

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemStats {
    pub cpu_usage: f32,
    pub memory_used: u64,
    pub memory_total: u64,
    pub disk_used: u64,
    pub disk_total: u64,
    pub network_rx: u64,
    pub network_tx: u64,
    pub cpu_temp: f32,
}

pub struct MonitorService;

impl MonitorService {
    pub fn new() -> Self {
        Self
    }

    pub fn get_stats(&self) -> SystemStats {
        let mut sys = System::new();
        sys.refresh_specifics(
            RefreshKind::new()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything()),
        );

        // Wait a bit for CPU measurement
        std::thread::sleep(std::time::Duration::from_millis(200));
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

        // Network
        let networks = Networks::new_with_refreshed_list();
        let (network_rx, network_tx) = networks.iter().fold((0u64, 0u64), |acc, (_, data)| {
            (acc.0 + data.received(), acc.1 + data.transmitted())
        });

        // Temperature (macOS doesn't expose this easily, using placeholder)
        let cpu_temp = self.get_cpu_temperature().unwrap_or(45.0);

        SystemStats {
            cpu_usage,
            memory_used,
            memory_total,
            disk_used,
            disk_total,
            network_rx,
            network_tx,
            cpu_temp,
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
}
