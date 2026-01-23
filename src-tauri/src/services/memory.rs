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

            // Parse page size dynamically from vm_stat output
            // First line typically contains: "Mach Virtual Memory Statistics: (page size of XXXX bytes)"
            let page_size = self.parse_page_size(&stdout);

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

    /// Parse page size from vm_stat output header.
    /// Falls back to system page size via libc if parsing fails.
    fn parse_page_size(&self, vm_stat_output: &str) -> u64 {
        // Try to parse from vm_stat header: "Mach Virtual Memory Statistics: (page size of 16384 bytes)"
        if let Some(first_line) = vm_stat_output.lines().next() {
            if let Some(start) = first_line.find("page size of ") {
                let after_prefix = &first_line[start + 13..];
                if let Some(end) = after_prefix.find(' ') {
                    if let Ok(size) = after_prefix[..end].parse::<u64>() {
                        return size;
                    }
                }
            }
        }

        // Fallback: use libc to get system page size
        // This works correctly on both Intel (4096) and Apple Silicon (16384)
        unsafe { libc::sysconf(libc::_SC_PAGESIZE) as u64 }
    }

    fn parse_vm_stat_line(&self, line: &str) -> u64 {
        line.split(':')
            .nth(1)
            .and_then(|s| s.trim().trim_end_matches('.').parse().ok())
            .unwrap_or(0)
    }

    pub fn purge_memory(&self) -> Result<String, String> {
        // The 'purge' command requires sudo privileges.
        // We provide a helpful message directing users to use Terminal.
        Err(
            "Para purgar memoria, ejecuta en Terminal: sudo purge\n\n\
             Esta operación requiere permisos de administrador y no puede \
             ejecutarse directamente desde la aplicación por razones de seguridad."
                .to_string(),
        )
    }

    /// Parse vm_stat output for testing purposes.
    /// Returns (active, inactive, wired, compressed, cached) in pages.
    #[cfg(test)]
    pub fn parse_vm_stat_output(&self, text: &str) -> (u64, u64, u64, u64, u64) {
        let page_size = self.parse_page_size(text);

        let mut active = 0u64;
        let mut inactive = 0u64;
        let mut wired = 0u64;
        let mut compressed = 0u64;
        let mut cached = 0u64;

        for line in text.lines() {
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

        (active, inactive, wired, compressed, cached)
    }

    #[cfg(test)]
    pub fn parse_page_size_for_test(&self, text: &str) -> u64 {
        self.parse_page_size(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!("../../tests/fixtures/vm_stat.txt");

    #[test]
    fn test_parse_page_size() {
        let service = MemoryService::new();
        let page_size = service.parse_page_size_for_test(FIXTURE);
        assert_eq!(page_size, 16384);
    }

    #[test]
    fn test_parse_page_size_4k() {
        let service = MemoryService::new();
        let intel_output = "Mach Virtual Memory Statistics: (page size of 4096 bytes)\nPages free: 123456.";
        let page_size = service.parse_page_size_for_test(intel_output);
        assert_eq!(page_size, 4096);
    }

    #[test]
    fn test_parse_vm_stat_line() {
        let service = MemoryService::new();
        let line = "Pages active:                            456789.";
        let pages = service.parse_vm_stat_line(line);
        assert_eq!(pages, 456789);
    }

    #[test]
    fn test_parse_vm_stat_output() {
        let service = MemoryService::new();
        let (active, inactive, wired, compressed, cached) = service.parse_vm_stat_output(FIXTURE);

        // Values from fixture multiplied by page_size (16384)
        assert_eq!(active, 456789 * 16384);
        assert_eq!(inactive, 234567 * 16384);
        assert_eq!(wired, 345678 * 16384);
        assert_eq!(compressed, 123456 * 16384);
        assert_eq!(cached, 567890 * 16384);
    }

    #[test]
    fn test_purge_memory_returns_error_message() {
        let service = MemoryService::new();
        let result = service.purge_memory();
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(msg.contains("sudo purge"));
    }
}
