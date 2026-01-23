use crate::services::scanner::{PortInfo, PortScannerService};
use tauri::command;
use std::sync::Mutex;
use std::collections::HashSet;

/// State to track PIDs that were discovered through port scanning
/// Only allows killing processes that were found by the scanner
pub struct ScannedPidsState(pub Mutex<HashSet<u32>>);

impl Default for ScannedPidsState {
    fn default() -> Self {
        Self(Mutex::new(HashSet::new()))
    }
}

/// Protected system PIDs that should never be killed
const PROTECTED_PID_THRESHOLD: u32 = 100;

#[command]
pub async fn scan_ports(state: tauri::State<'_, ScannedPidsState>) -> Result<Vec<PortInfo>, String> {
    let service = PortScannerService::new();
    let ports = service.scan().await.map_err(|e| e.to_string())?;

    // Track all discovered PIDs
    let mut scanned_pids = state.0.lock().map_err(|e| e.to_string())?;
    scanned_pids.clear();
    for port in &ports {
        if port.pid > 0 {
            scanned_pids.insert(port.pid);
        }
    }

    Ok(ports)
}

#[command]
pub async fn kill_process(pid: u32, state: tauri::State<'_, ScannedPidsState>) -> Result<(), String> {
    use std::process::Command;

    // Protect system-critical processes
    if pid < PROTECTED_PID_THRESHOLD {
        return Err(format!(
            "Cannot kill PID {}: system-critical process (PID < {})",
            pid, PROTECTED_PID_THRESHOLD
        ));
    }

    // Verify the PID was discovered through scanning
    {
        let scanned_pids = state.0.lock().map_err(|e| e.to_string())?;
        if !scanned_pids.contains(&pid) {
            return Err(format!(
                "Cannot kill PID {}: not found in last port scan. Run a scan first.",
                pid
            ));
        }
    }

    // Use SIGTERM (15) first for graceful shutdown instead of SIGKILL (9)
    let output = Command::new("kill")
        .args(["-15", &pid.to_string()])
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to terminate process {}: {}", pid, stderr));
    }

    Ok(())
}
