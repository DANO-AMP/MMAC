use crate::services::orphaned::{OrphanedScanResult, OrphanedService};

#[tauri::command]
pub async fn scan_orphaned_files() -> Result<OrphanedScanResult, String> {
    let service = OrphanedService::new();
    service.scan_orphaned_files()
}

#[tauri::command]
pub async fn delete_orphaned_file(path: String) -> Result<u64, String> {
    let service = OrphanedService::new();
    service.delete_orphaned(&path)
}
