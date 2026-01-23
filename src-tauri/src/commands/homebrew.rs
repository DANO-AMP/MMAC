use crate::services::homebrew::{BrewPackage, HomebrewInfo, HomebrewService};

#[tauri::command]
pub fn get_homebrew_info() -> Result<HomebrewInfo, String> {
    let service = HomebrewService::new();
    service.check_homebrew()
}

#[tauri::command]
pub fn list_brew_packages() -> Result<Vec<BrewPackage>, String> {
    let service = HomebrewService::new();
    service.list_packages()
}

#[tauri::command]
pub fn get_outdated_packages() -> Result<Vec<BrewPackage>, String> {
    let service = HomebrewService::new();
    service.get_outdated()
}

#[tauri::command]
pub fn upgrade_brew_package(name: String, is_cask: bool) -> Result<String, String> {
    let service = HomebrewService::new();
    service.upgrade_package(&name, is_cask)
}

#[tauri::command]
pub fn upgrade_all_packages() -> Result<String, String> {
    let service = HomebrewService::new();
    service.upgrade_all()
}

#[tauri::command]
pub fn uninstall_brew_package(name: String, is_cask: bool) -> Result<String, String> {
    let service = HomebrewService::new();
    service.uninstall_package(&name, is_cask)
}

#[tauri::command]
pub fn brew_cleanup() -> Result<String, String> {
    let service = HomebrewService::new();
    service.cleanup()
}
