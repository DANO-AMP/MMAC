use crate::services::monitor::{MonitorService, SystemStats};
use tauri::command;

#[command]
pub fn get_system_stats() -> Result<SystemStats, String> {
    let service = MonitorService::new();
    Ok(service.get_stats())
}
