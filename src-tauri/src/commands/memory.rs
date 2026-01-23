use crate::services::memory::{MemoryInfo, MemoryService};
use tauri::command;

#[command]
pub fn get_memory_info() -> MemoryInfo {
    let service = MemoryService::new();
    service.get_memory_info()
}

#[command]
pub fn purge_memory() -> Result<String, String> {
    let service = MemoryService::new();
    service.purge_memory()
}
