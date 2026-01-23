use crate::services::network::{HostEntry, NetworkConnection, NetworkService};
use tauri::command;

#[command]
pub fn get_active_connections() -> Vec<NetworkConnection> {
    let service = NetworkService::new();
    service.get_active_connections()
}

#[command]
pub fn get_hosts() -> Vec<HostEntry> {
    let service = NetworkService::new();
    service.get_hosts()
}

#[command]
pub fn flush_dns() -> Result<String, String> {
    let service = NetworkService::new();
    service.flush_dns()
}
