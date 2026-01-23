use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;

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
}

pub struct PortScannerService;

impl PortScannerService {
    pub fn new() -> Self {
        Self
    }

    pub async fn scan(&self) -> Result<Vec<PortInfo>, Box<dyn std::error::Error + Send + Sync>> {
        let output = Command::new("lsof")
            .args(["-i", "-P", "-n"])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut ports_map: HashMap<(u16, u32), PortInfo> = HashMap::new();

        for line in stdout.lines().skip(1) {
            if let Some(port_info) = self.parse_lsof_line(line) {
                // Only keep LISTEN entries or unique port/pid combinations
                if line.contains("LISTEN") || line.contains("ESTABLISHED") {
                    let key = (port_info.port, port_info.pid);
                    if !ports_map.contains_key(&key) {
                        ports_map.insert(key, port_info);
                    }
                }
            }
        }

        // Filter to only listening ports (servers)
        let mut listening_ports: Vec<PortInfo> = ports_map
            .into_values()
            .filter(|p| p.port > 0)
            .collect();

        // Enrich with process info
        for port in &mut listening_ports {
            if let Some((cmd, cwd)) = self.get_process_info(port.pid) {
                port.command = Some(cmd);
                port.working_dir = cwd;
            }
            port.service_type = self.detect_service_type(port.port, &port.process_name);
        }

        // Sort by port number
        listening_ports.sort_by(|a, b| a.port.cmp(&b.port));

        Ok(listening_ports)
    }

    fn parse_lsof_line(&self, line: &str) -> Option<PortInfo> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 9 {
            return None;
        }

        let process_name = parts[0].to_string();
        let pid: u32 = parts[1].parse().ok()?;

        // Find the NAME column (last part usually contains address:port)
        let name_part = parts.last()?;

        // Parse address:port
        if let Some((addr, port_str)) = name_part.rsplit_once(':') {
            // Handle cases like "*:3000" or "127.0.0.1:3000"
            let port: u16 = port_str.parse().ok()?;
            let local_address = if addr == "*" {
                "0.0.0.0".to_string()
            } else {
                addr.to_string()
            };

            // Determine protocol from the line
            let protocol = if line.contains("TCP") {
                "TCP"
            } else if line.contains("UDP") {
                "UDP"
            } else {
                "TCP"
            }
            .to_string();

            return Some(PortInfo {
                port,
                pid,
                process_name,
                service_type: String::new(), // Will be filled later
                protocol,
                local_address,
                working_dir: None,
                command: None,
            });
        }

        None
    }

    fn get_process_info(&self, pid: u32) -> Option<(String, Option<String>)> {
        // Get command line
        let cmd_output = Command::new("ps")
            .args(["-p", &pid.to_string(), "-o", "command="])
            .output()
            .ok()?;

        let cmd = String::from_utf8_lossy(&cmd_output.stdout)
            .trim()
            .to_string();

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

        Some((cmd, cwd))
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
}
