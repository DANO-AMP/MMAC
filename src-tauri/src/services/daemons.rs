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

    /// Parse launchctl list output into LaunchService entries.
    /// Extracted so tests can exercise parsing without running commands.
    #[cfg(test)]
    fn parse_launchctl_output(&self, output: &str, domain_type: &str) -> Vec<LaunchService> {
        let mut services = Vec::new();

        for line in output.lines().skip(1) {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 3 {
                let pid = parts[0].trim().parse::<u32>().ok();
                let last_exit_status = parts[1].trim().parse::<i32>().ok();
                let label = parts[2].trim().to_string();

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

        services.sort_by(|a, b| a.label.cmp(&b.label));
        services
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Struct serialization tests ----

    #[test]
    fn test_launch_service_serialization_roundtrip() {
        let service = LaunchService {
            label: "com.example.agent".to_string(),
            pid: Some(1234),
            status: "running".to_string(),
            kind: "User Agent".to_string(),
            last_exit_status: Some(0),
        };

        let json = serde_json::to_string(&service).expect("serialize");
        let deserialized: LaunchService = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.label, "com.example.agent");
        assert_eq!(deserialized.pid, Some(1234));
        assert_eq!(deserialized.status, "running");
        assert_eq!(deserialized.kind, "User Agent");
        assert_eq!(deserialized.last_exit_status, Some(0));
    }

    #[test]
    fn test_launch_service_serialization_with_none_fields() {
        let service = LaunchService {
            label: "com.example.daemon".to_string(),
            pid: None,
            status: "stopped".to_string(),
            kind: "System".to_string(),
            last_exit_status: None,
        };

        let json = serde_json::to_string(&service).expect("serialize");
        let deserialized: LaunchService = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.pid, None);
        assert_eq!(deserialized.last_exit_status, None);
    }

    #[test]
    fn test_services_result_serialization_roundtrip() {
        let result = ServicesResult {
            user_agents: vec![LaunchService {
                label: "com.example.agent".to_string(),
                pid: Some(100),
                status: "running".to_string(),
                kind: "User Agent".to_string(),
                last_exit_status: Some(0),
            }],
            user_daemons: vec![],
            system_agents: vec![],
        };

        let json = serde_json::to_string(&result).expect("serialize");
        let deserialized: ServicesResult = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.user_agents.len(), 1);
        assert!(deserialized.user_daemons.is_empty());
        assert!(deserialized.system_agents.is_empty());
    }

    // ---- Parsing tests using parse_launchctl_output ----

    const SAMPLE_LAUNCHCTL_OUTPUT: &str = "PID\tStatus\tLabel\n\
        1234\t0\tcom.example.running\n\
        -\t0\tcom.example.stopped\n\
        -\t78\tcom.example.errored\n\
        -\t-\tcom.example.unknown\n\
        5678\t0\tcom.apple.internal.service";

    #[test]
    fn test_parse_launchctl_output_running_service() {
        let service = DaemonsService::new();
        let services = service.parse_launchctl_output(SAMPLE_LAUNCHCTL_OUTPUT, "user");

        let running = services.iter().find(|s| s.label == "com.example.running").expect("find running");
        assert_eq!(running.pid, Some(1234));
        assert_eq!(running.status, "running");
        assert_eq!(running.last_exit_status, Some(0));
    }

    #[test]
    fn test_parse_launchctl_output_stopped_service() {
        let service = DaemonsService::new();
        let services = service.parse_launchctl_output(SAMPLE_LAUNCHCTL_OUTPUT, "user");

        let stopped = services.iter().find(|s| s.label == "com.example.stopped").expect("find stopped");
        assert_eq!(stopped.pid, None);
        assert_eq!(stopped.status, "stopped");
        assert_eq!(stopped.last_exit_status, Some(0));
    }

    #[test]
    fn test_parse_launchctl_output_errored_service() {
        let service = DaemonsService::new();
        let services = service.parse_launchctl_output(SAMPLE_LAUNCHCTL_OUTPUT, "user");

        let errored = services.iter().find(|s| s.label == "com.example.errored").expect("find errored");
        assert_eq!(errored.pid, None);
        assert_eq!(errored.status, "error");
        assert_eq!(errored.last_exit_status, Some(78));
    }

    #[test]
    fn test_parse_launchctl_output_unknown_service() {
        let service = DaemonsService::new();
        let services = service.parse_launchctl_output(SAMPLE_LAUNCHCTL_OUTPUT, "user");

        let unknown = services.iter().find(|s| s.label == "com.example.unknown").expect("find unknown");
        assert_eq!(unknown.pid, None);
        assert_eq!(unknown.status, "unknown");
        assert_eq!(unknown.last_exit_status, None);
    }

    #[test]
    fn test_parse_launchctl_output_filters_apple_services_for_user() {
        let service = DaemonsService::new();
        let services = service.parse_launchctl_output(SAMPLE_LAUNCHCTL_OUTPUT, "user");

        let apple = services.iter().find(|s| s.label == "com.apple.internal.service");
        assert!(apple.is_none(), "Apple services should be filtered for user domain");
    }

    #[test]
    fn test_parse_launchctl_output_keeps_apple_services_for_system() {
        let service = DaemonsService::new();
        let services = service.parse_launchctl_output(SAMPLE_LAUNCHCTL_OUTPUT, "system");

        let apple = services.iter().find(|s| s.label == "com.apple.internal.service");
        assert!(apple.is_some(), "Apple services should be kept for system domain");
    }

    #[test]
    fn test_parse_launchctl_output_sorted_by_label() {
        let service = DaemonsService::new();
        let services = service.parse_launchctl_output(SAMPLE_LAUNCHCTL_OUTPUT, "user");

        let labels: Vec<&str> = services.iter().map(|s| s.label.as_str()).collect();
        let mut sorted_labels = labels.clone();
        sorted_labels.sort();
        assert_eq!(labels, sorted_labels, "Services should be sorted by label");
    }

    #[test]
    fn test_parse_launchctl_output_empty_input() {
        let service = DaemonsService::new();
        let services = service.parse_launchctl_output("", "user");

        assert!(services.is_empty());
    }

    #[test]
    fn test_parse_launchctl_output_header_only() {
        let service = DaemonsService::new();
        let services = service.parse_launchctl_output("PID\tStatus\tLabel", "user");

        assert!(services.is_empty());
    }

    #[test]
    fn test_parse_launchctl_output_malformed_line() {
        let service = DaemonsService::new();
        // Line with only two columns
        let output = "PID\tStatus\tLabel\nonly\ttwo";
        let services = service.parse_launchctl_output(output, "user");

        assert!(services.is_empty());
    }

    #[test]
    fn test_parse_launchctl_output_negative_exit_status() {
        let service = DaemonsService::new();
        let output = "PID\tStatus\tLabel\n-\t-1\tcom.example.neg";
        let services = service.parse_launchctl_output(output, "user");

        assert_eq!(services.len(), 1);
        assert_eq!(services[0].last_exit_status, Some(-1));
        assert_eq!(services[0].status, "error");
    }
}
