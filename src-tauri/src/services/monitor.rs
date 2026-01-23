use serde::{Deserialize, Serialize};
use sysinfo::{CpuRefreshKind, Disks, MemoryRefreshKind, Networks, RefreshKind, System};
use std::sync::Mutex;
use std::time::Instant;

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
}

struct NetworkState {
    networks: Networks,
    last_rx: u64,
    last_tx: u64,
    last_time: Instant,
}

pub struct MonitorService {
    network_state: Mutex<Option<NetworkState>>,
}

impl MonitorService {
    pub fn new() -> Self {
        Self {
            network_state: Mutex::new(None),
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

    fn get_network_speed(&self) -> (u64, u64) {
        let mut state_guard = self.network_state.lock().unwrap();

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
}
