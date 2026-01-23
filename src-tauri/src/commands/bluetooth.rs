use crate::services::bluetooth::{BluetoothInfo, BluetoothService};

#[tauri::command]
pub fn get_bluetooth_info() -> Result<BluetoothInfo, String> {
    let service = BluetoothService::new();
    service.get_bluetooth_info()
}
