use crate::services::uninstaller::{AppInfo, UninstallerService};
use tauri::command;

#[command]
pub async fn list_installed_apps() -> Result<Vec<AppInfo>, String> {
    let service = UninstallerService::new();
    service.list_apps().await.map_err(|e| e.to_string())
}

#[command]
pub async fn uninstall_app(bundle_id: String, include_remnants: bool) -> Result<(), String> {
    let service = UninstallerService::new();
    service
        .uninstall(&bundle_id, include_remnants)
        .await
        .map_err(|e| e.to_string())
}
