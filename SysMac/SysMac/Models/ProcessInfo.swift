import Foundation

struct ProcessItem: Identifiable {
    var id: UInt32 { pid }
    let pid: UInt32
    let ppid: UInt32
    let name: String
    let cpuUsage: Float
    let memoryMB: Float
    let memoryPercent: Float
    let user: String
    let state: String
    let threads: UInt32
    let command: String
}
