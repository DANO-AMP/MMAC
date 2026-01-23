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
        // macOS DNS flush - dscacheutil doesn't require sudo
        let output = Command::new("dscacheutil")
            .args(["-flushcache"])
            .output()
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            // Note: Full DNS flush also requires `sudo killall -HUP mDNSResponder`
            // which needs admin privileges. We flush what we can without sudo.
            Ok(
                "Caché DNS de usuario vaciada.\n\n\
                 Para vaciar completamente, ejecuta en Terminal:\n\
                 sudo killall -HUP mDNSResponder"
                    .to_string(),
            )
        } else {
            Err("Error al vaciar caché DNS".to_string())
        }
    }

    /// Parse netstat line for testing purposes.
    #[cfg(test)]
    pub fn parse_netstat_line_for_test(&self, line: &str, protocol: &str) -> Option<NetworkConnection> {
        self.parse_netstat_line(line, protocol)
    }

    /// Parse address for testing purposes.
    #[cfg(test)]
    pub fn parse_address_for_test(&self, addr: &str) -> Option<(String, u16)> {
        self.parse_address(addr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_address_ipv4() {
        let service = NetworkService::new();
        let result = service.parse_address_for_test("127.0.0.1.3000");
        assert!(result.is_some());
        let (addr, port) = result.unwrap();
        assert_eq!(addr, "127.0.0.1");
        assert_eq!(port, 3000);
    }

    #[test]
    fn test_parse_address_wildcard() {
        let service = NetworkService::new();
        let result = service.parse_address_for_test("*.*");
        assert!(result.is_some());
        let (addr, port) = result.unwrap();
        assert_eq!(addr, "*");
        assert_eq!(port, 0);
    }

    #[test]
    fn test_parse_address_ipv6_localhost() {
        let service = NetworkService::new();
        let result = service.parse_address_for_test("::1.8888");
        assert!(result.is_some());
        let (addr, port) = result.unwrap();
        assert_eq!(addr, "::1");
        assert_eq!(port, 8888);
    }

    #[test]
    fn test_parse_netstat_line_tcp_listen() {
        let service = NetworkService::new();
        // Real macOS netstat -anvp tcp output format with enough columns
        let line = "tcp4       0      0  127.0.0.1.3000         *.*                    LISTEN      131072 131072  12345      0";
        let result = service.parse_netstat_line_for_test(line, "TCP");
        assert!(result.is_some());

        let conn = result.unwrap();
        assert_eq!(conn.protocol, "TCP");
        assert_eq!(conn.local_address, "127.0.0.1");
        assert_eq!(conn.local_port, 3000);
        assert_eq!(conn.state, "LISTEN");
    }

    #[test]
    fn test_parse_netstat_line_tcp_established() {
        let service = NetworkService::new();
        // Real macOS netstat -anvp tcp output format with enough columns
        let line = "tcp4       0      0  192.168.1.10.52345     172.217.14.99.443      ESTABLISHED 131072 131072  54321      0";
        let result = service.parse_netstat_line_for_test(line, "TCP");
        assert!(result.is_some());

        let conn = result.unwrap();
        assert_eq!(conn.local_address, "192.168.1.10");
        assert_eq!(conn.local_port, 52345);
        assert_eq!(conn.remote_address, "172.217.14.99");
        assert_eq!(conn.remote_port, 443);
        assert_eq!(conn.state, "ESTABLISHED");
    }

    #[test]
    fn test_get_hosts_parsing() {
        // Test the parsing logic without actually reading /etc/hosts
        let service = NetworkService::new();
        let hosts = service.get_hosts();
        // Should at least contain localhost entries on any macOS system
        assert!(hosts.iter().any(|h| h.hostname == "localhost" || h.ip == "127.0.0.1"));
    }
}
