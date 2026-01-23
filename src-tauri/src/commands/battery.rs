use crate::services::battery::{BatteryInfo, BatteryService};
use tauri::command;

#[command]
pub fn get_battery_info() -> Option<BatteryInfo> {
    let service = BatteryService::new();
    service.get_battery_info()
}
