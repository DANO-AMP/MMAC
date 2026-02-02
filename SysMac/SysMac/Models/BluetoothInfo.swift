import Foundation

struct BluetoothDevice: Identifiable {
    let id = UUID()
    let name: String
    let address: String
    let deviceType: String
    let batteryPercent: UInt8?
    let isConnected: Bool
    let isPaired: Bool
    let vendor: String?
}

struct BluetoothInfo {
    let enabled: Bool
    let discoverable: Bool
    let address: String?
    let devices: [BluetoothDevice]
}
