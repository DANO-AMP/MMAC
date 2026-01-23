use crate::services::firewall::{FirewallService, FirewallStatus, ProcessConnections};

#[tauri::command]
pub async fn get_outgoing_connections() -> Result<Vec<ProcessConnections>, String> {
    let service = FirewallService::new();
    service.get_outgoing_connections()
}

#[tauri::command]
pub async fn get_firewall_status() -> Result<FirewallStatus, String> {
    let service = FirewallService::new();
    service.get_firewall_status()
}

#[tauri::command]
pub async fn resolve_hostname(ip: String) -> String {
    let service = FirewallService::new();
    service.resolve_hostname(&ip)
}
