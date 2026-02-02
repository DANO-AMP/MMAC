use crate::services::analyzer::{AnalyzerService, DiskItem};
use std::process::Command;
use std::path::Path;
use tauri::command;

#[command]
pub async fn analyze_path(path: String) -> Result<Vec<DiskItem>, String> {
    let service = AnalyzerService::new();
    service.analyze(&path).await.map_err(|e| e.to_string())
}

#[command]
pub fn reveal_in_finder(path: String) -> Result<(), String> {
    let expanded = shellexpand::tilde(&path).to_string();

    // Validate path to prevent directory traversal attacks
    if expanded.contains("..") {
        return Err("Invalid path: contains '..'".to_string());
    }

    // Verify path exists
    let path_obj = Path::new(&expanded);
    if !path_obj.exists() {
        return Err("Path does not exist".to_string());
    }

    // Get canonical path to ensure it's a valid absolute path
    let canonical = path_obj.canonicalize()
        .map_err(|e| format!("Failed to resolve path: {}", e))?;

    Command::new("open")
        .args(["-R", canonical.to_str().ok_or("Invalid path encoding")?])
        .output()
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[command]
pub fn move_to_trash(path: String) -> Result<(), String> {
    let expanded = shellexpand::tilde(&path).to_string();
    trash::delete(&expanded).map_err(|e| e.to_string())
}
