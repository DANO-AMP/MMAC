import Foundation

struct PortInfo: Identifiable {
    let id = UUID()
    let port: UInt16
    let pid: UInt32
    let processName: String
    let serviceType: String
    let proto: String
    let localAddress: String
    let workingDir: String?
    let command: String?
    let cpuUsage: Float
    let memoryMB: Float
}
