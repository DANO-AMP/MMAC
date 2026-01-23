use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkConnection {
    pub protocol: String,
    pub local_address: String,
    pub local_port: u16,
    pub remote_address: String,
    pub remote_port: u16,
    pub state: String,
    pub pid: u32,
    pub process_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HostEntry {
    pub ip: String,
    pub hostname: String,
    pub comment: Option<String>,
}

pub struct NetworkService;

impl NetworkService {
    pub fn new() -> Self {
        Self
    }

    pub fn get_active_connections(&self) -> Vec<NetworkConnection> {
        let mut connections = Vec::new();

        // Get TCP connections
        let output = Command::new("netstat")
            .args(["-anvp", "tcp"])
            .output();

        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines().skip(2) {
                if let Some(conn) = self.parse_netstat_line(line, "TCP") {
                    connections.push(conn);
                }
            }
        }

        // Get UDP connections
        let output = Command::new("netstat")
            .args(["-anvp", "udp"])
            .output();

        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines().skip(2) {
                if let Some(conn) = self.parse_netstat_line(line, "UDP") {
                    connections.push(conn);
                }
            }
        }

        // Enrich with process names
        for conn in &mut connections {
            if conn.pid > 0 {
                conn.process_name = self.get_process_name(conn.pid);
            }
        }

        connections
    }

    fn parse_netstat_line(&self, line: &str, protocol: &str) -> Option<NetworkConnection> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 9 {
            return None;
        }

        // Parse local address
        let local = parts[3];
        let (local_addr, local_port) = self.parse_address(local)?;

        // Parse remote address
        let remote = parts[4];
        let (remote_addr, remote_port) = self.parse_address(remote)?;

        // State (for TCP)
        let state = if protocol == "TCP" && parts.len() > 5 {
            parts[5].to_string()
        } else {
            String::new()
        };

        // PID is typically the last column
        let pid: u32 = parts.last()?.parse().unwrap_or(0);

        Some(NetworkConnection {
            protocol: protocol.to_string(),
            local_address: local_addr,
            local_port,
            remote_address: remote_addr,
            remote_port,
            state,
            pid,
            process_name: String::new(),
        })
    }

    fn parse_address(&self, addr: &str) -> Option<(String, u16)> {
        // Handle formats like "192.168.1.1.443" or "*.80" or "127.0.0.1.8080"
        if addr == "*.*" {
            return Some(("*".to_string(), 0));
        }

        let last_dot = addr.rfind('.')?;
        let ip = &addr[..last_dot];
        let port: u16 = addr[last_dot + 1..].parse().unwrap_or(0);

        Some((ip.to_string(), port))
    }

    fn get_process_name(&self, pid: u32) -> String {
        let output = Command::new("ps")
            .args(["-p", &pid.to_string(), "-o", "comm="])
            .output();

        if let Ok(out) = output {
            return String::from_utf8_lossy(&out.stdout).trim().to_string();
        }
        String::new()
    }

    pub fn get_hosts(&self) -> Vec<HostEntry> {
        let mut entries = Vec::new();

        if let Ok(content) = std::fs::read_to_string("/etc/hosts") {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }

                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 2 {
                    let ip = parts[0].to_string();
                    let hostname = parts[1].to_string();
                    let comment = if parts.len() > 2 {
                        Some(parts[2..].join(" "))
                    } else {
                        None
                    };

                    entries.push(HostEntry { ip, hostname, comment });
                }
            }
        }

        entries
    }

    pub fn flush_dns(&self) -> Result<String, String> {
        // macOS DNS flush
        let output = Command::new("dscacheutil")
            .args(["-flushcache"])
            .output()
            .map_err(|e| e.to_string())?;

        // Also restart mDNSResponder
        let _ = Command::new("sudo")
            .args(["killall", "-HUP", "mDNSResponder"])
            .output();

        if output.status.success() {
            Ok("Caché DNS vaciada".to_string())
        } else {
            Ok("Caché DNS vaciada (parcial)".to_string())
        }
    }
}
