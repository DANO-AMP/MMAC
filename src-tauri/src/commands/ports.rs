use crate::services::scanner::{PortInfo, PortScannerService};
use tauri::command;
use std::sync::Mutex;
use std::collections::HashMap;

/// Information about a scanned process for validation
#[derive(Clone)]
pub(crate) struct ScannedProcess {
    #[allow(dead_code)]
    pid: u32,
    process_name: String,
}

/// State to track PIDs and their process names discovered through port scanning
/// Only allows killing processes that were found by the scanner
pub struct ScannedPidsState(pub Mutex<HashMap<u32, ScannedProcess>>);

impl Default for ScannedPidsState {
    fn default() -> Self {
        Self(Mutex::new(HashMap::new()))
    }
}

/// Protected system PIDs that should never be killed (increased threshold)
const PROTECTED_PID_THRESHOLD: u32 = 200;

/// Process names that are protected from being killed
const PROTECTED_PROCESS_NAMES: &[&str] = &[
    "kernel_task",
    "launchd",
    "WindowServer",
    "loginwindow",
    "Finder",
    "Dock",
    "SystemUIServer",
    "cfprefsd",
    "distnoted",
    "logd",
    "notifyd",
    "opendirectoryd",
    "securityd",
    "trustd",
    "UserEventAgent",
    "coreservicesd",
    "mds",
    "mds_stores",
    "configd",
    "coreaudiod",
    "audiomxd",
    "bluetoothd",
    "airportd",
    "apsd",
    "backupd",
    "cloudd",
    "CommCenter",
    "coreduetd",
    "dasd",
    "diagnosticd",
    "diskarbitrationd",
    "fseventsd",
    "hidd",
    "imagent",
    "installd",
    "kernelmanagerd",
    "locationd",
    "lsd",
    "mediaremoted",
    "netbiosd",
    "nfcd",
    "nsurlsessiond",
    "powerd",
    "rapportd",
    "runningboardd",
    "sharingd",
    "smd",
    "symptomsd",
    "timed",
    "usbd",
    "watchdogd",
    "wifid",
    "wirelessproxd",
];

/// Checks if a process name is in the protected list
fn is_protected_process(name: &str) -> bool {
    let name_lower = name.to_lowercase();
    PROTECTED_PROCESS_NAMES.iter().any(|protected| {
        name_lower == protected.to_lowercase()
    })
}

/// Gets the current process name for a PID (for atomic verification)
fn get_current_process_name(pid: u32) -> Option<String> {
    use std::process::Command;

    let output = Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "comm="])
        .output()
        .ok()?;

    if output.status.success() {
        let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !name.is_empty() {
            return Some(name);
        }
    }
    None
}

#[command]
pub async fn scan_ports(state: tauri::State<'_, ScannedPidsState>) -> Result<Vec<PortInfo>, String> {
    let service = PortScannerService::new();
    let ports = service.scan().await.map_err(|e| e.to_string())?;

    // Track all discovered PIDs with their process names
    let mut scanned_pids = state.0.lock().map_err(|e| e.to_string())?;
    scanned_pids.clear();
    for port in &ports {
        if port.pid > 0 {
            scanned_pids.insert(port.pid, ScannedProcess {
                pid: port.pid,
                process_name: port.process_name.clone(),
            });
        }
    }

    Ok(ports)
}

#[command]
pub async fn kill_process(pid: u32, state: tauri::State<'_, ScannedPidsState>) -> Result<(), String> {
    use std::process::Command;

    // Protect system-critical processes by PID threshold
    if pid < PROTECTED_PID_THRESHOLD {
        return Err(format!(
            "Cannot kill PID {}: system-critical process (PID < {})",
            pid, PROTECTED_PID_THRESHOLD
        ));
    }

    // Get the scanned process info and verify it exists
    let scanned_process = {
        let scanned_pids = state.0.lock().map_err(|e| e.to_string())?;
        match scanned_pids.get(&pid) {
            Some(process) => process.clone(),
            None => {
                return Err(format!(
                    "Cannot kill PID {}: not found in last port scan. Run a scan first.",
                    pid
                ));
            }
        }
    };

    // Check if the scanned process name is protected
    if is_protected_process(&scanned_process.process_name) {
        return Err(format!(
            "Cannot kill PID {}: '{}' is a protected system process",
            pid, scanned_process.process_name
        ));
    }

    // ATOMIC CHECK: Verify the process hasn't changed since scanning (prevents TOCTOU attacks)
    // This ensures we're killing the same process we scanned, not a reused PID
    let current_name = get_current_process_name(pid).ok_or_else(|| {
        format!("Cannot kill PID {}: process no longer exists", pid)
    })?;

    // Compare process names (handle potential path differences by comparing base names)
    let scanned_base = scanned_process.process_name.rsplit('/').next().unwrap_or(&scanned_process.process_name);
    let current_base = current_name.rsplit('/').next().unwrap_or(&current_name);

    if scanned_base != current_base {
        return Err(format!(
            "Cannot kill PID {}: process changed from '{}' to '{}' since last scan. Run a new scan.",
            pid, scanned_process.process_name, current_name
        ));
    }

    // Double-check the current process name isn't protected (defense in depth)
    if is_protected_process(&current_name) {
        return Err(format!(
            "Cannot kill PID {}: '{}' is a protected system process",
            pid, current_name
        ));
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

    // Remove from scanned list after successful kill
    {
        let mut scanned_pids = state.0.lock().map_err(|e| e.to_string())?;
        scanned_pids.remove(&pid);
    }

    Ok(())
}
