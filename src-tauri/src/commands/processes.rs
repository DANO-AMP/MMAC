use crate::services::processes::{ProcessInfo, ProcessService};
use tauri::command;

#[command]
pub fn get_all_processes() -> Vec<ProcessInfo> {
    let service = ProcessService::new();
    service.get_all_processes()
}

#[command]
pub fn kill_process_by_pid(pid: u32, force: bool) -> Result<(), String> {
    let service = ProcessService::new();
    service.kill_process(pid, force)
}

#[command]
#[allow(dead_code)] // Called via Tauri IPC, not directly from Rust
pub fn send_process_signal(pid: u32, signal: String) -> Result<(), String> {
    let service = ProcessService::new();
    service.send_signal(pid, &signal)
}
