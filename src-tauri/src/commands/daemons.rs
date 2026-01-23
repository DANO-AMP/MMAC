use crate::services::daemons::{DaemonsService, ServicesResult};

#[tauri::command]
pub async fn list_launch_services() -> Result<ServicesResult, String> {
    let service = DaemonsService::new();
    service.list_services()
}

#[tauri::command]
pub async fn start_launch_service(label: String) -> Result<String, String> {
    let service = DaemonsService::new();
    service.start_service(&label)
}

#[tauri::command]
pub async fn stop_launch_service(label: String) -> Result<String, String> {
    let service = DaemonsService::new();
    service.stop_service(&label)
}

#[tauri::command]
pub async fn get_launch_service_info(label: String) -> Result<String, String> {
    let service = DaemonsService::new();
    service.get_service_info(&label)
}
