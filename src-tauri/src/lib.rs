mod commands;
mod services;

use commands::{
    analyzer::*, battery::*, cleaning::*, duplicates::*, largefiles::*, memory::*, monitor::*,
    network::*, ports::*, processes::*, projects::*, startup::*, uninstaller::*,
};
use commands::ports::ScannedPidsState;
use services::monitor::MonitorService;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .manage(MonitorService::new())
        .manage(ScannedPidsState::default())
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
            // Startup
            get_startup_items,
            toggle_startup_item,
            remove_login_item,
            // Duplicates
            scan_duplicates,
            // Large Files
            find_large_files,
            // Memory
            get_memory_info,
            purge_memory,
            // Battery
            get_battery_info,
            // Network
            get_active_connections,
            get_hosts,
            flush_dns,
            // Processes
            get_all_processes,
            kill_process_by_pid,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
