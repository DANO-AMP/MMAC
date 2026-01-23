mod commands;
mod services;

use commands::{
    analyzer::*, cleaning::*, monitor::*, ports::*, projects::*, uninstaller::*,
};
use services::monitor::MonitorService;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(MonitorService::new())
        .invoke_handler(tauri::generate_handler![
            // Cleaning
            scan_system,
            clean_category,
            // Uninstaller
            list_installed_apps,
            uninstall_app,
            // Analyzer
            analyze_path,
            reveal_in_finder,
            move_to_trash,
            // Monitor
            get_system_stats,
            // Ports
            scan_ports,
            kill_process,
            // Projects
            scan_project_artifacts,
            delete_artifact,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
