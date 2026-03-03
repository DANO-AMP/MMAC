import Foundation

struct SystemStats {
    var cpuUsage: Float = 0
    var memoryUsed: UInt64 = 0
    var memoryTotal: UInt64 = 0
    var diskUsed: UInt64 = 0
    var diskTotal: UInt64 = 0
    var networkRx: UInt64 = 0  // bytes per second
    var networkTx: UInt64 = 0  // bytes per second
    var cpuTemp: Float = 0
    var fanSpeed: UInt32? = nil  // RPM
    var diskReadSpeed: UInt64 = 0   // bytes per second
    var diskWriteSpeed: UInt64 = 0  // bytes per second
    var gpuName: String? = nil
    var gpuVendor: String? = nil
}