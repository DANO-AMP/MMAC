use crate::services::scanner::{PortInfo, PortScannerService};
use tauri::command;

#[command]
pub async fn scan_ports() -> Result<Vec<PortInfo>, String> {
    let service = PortScannerService::new();
    service.scan().await.map_err(|e| e.to_string())
}

#[command]
pub async fn kill_process(pid: u32) -> Result<(), String> {
    use std::process::Command;

    Command::new("kill")
        .args(["-9", &pid.to_string()])
        .output()
        .map_err(|e| e.to_string())?;

    Ok(())
}
