use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use tracing::warn;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PortInfo {
    pub port: u16,
    pub pid: u32,
    pub process_name: String,
    pub service_type: String,
    pub protocol: String,
    pub local_address: String,
    pub working_dir: Option<String>,
    pub command: Option<String>,
    pub cpu_usage: f32,
    pub memory_mb: f32,
}

pub struct PortScannerService;

impl PortScannerService {
    pub fn new() -> Self {
        Self
    }

    pub async fn scan(&self) -> Result<Vec<PortInfo>, Box<dyn std::error::Error + Send + Sync>> {
        let listening_ports = tokio::task::spawn_blocking(move || {
            let output = Command::new("lsof")
                .args(["-i", "-P", "-n"])
                .output()?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                warn!("lsof command failed with status {}: {}", output.status, stderr);
                // Return empty list instead of failing completely
                return Ok::<Vec<PortInfo>, std::io::Error>(Vec::new());
            }

            let stdout = String::from_utf8_lossy(&output.stdout);

            // Validate output format - first line should be the header
            let first_line = stdout.lines().next().unwrap_or("");
            if !first_line.contains("COMMAND") || !first_line.contains("PID") {
                warn!(
                    "Unexpected lsof output format. Expected header with COMMAND and PID, got: '{}'",
                    first_line
                );
                return Ok(Vec::new());
            }

            let scanner = PortScannerService::new();
            let mut ports_map: HashMap<(u16, u32), PortInfo> = HashMap::new();
            let mut parse_errors = 0;

            for (line_num, line) in stdout.lines().skip(1).enumerate() {
                // Only process LISTEN entries (actual servers)
                if !line.contains("LISTEN") {
                    continue;
                }

                match scanner.parse_lsof_line(line) {
                    Some(port_info) => {
                        let key = (port_info.port, port_info.pid);
                        if !ports_map.contains_key(&key) {
                            ports_map.insert(key, port_info);
                        }
                    }
                    None => {
                        parse_errors += 1;
                        if parse_errors <= 5 {
                            // Only log first 5 errors to avoid log spam
                            warn!(
                                "Failed to parse lsof line {} (LISTEN entry): '{}'",
                                line_num + 2,
                                line
                            );
                        }
                    }
                }
            }

            if parse_errors > 5 {
                warn!(
                    "Suppressed {} additional lsof parse errors",
                    parse_errors - 5
                );
            }

            // Filter to only listening ports (servers)
            let mut listening_ports: Vec<PortInfo> = ports_map
                .into_values()
                .filter(|p| p.port > 0)
                .collect();

            // Enrich with process info
            for port in &mut listening_ports {
                if let Some((cmd, cwd, cpu, mem)) = scanner.get_process_info(port.pid) {
                    port.command = Some(cmd);
                    port.working_dir = cwd;
                    port.cpu_usage = cpu;
                    port.memory_mb = mem;
                }
                port.service_type = scanner.detect_service_type(port.port, &port.process_name);
            }

            // Sort by port number
            listening_ports.sort_by(|a, b| a.port.cmp(&b.port));

            Ok(listening_ports)
        }).await.map_err(|e| e.to_string())??;

        Ok(listening_ports)
    }

    fn parse_lsof_line(&self, line: &str) -> Option<PortInfo> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 9 {
            return None;
        }

        let process_name = parts[0].to_string();
        let pid: u32 = parts[1].parse().ok()?;

        // Find the NAME column (last or second-to-last part)
        let name_part = parts.last()?;

        // Skip if it's a state like (LISTEN) or (ESTABLISHED)
        let addr_part = if name_part.starts_with('(') {
            // The address is in the second-to-last position
            parts.get(parts.len() - 2)?
        } else {
            name_part
        };

        // For connections like "192.168.2.40:54850->104.199.65.9:80", take the local part
        let local_part = if addr_part.contains("->") {
            addr_part.split("->").next()?
        } else {
            addr_part
        };

        // Parse address:port - handle IPv6 with brackets like [::1]:8080
        let (addr, port_str) = if local_part.contains('[') {
            // IPv6 format: [::1]:8080
            let bracket_end = local_part.rfind(']')?;
            let addr = &local_part[1..bracket_end];
            let port = &local_part[bracket_end + 2..]; // Skip ]:
            (addr.to_string(), port)
        } else {
            // IPv4 or * format: 127.0.0.1:8080 or *:8080
            let colon_pos = local_part.rfind(':')?;
            let addr = &local_part[..colon_pos];
            let port = &local_part[colon_pos + 1..];
            (if addr == "*" { "0.0.0.0".to_string() } else { addr.to_string() }, port)
        };

        let port: u16 = port_str.parse().ok()?;

        // Determine protocol from the line
        let protocol = if line.contains("TCP") {
            "TCP"
        } else if line.contains("UDP") {
            "UDP"
        } else {
            "TCP"
        }
        .to_string();

        Some(PortInfo {
            port,
            pid,
            process_name,
            service_type: String::new(), // Will be filled later
            protocol,
            local_address: addr,
            working_dir: None,
            command: None,
            cpu_usage: 0.0,
            memory_mb: 0.0,
        })
    }

    fn get_process_info(&self, pid: u32) -> Option<(String, Option<String>, f32, f32)> {
        // Get command line, CPU and memory usage
        let cmd_output = Command::new("ps")
            .env("LC_ALL", "C")
            .args(["-p", &pid.to_string(), "-o", "command=,%cpu=,rss="])
            .output()
            .ok()?;

        if !cmd_output.status.success() {
            warn!(
                "ps command failed for PID {}: {}",
                pid,
                String::from_utf8_lossy(&cmd_output.stderr)
            );
            return None;
        }

        let output = String::from_utf8_lossy(&cmd_output.stdout);
        let output = output.trim();

        if output.is_empty() {
            warn!("Empty ps output for PID {}", pid);
            return None;
        }

        // Parse the output - format is: "command %cpu rss"
        // RSS is in KB, we convert to MB
        let parts: Vec<&str> = output.rsplitn(3, char::is_whitespace).collect();

        let (cmd, cpu, mem_kb) = if parts.len() >= 3 {
            let rss: f32 = match parts[0].trim().parse() {
                Ok(v) => v,
                Err(_) => {
                    warn!(
                        "Failed to parse RSS '{}' for PID {}, using fallback",
                        parts[0], pid
                    );
                    0.0
                }
            };
            let cpu: f32 = match parts[1].trim().parse() {
                Ok(v) => v,
                Err(_) => {
                    warn!(
                        "Failed to parse CPU '{}' for PID {}, using fallback",
                        parts[1], pid
                    );
                    0.0
                }
            };
            let cmd = parts[2].trim().to_string();
            (cmd, cpu, rss)
        } else {
            warn!(
                "Unexpected ps output format for PID {}: '{}' (expected 3 parts, got {})",
                pid,
                output,
                parts.len()
            );
            (output.to_string(), 0.0, 0.0)
        };

        let memory_mb = mem_kb / 1024.0;

        // Get working directory
        let cwd_output = Command::new("lsof")
            .args(["-p", &pid.to_string(), "-Fn"])
            .output()
            .ok()?;

        let cwd_stdout = String::from_utf8_lossy(&cwd_output.stdout);
        let cwd = cwd_stdout
            .lines()
            .find(|l| l.starts_with('n') && l.contains('/'))
            .map(|l| l[1..].to_string());

        Some((cmd, cwd, cpu, memory_mb))
    }

    fn detect_service_type(&self, port: u16, process_name: &str) -> String {
        let name_lower = process_name.to_lowercase();

        // Detect by process name first
        if name_lower.contains("node") || name_lower.contains("npm") {
            if port == 3000 {
                return "Next.js / React".to_string();
            } else if port == 5173 {
                return "Vite Dev Server".to_string();
            }
            return "Node.js Server".to_string();
        }

        if name_lower.contains("python") {
            if port == 8888 {
                return "Jupyter Notebook".to_string();
            } else if port == 8000 {
                return "Django / FastAPI".to_string();
            } else if port == 5000 {
                return "Flask".to_string();
            }
            return "Python Server".to_string();
        }

        if name_lower.contains("java") {
            return "Java Server".to_string();
        }

        if name_lower.contains("postgres") {
            return "PostgreSQL".to_string();
        }

        if name_lower.contains("mysql") {
            return "MySQL".to_string();
        }

        if name_lower.contains("mongo") {
            return "MongoDB".to_string();
        }

        if name_lower.contains("redis") {
            return "Redis".to_string();
        }

        if name_lower.contains("nginx") {
            return "Nginx".to_string();
        }

        if name_lower.contains("apache") || name_lower.contains("httpd") {
            return "Apache".to_string();
        }

        // Detect by port
        match port {
            80 => "HTTP Server".to_string(),
            443 => "HTTPS Server".to_string(),
            22 => "SSH".to_string(),
            3000 | 3001 | 4000 | 5000 | 8000 | 8080 => "HTTP Server".to_string(),
            3306 => "MySQL".to_string(),
            5432 => "PostgreSQL".to_string(),
            6379 => "Redis".to_string(),
            27017 => "MongoDB".to_string(),
            9200 => "Elasticsearch".to_string(),
            _ => "Service".to_string(),
        }
    }

    /// Parse lsof line for testing purposes.
    #[cfg(test)]
    pub fn parse_line_for_test(&self, line: &str) -> Option<PortInfo> {
        self.parse_lsof_line(line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_lsof_ipv4_listen() {
        let service = PortScannerService::new();
        let line = "node      12345 user   23u  IPv4 0x1234567890abcdef      0t0  TCP 127.0.0.1:3000 (LISTEN)";

        let result = service.parse_line_for_test(line);
        assert!(result.is_some());

        let port_info = result.unwrap();
        assert_eq!(port_info.port, 3000);
        assert_eq!(port_info.pid, 12345);
        assert_eq!(port_info.process_name, "node");
        assert_eq!(port_info.local_address, "127.0.0.1");
        assert_eq!(port_info.protocol, "TCP");
    }

    #[test]
    fn test_parse_lsof_wildcard_address() {
        let service = PortScannerService::new();
        let line = "java       3456 user   45u  IPv4 0x1234567890abcde3      0t0  TCP *:8080 (LISTEN)";

        let result = service.parse_line_for_test(line);
        assert!(result.is_some());

        let port_info = result.unwrap();
        assert_eq!(port_info.port, 8080);
        assert_eq!(port_info.local_address, "0.0.0.0");
    }

    #[test]
    fn test_parse_lsof_ipv6() {
        let service = PortScannerService::new();
        let line = "python3    4567 user   12u  IPv6 0x1234567890abcde4      0t0  TCP [::1]:8888 (LISTEN)";

        let result = service.parse_line_for_test(line);
        assert!(result.is_some());

        let port_info = result.unwrap();
        assert_eq!(port_info.port, 8888);
        assert_eq!(port_info.pid, 4567);
        assert_eq!(port_info.local_address, "::1");
    }

    #[test]
    fn test_detect_service_type_node() {
        let service = PortScannerService::new();

        assert_eq!(service.detect_service_type(3000, "node"), "Next.js / React");
        assert_eq!(service.detect_service_type(5173, "node"), "Vite Dev Server");
        assert_eq!(service.detect_service_type(4000, "node"), "Node.js Server");
    }

    #[test]
    fn test_detect_service_type_python() {
        let service = PortScannerService::new();

        assert_eq!(service.detect_service_type(8888, "python3"), "Jupyter Notebook");
        assert_eq!(service.detect_service_type(8000, "python3"), "Django / FastAPI");
        assert_eq!(service.detect_service_type(5000, "python3"), "Flask");
    }

    #[test]
    fn test_detect_service_type_databases() {
        let service = PortScannerService::new();

        assert_eq!(service.detect_service_type(5432, "postgres"), "PostgreSQL");
        assert_eq!(service.detect_service_type(3306, "mysqld"), "MySQL");
        assert_eq!(service.detect_service_type(27017, "mongod"), "MongoDB");
        assert_eq!(service.detect_service_type(6379, "redis-server"), "Redis");
    }

    #[test]
    fn test_detect_service_type_by_port_fallback() {
        let service = PortScannerService::new();

        // When process name doesn't match, fall back to port
        assert_eq!(service.detect_service_type(80, "unknown"), "HTTP Server");
        assert_eq!(service.detect_service_type(443, "unknown"), "HTTPS Server");
        assert_eq!(service.detect_service_type(22, "unknown"), "SSH");
    }
}
