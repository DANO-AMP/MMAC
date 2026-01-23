use crate::services::duplicates::{DuplicateScanResult, DuplicateService};
use tauri::command;

#[command]
pub async fn scan_duplicates(path: String, min_size: u64) -> Result<DuplicateScanResult, String> {
    let service = DuplicateService::new();
    service.scan_duplicates(&path, min_size)
}
