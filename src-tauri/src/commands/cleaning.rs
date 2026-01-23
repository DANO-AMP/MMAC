use crate::services::cleaner::{CleaningService, ScanResult};
use tauri::command;

#[command]
pub async fn scan_system() -> Result<Vec<ScanResult>, String> {
    let service = CleaningService::new();
    service.scan_all().await.map_err(|e| e.to_string())
}

#[command]
pub async fn clean_category(category: String) -> Result<u64, String> {
    let service = CleaningService::new();
    service
        .clean_category(&category)
        .await
        .map_err(|e| e.to_string())
}
