use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_mb: f32,
    pub memory_percent: f32,
    pub user: String,
    pub state: String,
    pub threads: u32,
    pub command: String,
}

pub struct ProcessService;

impl ProcessService {
    pub fn new() -> Self {
        Self
    }

    pub fn get_all_processes(&self) -> Vec<ProcessInfo> {
        // Use ps to get all processes with CPU, memory, user, state, threads
        // Format: pid, %cpu, rss, %mem, user, state, threads, comm, args
        let output = Command::new("ps")
            .args([
                "-axo",
                "pid,%cpu,rss,%mem,user,state,wq,comm",
                "-r", // Sort by CPU usage descending
            ])
            .output();

        let mut processes = Vec::new();

        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);

            for line in stdout.lines().skip(1) {
                if let Some(proc) = self.parse_ps_line(line) {
                    processes.push(proc);
                }
            }
        }

        // Get full command for top processes
        for proc in processes.iter_mut().take(100) {
            if let Some(cmd) = self.get_full_command(proc.pid) {
                proc.command = cmd;
            }
        }

        processes
    }

    fn parse_ps_line(&self, line: &str) -> Option<ProcessInfo> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 8 {
            return None;
        }

        let pid: u32 = parts[0].parse().ok()?;
        let cpu_usage: f32 = parts[1].parse().unwrap_or(0.0);
        let rss_kb: f32 = parts[2].parse().unwrap_or(0.0);
        let memory_percent: f32 = parts[3].parse().unwrap_or(0.0);
        let user = parts[4].to_string();
        let state = self.parse_state(parts[5]);
        let threads: u32 = parts[6].parse().unwrap_or(1);
        let name = parts[7].to_string();

        Some(ProcessInfo {
            pid,
            name: name.clone(),
            cpu_usage,
            memory_mb: rss_kb / 1024.0,
            memory_percent,
            user,
            state,
            threads,
            command: name, // Will be enriched later for top processes
        })
    }

    fn parse_state(&self, state: &str) -> String {
        match state.chars().next() {
            Some('R') => "Ejecutando".to_string(),
            Some('S') => "Suspendido".to_string(),
            Some('I') => "Inactivo".to_string(),
            Some('U') => "Espera".to_string(),
            Some('Z') => "Zombie".to_string(),
            Some('T') => "Detenido".to_string(),
            _ => "Desconocido".to_string(),
        }
    }

    fn get_full_command(&self, pid: u32) -> Option<String> {
        let output = Command::new("ps")
            .args(["-p", &pid.to_string(), "-o", "args="])
            .output()
            .ok()?;

        let cmd = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if cmd.is_empty() {
            None
        } else {
            Some(cmd)
        }
    }

    pub fn kill_process(&self, pid: u32, force: bool) -> Result<(), String> {
        // Protect system-critical processes
        if pid < 100 {
            return Err(format!(
                "No se puede terminar el proceso {}: es un proceso del sistema",
                pid
            ));
        }

        let signal = if force { "-9" } else { "-15" };

        let output = Command::new("kill")
            .args([signal, &pid.to_string()])
            .output()
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Error al terminar proceso: {}", stderr))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_state() {
        let service = ProcessService::new();
        assert_eq!(service.parse_state("R"), "Ejecutando");
        assert_eq!(service.parse_state("S"), "Suspendido");
        assert_eq!(service.parse_state("I"), "Inactivo");
        assert_eq!(service.parse_state("Z"), "Zombie");
    }

    #[test]
    fn test_parse_ps_line() {
        let service = ProcessService::new();
        let line = "  1234  5.2 102400  2.5 user     S    4 process_name";
        let result = service.parse_ps_line(line);

        assert!(result.is_some());
        let proc = result.unwrap();
        assert_eq!(proc.pid, 1234);
        assert!((proc.cpu_usage - 5.2).abs() < 0.01);
        assert_eq!(proc.memory_mb, 100.0); // 102400 KB = 100 MB
        assert_eq!(proc.user, "user");
        assert_eq!(proc.state, "Suspendido");
    }

    #[test]
    fn test_kill_protected_pid() {
        let service = ProcessService::new();
        let result = service.kill_process(1, false);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("proceso del sistema"));
    }
}
