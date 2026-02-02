import Foundation

struct BatteryInfo {
    let isPresent: Bool
    let isCharging: Bool
    let isFullyCharged: Bool
    let chargePercent: Float
    let cycleCount: UInt32
    let maxCapacity: UInt32
    let designCapacity: UInt32
    let healthPercent: Float
    let temperature: Float
    let voltage: Float
    let amperage: Int32
    let timeRemaining: String?
    let powerSource: String
    let condition: String
}
