use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LaunchService {
    pub label: String,
    pub pid: Option<u32>,
    pub status: String,
    pub kind: String,
    pub last_exit_status: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServicesResult {
    pub user_agents: Vec<LaunchService>,
    pub user_daemons: Vec<LaunchService>,
    pub system_agents: Vec<LaunchService>,
}

pub struct DaemonsService;

impl DaemonsService {
    pub fn new() -> Self {
        Self
    }

    pub fn list_services(&self) -> Result<ServicesResult, String> {
        let mut user_agents = Vec::new();
        let mut user_daemons = Vec::new();
        let mut system_agents = Vec::new();

        // List user agents (gui domain)
        if let Ok(services) = self.list_domain_services("user", "gui") {
            user_agents = services.into_iter()
                .map(|mut s| { s.kind = "User Agent".to_string(); s })
                .collect();
        }

        // List user daemons
        if let Ok(services) = self.list_domain_services("user", "user") {
            user_daemons = services.into_iter()
                .map(|mut s| { s.kind = "User Daemon".to_string(); s })
                .collect();
        }

        // List system services (requires elevated permissions, so just try)
        if let Ok(services) = self.list_domain_services("system", "system") {
            system_agents = services.into_iter()
                .map(|mut s| { s.kind = "System".to_string(); s })
                .collect();
        }

        Ok(ServicesResult {
            user_agents,
            user_daemons,
            system_agents,
        })
    }

    fn list_domain_services(&self, domain_type: &str, _domain_name: &str) -> Result<Vec<LaunchService>, String> {
        let output = if domain_type == "system" {
            Command::new("launchctl")
                .args(["list"])
                .output()
                .map_err(|e| format!("Failed to run launchctl: {}", e))?
        } else {
            Command::new("launchctl")
                .args(["list"])
                .output()
                .map_err(|e| format!("Failed to run launchctl: {}", e))?
        };

        if !output.status.success() {
            return Err("launchctl command failed".to_string());
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut services = Vec::new();

        for line in output_str.lines().skip(1) {
            // Format: PID\tStatus\tLabel
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 3 {
                let pid = parts[0].trim().parse::<u32>().ok();
                let last_exit_status = parts[1].trim().parse::<i32>().ok();
                let label = parts[2].trim().to_string();

                // Skip Apple system services for cleaner list
                if label.starts_with("com.apple.") && domain_type != "system" {
                    continue;
                }

                let status = if pid.is_some() {
                    "running".to_string()
                } else if last_exit_status == Some(0) {
                    "stopped".to_string()
                } else if last_exit_status.is_some() {
                    "error".to_string()
                } else {
                    "unknown".to_string()
                };

                services.push(LaunchService {
                    label,
                    pid,
                    status,
                    kind: String::new(),
                    last_exit_status,
                });
            }
        }

        // Sort by label
        services.sort_by(|a, b| a.label.cmp(&b.label));

        Ok(services)
    }

    pub fn start_service(&self, label: &str) -> Result<String, String> {
        // Try to start using launchctl kickstart
        let output = Command::new("launchctl")
            .args(["start", label])
            .output()
            .map_err(|e| format!("Failed to start service: {}", e))?;

        if output.status.success() {
            Ok(format!("Started {}", label))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.is_empty() {
                Ok(format!("Started {} (or already running)", label))
            } else {
                Err(stderr.to_string())
            }
        }
    }

    pub fn stop_service(&self, label: &str) -> Result<String, String> {
        let output = Command::new("launchctl")
            .args(["stop", label])
            .output()
            .map_err(|e| format!("Failed to stop service: {}", e))?;

        if output.status.success() {
            Ok(format!("Stopped {}", label))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.is_empty() {
                Ok(format!("Stopped {} (or already stopped)", label))
            } else {
                Err(stderr.to_string())
            }
        }
    }

    pub fn get_service_info(&self, label: &str) -> Result<String, String> {
        let output = Command::new("launchctl")
            .args(["print", &format!("gui/{}/{}", self.get_user_id(), label)])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                return Ok(String::from_utf8_lossy(&output.stdout).to_string());
            }
        }

        // Fallback to list format
        let output = Command::new("launchctl")
            .args(["list", label])
            .output()
            .map_err(|e| format!("Failed to get service info: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err("Service not found or access denied".to_string())
        }
    }

    fn get_user_id(&self) -> u32 {
        // Get current user ID
        Command::new("id")
            .arg("-u")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().parse::<u32>().unwrap_or(501))
            .unwrap_or(501)
    }
}
