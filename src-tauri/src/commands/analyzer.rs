use crate::services::analyzer::{AnalyzerService, DiskItem};
use std::process::Command;
use tauri::command;

#[command]
pub async fn analyze_path(path: String) -> Result<Vec<DiskItem>, String> {
    let service = AnalyzerService::new();
    service.analyze(&path).await.map_err(|e| e.to_string())
}

#[command]
pub fn reveal_in_finder(path: String) -> Result<(), String> {
    let expanded = shellexpand::tilde(&path).to_string();

    Command::new("open")
        .args(["-R", &expanded])
        .output()
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[command]
pub fn move_to_trash(path: String) -> Result<(), String> {
    let expanded = shellexpand::tilde(&path).to_string();
    trash::delete(&expanded).map_err(|e| e.to_string())
}
