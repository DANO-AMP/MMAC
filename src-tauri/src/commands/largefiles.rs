use crate::services::largefiles::{LargeFile, LargeFilesService};
use tauri::command;

#[command]
pub async fn find_large_files(path: String, min_size: u64, limit: usize) -> Vec<LargeFile> {
    let service = LargeFilesService::new();
    service.find_large_files(&path, min_size, limit)
}
