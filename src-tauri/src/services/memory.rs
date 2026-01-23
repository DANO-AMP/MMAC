use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub active: u64,
    pub inactive: u64,
    pub wired: u64,
    pub compressed: u64,
    pub app_memory: u64,
    pub cached: u64,
}

pub struct MemoryService;

impl MemoryService {
    pub fn new() -> Self {
        Self
    }

    pub fn get_memory_info(&self) -> MemoryInfo {
        let sys = sysinfo::System::new_all();

        let total = sys.total_memory();
        let used = sys.used_memory();
        let free = sys.free_memory();

        // Get detailed memory stats from vm_stat
        let (active, inactive, wired, compressed, cached) = self.get_vm_stat();

        MemoryInfo {
            total,
            used,
            free,
            active,
            inactive,
            wired,
            compressed,
            app_memory: active + wired,
            cached,
        }
    }

    fn get_vm_stat(&self) -> (u64, u64, u64, u64, u64) {
        let output = Command::new("vm_stat").output();

        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let page_size = 16384u64; // Default page size on Apple Silicon

            let mut active = 0u64;
            let mut inactive = 0u64;
            let mut wired = 0u64;
            let mut compressed = 0u64;
            let mut cached = 0u64;

            for line in stdout.lines() {
                if line.contains("Pages active:") {
                    active = self.parse_vm_stat_line(line) * page_size;
                } else if line.contains("Pages inactive:") {
                    inactive = self.parse_vm_stat_line(line) * page_size;
                } else if line.contains("Pages wired down:") {
                    wired = self.parse_vm_stat_line(line) * page_size;
                } else if line.contains("Pages occupied by compressor:") {
                    compressed = self.parse_vm_stat_line(line) * page_size;
                } else if line.contains("File-backed pages:") {
                    cached = self.parse_vm_stat_line(line) * page_size;
                }
            }

            return (active, inactive, wired, compressed, cached);
        }

        (0, 0, 0, 0, 0)
    }

    fn parse_vm_stat_line(&self, line: &str) -> u64 {
        line.split(':')
            .nth(1)
            .and_then(|s| s.trim().trim_end_matches('.').parse().ok())
            .unwrap_or(0)
    }

    pub fn purge_memory(&self) -> Result<String, String> {
        // Note: purge requires sudo, so we use a different approach
        // We can use memory_pressure command or just return info
        let output = Command::new("sudo")
            .args(["purge"])
            .output()
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            Ok("Memoria purgada exitosamente".to_string())
        } else {
            // Try without sudo - just trigger garbage collection in apps
            Err("Se requieren permisos de administrador para purgar memoria".to_string())
        }
    }
}
