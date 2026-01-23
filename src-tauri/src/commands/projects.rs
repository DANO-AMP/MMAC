use crate::services::projects::{ProjectArtifact, ProjectsService};
use tauri::command;

#[command]
pub async fn scan_project_artifacts() -> Result<Vec<ProjectArtifact>, String> {
    let service = ProjectsService::new();
    service.scan().await.map_err(|e| e.to_string())
}

#[command]
pub fn delete_artifact(path: String) -> Result<(), String> {
    let expanded = shellexpand::tilde(&path).to_string();
    trash::delete(&expanded).map_err(|e| e.to_string())
}
