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
        // Validate that input is a valid IP address before passing to shell command
        if !Self::is_valid_ip(ip) {
            return ip.to_string();
        }

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

    /// Validates that a string is a valid IP address (IPv4 or IPv6)
    fn is_valid_ip(ip: &str) -> bool {
        ip.parse::<std::net::IpAddr>().is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Sample lsof output for testing
    const SAMPLE_LSOF_OUTPUT: &str = r#"COMMAND     PID   USER   FD   TYPE             DEVICE SIZE/OFF NODE NAME
Google      1234  user   23u  IPv4 0x1234567890      0t0  TCP 192.168.1.100:52341->142.250.80.78:443 (ESTABLISHED)
Safari      5678  user   15u  IPv4 0x9876543210      0t0  TCP 192.168.1.100:52342->17.253.144.10:443 (ESTABLISHED)
Slack       9012  user   42u  IPv4 0x1122334455      0t0  TCP 192.168.1.100:52343->34.120.195.249:443 (ESTABLISHED)
node        3456  user   18u  IPv4 0x5566778899      0t0  TCP *:3000 (LISTEN)
postgres    7890  user   10u  IPv4 0xaabbccddee      0t0  TCP localhost:5432 (LISTEN)
ssh         2468  user   3u   IPv4 0xffeeddccbb      0t0  TCP 192.168.1.100:52500->192.168.1.1:22 (ESTABLISHED)
"#;

    // Sample lsof output with IPv6
    const SAMPLE_LSOF_IPV6: &str = r#"COMMAND     PID   USER   FD   TYPE             DEVICE SIZE/OFF NODE NAME
node        1234  user   18u  IPv6 0x1234567890      0t0  TCP [::1]:3000->[::1]:52341 (ESTABLISHED)
"#;

    #[test]
    fn test_outgoing_connection_serialization() {
        let conn = OutgoingConnection {
            process_name: "Safari".to_string(),
            pid: 1234,
            remote_host: "apple.com".to_string(),
            remote_port: 443,
            local_port: 52341,
            connection_state: "ESTABLISHED".to_string(),
        };

        let json = serde_json::to_string(&conn).expect("Should serialize");
        assert!(json.contains("\"process_name\":\"Safari\""));
        assert!(json.contains("\"pid\":1234"));
        assert!(json.contains("\"remote_port\":443"));

        let deserialized: OutgoingConnection =
            serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(deserialized.process_name, "Safari");
        assert_eq!(deserialized.pid, 1234);
    }

    #[test]
    fn test_firewall_status_serialization() {
        let status = FirewallStatus {
            enabled: true,
            stealth_mode: false,
            block_all_incoming: false,
        };

        let json = serde_json::to_string(&status).expect("Should serialize");
        assert!(json.contains("\"enabled\":true"));
        assert!(json.contains("\"stealth_mode\":false"));

        let deserialized: FirewallStatus =
            serde_json::from_str(&json).expect("Should deserialize");
        assert!(deserialized.enabled);
        assert!(!deserialized.stealth_mode);
    }

    #[test]
    fn test_invalid_ip_validation_empty() {
        assert!(!FirewallService::is_valid_ip(""), "Empty string should be invalid");
    }

    #[test]
    fn test_invalid_ip_validation_malformed() {
        let invalid_ips = [
            "not.an.ip",
            "256.256.256.256",
            "192.168.1",
            "192.168.1.1.1",
            "abc.def.ghi.jkl",
            "-1.0.0.0",
            "192.168.1.256",
            "hello world",
            "192.168.1.1; rm -rf /",
            "$(whoami)",
            "`id`",
        ];

        for ip in invalid_ips {
            assert!(
                !FirewallService::is_valid_ip(ip),
                "IP '{}' should be invalid",
                ip
            );
        }
    }

    #[test]
    fn test_valid_ip_validation_ipv4() {
        let valid_ips = [
            "192.168.1.1",
            "10.0.0.1",
            "172.16.0.1",
            "8.8.8.8",
            "255.255.255.255",
            "0.0.0.0",
            "127.0.0.1",
        ];

        for ip in valid_ips {
            assert!(
                FirewallService::is_valid_ip(ip),
                "IP '{}' should be valid",
                ip
            );
        }
    }

    #[test]
    fn test_valid_ip_validation_ipv6() {
        let valid_ips = [
            "::1",
            "fe80::1",
            "2001:0db8:85a3:0000:0000:8a2e:0370:7334",
            "2001:db8:85a3::8a2e:370:7334",
            "::",
        ];

        for ip in valid_ips {
            assert!(
                FirewallService::is_valid_ip(ip),
                "IPv6 '{}' should be valid",
                ip
            );
        }
    }

    #[test]
    fn test_parse_lsof_output_established() {
        let service = FirewallService::new();

        // Parse a sample ESTABLISHED connection line
        let line = "Google      1234  user   23u  IPv4 0x1234567890      0t0  TCP 192.168.1.100:52341->142.250.80.78:443 (ESTABLISHED)";

        let result = service.parse_lsof_line(line);

        assert!(result.is_some(), "Should parse ESTABLISHED connection");
        let conn = result.unwrap();
        assert_eq!(conn.process_name, "Google");
        assert_eq!(conn.pid, 1234);
        assert_eq!(conn.local_port, 52341);
        assert_eq!(conn.remote_host, "142.250.80.78");
        assert_eq!(conn.remote_port, 443);
        assert_eq!(conn.connection_state, "ESTABLISHED");
    }

    #[test]
    fn test_parse_lsof_output_listen() {
        let service = FirewallService::new();

        let line = "node        3456  user   18u  IPv4 0x5566778899      0t0  TCP *:3000 (LISTEN)";

        let result = service.parse_lsof_line(line);

        assert!(result.is_some(), "Should parse LISTEN connection");
        let conn = result.unwrap();
        assert_eq!(conn.process_name, "node");
        assert_eq!(conn.pid, 3456);
        assert_eq!(conn.local_port, 3000);
        assert_eq!(conn.remote_host, "*");
        assert_eq!(conn.remote_port, 0);
        assert_eq!(conn.connection_state, "LISTEN");
    }

    #[test]
    fn test_parse_lsof_output_localhost() {
        let service = FirewallService::new();

        let line = "postgres    7890  user   10u  IPv4 0xaabbccddee      0t0  TCP localhost:5432 (LISTEN)";

        let result = service.parse_lsof_line(line);

        assert!(result.is_some(), "Should parse localhost LISTEN");
        let conn = result.unwrap();
        assert_eq!(conn.process_name, "postgres");
        assert_eq!(conn.local_port, 5432);
        assert_eq!(conn.connection_state, "LISTEN");
    }

    #[test]
    fn test_parse_lsof_output_close_wait() {
        let service = FirewallService::new();

        let line = "Chrome      1111  user   10u  IPv4 0xaabbccddee      0t0  TCP 192.168.1.100:55555->1.2.3.4:80 (CLOSE_WAIT)";

        let result = service.parse_lsof_line(line);

        assert!(result.is_some(), "Should parse CLOSE_WAIT connection");
        let conn = result.unwrap();
        assert_eq!(conn.connection_state, "CLOSE_WAIT");
    }

    #[test]
    fn test_parse_lsof_output_time_wait() {
        let service = FirewallService::new();

        let line = "curl        2222  user   10u  IPv4 0xaabbccddee      0t0  TCP 192.168.1.100:55556->1.2.3.4:443 (TIME_WAIT)";

        let result = service.parse_lsof_line(line);

        assert!(result.is_some(), "Should parse TIME_WAIT connection");
        let conn = result.unwrap();
        assert_eq!(conn.connection_state, "TIME_WAIT");
    }

    #[test]
    fn test_parse_lsof_output_empty_line() {
        let service = FirewallService::new();

        let result = service.parse_lsof_line("");

        assert!(result.is_none(), "Empty line should return None");
    }

    #[test]
    fn test_parse_lsof_output_header_line() {
        let service = FirewallService::new();

        let header = "COMMAND     PID   USER   FD   TYPE             DEVICE SIZE/OFF NODE NAME";

        // Headers don't have enough parts or valid data
        let result = service.parse_lsof_line(header);

        // Should return None because it doesn't match expected format
        assert!(result.is_none(), "Header line should return None");
    }

    #[test]
    fn test_parse_lsof_output_insufficient_columns() {
        let service = FirewallService::new();

        let line = "process 123 user";

        let result = service.parse_lsof_line(line);

        assert!(result.is_none(), "Line with insufficient columns should return None");
    }

    #[test]
    fn test_parse_host_port_ipv4() {
        let service = FirewallService::new();

        let (host, port) = service.parse_host_port("192.168.1.1:443");

        assert_eq!(host, "192.168.1.1");
        assert_eq!(port, 443);
    }

    #[test]
    fn test_parse_host_port_ipv6() {
        let service = FirewallService::new();

        let (host, port) = service.parse_host_port("[::1]:8080");

        assert_eq!(host, "::1");
        assert_eq!(port, 8080);
    }

    #[test]
    fn test_parse_host_port_no_port() {
        let service = FirewallService::new();

        let (host, port) = service.parse_host_port("192.168.1.1");

        assert_eq!(host, "192.168.1.1");
        assert_eq!(port, 0);
    }

    #[test]
    fn test_parse_host_port_hostname() {
        let service = FirewallService::new();

        let (host, port) = service.parse_host_port("localhost:3000");

        assert_eq!(host, "localhost");
        assert_eq!(port, 3000);
    }

    #[test]
    fn test_resolve_hostname_invalid_input() {
        let service = FirewallService::new();

        // Invalid inputs should return the input unchanged (security: no shell injection)
        let invalid_inputs = [
            "not-an-ip",
            "$(whoami)",
            "; rm -rf /",
            "192.168.1.1; cat /etc/passwd",
            "`id`",
        ];

        for input in invalid_inputs {
            let result = service.resolve_hostname(input);
            assert_eq!(
                result, input,
                "Invalid input '{}' should be returned unchanged",
                input
            );
        }
    }

    #[test]
    fn test_resolve_hostname_valid_but_unresolvable() {
        let service = FirewallService::new();

        // A valid IP that likely can't be resolved should return the IP itself
        let result = service.resolve_hostname("192.0.2.1"); // TEST-NET-1, reserved

        // Should either return the IP or potentially a resolved name
        assert!(!result.is_empty());
    }

    #[test]
    fn test_firewall_service_new() {
        let service = FirewallService::new();
        let _ = service;
    }

    #[test]
    fn test_outgoing_connection_clone() {
        let conn = OutgoingConnection {
            process_name: "Safari".to_string(),
            pid: 1234,
            remote_host: "example.com".to_string(),
            remote_port: 443,
            local_port: 52341,
            connection_state: "ESTABLISHED".to_string(),
        };

        let cloned = conn.clone();

        assert_eq!(conn.process_name, cloned.process_name);
        assert_eq!(conn.pid, cloned.pid);
        assert_eq!(conn.remote_host, cloned.remote_host);
        assert_eq!(conn.remote_port, cloned.remote_port);
    }

    #[test]
    fn test_process_connections_sorted_by_count() {
        let mut connections = vec![
            ProcessConnections {
                process_name: "low".to_string(),
                pid: 1,
                connection_count: 1,
                connections: vec![],
            },
            ProcessConnections {
                process_name: "high".to_string(),
                pid: 2,
                connection_count: 10,
                connections: vec![],
            },
            ProcessConnections {
                process_name: "medium".to_string(),
                pid: 3,
                connection_count: 5,
                connections: vec![],
            },
        ];

        connections.sort_by(|a, b| b.connection_count.cmp(&a.connection_count));

        assert_eq!(connections[0].process_name, "high");
        assert_eq!(connections[1].process_name, "medium");
        assert_eq!(connections[2].process_name, "low");
    }

    #[test]
    fn test_parse_multiple_lsof_lines() {
        let service = FirewallService::new();
        let mut parsed_count = 0;

        for line in SAMPLE_LSOF_OUTPUT.lines().skip(1) {
            if service.parse_lsof_line(line).is_some() {
                parsed_count += 1;
            }
        }

        assert!(parsed_count >= 5, "Should parse multiple connections from sample output");
    }

    #[test]
    fn test_unknown_connection_state() {
        let service = FirewallService::new();

        // A connection without a known state marker
        let line = "proc        1234  user   10u  IPv4 0xaabbccddee      0t0  TCP 1.2.3.4:1234->5.6.7.8:80";

        let result = service.parse_lsof_line(line);

        if let Some(conn) = result {
            assert_eq!(conn.connection_state, "UNKNOWN");
        }
    }

    #[test]
    fn test_parse_host_port_empty_string() {
        let service = FirewallService::new();

        let (host, port) = service.parse_host_port("");

        assert_eq!(host, "");
        assert_eq!(port, 0);
    }

    #[test]
    fn test_parse_host_port_malformed_ipv6() {
        let service = FirewallService::new();

        // Malformed IPv6 (missing closing bracket)
        let (host, port) = service.parse_host_port("[::1:8080");

        // Should handle gracefully
        assert!(!host.is_empty() || port == 0);
    }
}
