use crate::services::startup::{StartupItem, StartupService};
use tauri::command;

#[command]
pub fn get_startup_items() -> Vec<StartupItem> {
    let service = StartupService::new();
    service.get_startup_items()
}

#[command]
pub fn toggle_startup_item(path: String, enable: bool) -> Result<(), String> {
    let service = StartupService::new();
    service.toggle_startup_item(&path, enable)
}

#[command]
pub fn remove_login_item(name: String) -> Result<(), String> {
    let service = StartupService::new();
    service.remove_login_item(&name)
}
