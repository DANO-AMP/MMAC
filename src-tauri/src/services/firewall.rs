use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OutgoingConnection {
    pub process_name: String,
    pub pid: u32,
    pub remote_host: String,
    pub remote_port: u16,
    pub local_port: u16,
    pub connection_state: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessConnections {
    pub process_name: String,
    pub pid: u32,
    pub connection_count: u32,
    pub connections: Vec<OutgoingConnection>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FirewallStatus {
    pub enabled: bool,
    pub stealth_mode: bool,
    pub block_all_incoming: bool,
}

pub struct FirewallService;

impl FirewallService {
    pub fn new() -> Self {
        Self
    }

    pub fn get_outgoing_connections(&self) -> Result<Vec<ProcessConnections>, String> {
        // Use lsof to get established connections
        let output = Command::new("lsof")
            .args(["-i", "-n", "-P"])
            .output()
            .map_err(|e| format!("Failed to run lsof: {}", e))?;

        if !output.status.success() {
            return Err("lsof command failed".to_string());
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut connections_by_process: HashMap<(String, u32), Vec<OutgoingConnection>> = HashMap::new();

        for line in output_str.lines().skip(1) {
            if let Some(conn) = self.parse_lsof_line(line) {
                let key = (conn.process_name.clone(), conn.pid);
                connections_by_process.entry(key).or_default().push(conn);
            }
        }

        let mut result: Vec<ProcessConnections> = connections_by_process
            .into_iter()
            .map(|((process_name, pid), connections)| ProcessConnections {
                process_name,
                pid,
                connection_count: connections.len() as u32,
                connections,
            })
            .collect();

        // Sort by connection count descending
        result.sort_by(|a, b| b.connection_count.cmp(&a.connection_count));

        Ok(result)
    }

    fn parse_lsof_line(&self, line: &str) -> Option<OutgoingConnection> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 9 {
            return None;
        }

        let process_name = parts[0].to_string();
        let pid = parts[1].parse::<u32>().ok()?;

        // Find the connection info (usually last column with -> format)
        let name_col = parts.get(8).or_else(|| parts.last())?;

        // Skip if not a network connection
        if !name_col.contains("->") && !name_col.contains(':') {
            return None;
        }

        // Parse connection state
        let connection_state = if line.contains("ESTABLISHED") {
            "ESTABLISHED".to_string()
        } else if line.contains("LISTEN") {
            "LISTEN".to_string()
        } else if line.contains("CLOSE_WAIT") {
            "CLOSE_WAIT".to_string()
        } else if line.contains("TIME_WAIT") {
            "TIME_WAIT".to_string()
        } else {
            "UNKNOWN".to_string()
        };

        // Parse remote and local addresses
        let (local_port, remote_host, remote_port) = if name_col.contains("->") {
            // Format: local:port->remote:port
            let parts: Vec<&str> = name_col.split("->").collect();
            let local = parts.get(0).unwrap_or(&"");
            let remote = parts.get(1).unwrap_or(&"");

            let local_port = local.rsplit(':').next()
                .and_then(|p| p.parse::<u16>().ok())
                .unwrap_or(0);

            let (remote_host, remote_port) = self.parse_host_port(remote);
            (local_port, remote_host, remote_port)
        } else if name_col.contains(':') {
            // Format: *:port (LISTEN) or host:port
            let local_port = name_col.rsplit(':').next()
                .and_then(|p| p.parse::<u16>().ok())
                .unwrap_or(0);
            (local_port, "*".to_string(), 0)
        } else {
            return None;
        };

        Some(OutgoingConnection {
            process_name,
            pid,
            remote_host,
            remote_port,
            local_port,
            connection_state,
        })
    }

    fn parse_host_port(&self, addr: &str) -> (String, u16) {
        // Handle IPv6 addresses [::1]:port
        if addr.starts_with('[') {
            if let Some(end_bracket) = addr.find(']') {
                let host = &addr[1..end_bracket];
                let port = addr[end_bracket + 1..].trim_start_matches(':')
                    .parse::<u16>().unwrap_or(0);
                return (host.to_string(), port);
            }
        }

        // Handle IPv4 addresses host:port
        if let Some(colon_pos) = addr.rfind(':') {
            let host = &addr[..colon_pos];
            let port = addr[colon_pos + 1..].parse::<u16>().unwrap_or(0);
            (host.to_string(), port)
        } else {
            (addr.to_string(), 0)
        }
    }

    pub fn get_firewall_status(&self) -> Result<FirewallStatus, String> {
        // Check macOS Application Firewall status
        let output = Command::new("defaults")
            .args(["read", "/Library/Preferences/com.apple.alf", "globalstate"])
            .output();

        let enabled = match output {
            Ok(out) if out.status.success() => {
                let val = String::from_utf8_lossy(&out.stdout).trim().to_string();
                val != "0"
            }
            _ => false,
        };

        // Check stealth mode
        let stealth_output = Command::new("defaults")
            .args(["read", "/Library/Preferences/com.apple.alf", "stealthenabled"])
            .output();

        let stealth_mode = match stealth_output {
            Ok(out) if out.status.success() => {
                let val = String::from_utf8_lossy(&out.stdout).trim().to_string();
                val == "1"
            }
            _ => false,
        };

        // Check block all incoming
        let block_output = Command::new("defaults")
            .args(["read", "/Library/Preferences/com.apple.alf", "allowsignedenabled"])
            .output();

        let block_all_incoming = match block_output {
            Ok(out) if out.status.success() => {
                let val = String::from_utf8_lossy(&out.stdout).trim().to_string();
                val == "0"
            }
            _ => false,
        };

        Ok(FirewallStatus {
            enabled,
            stealth_mode,
            block_all_incoming,
        })
    }

    pub fn resolve_hostname(&self, ip: &str) -> String {
        // Try to resolve IP to hostname
        let output = Command::new("host")
            .arg(ip)
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let result = String::from_utf8_lossy(&output.stdout);
                // Parse "IP domain name pointer hostname." format
                if let Some(hostname) = result.split("pointer").nth(1) {
                    let hostname = hostname.trim().trim_end_matches('.');
                    if !hostname.is_empty() {
                        return hostname.to_string();
                    }
                }
            }
        }

        ip.to_string()
    }
}
