use crate::services::monitor::{MonitorService, SystemStats};
use tauri::{command, State};

#[command]
pub async fn get_system_stats(monitor: State<'_, MonitorService>) -> Result<SystemStats, String> {
    Ok(monitor.get_stats().await)
}
