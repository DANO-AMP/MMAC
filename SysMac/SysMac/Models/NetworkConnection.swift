import Foundation

struct NetworkConnection: Identifiable {
    let id = UUID()
    let proto: String
    let localAddress: String
    let localPort: UInt16
    let remoteAddress: String
    let remotePort: UInt16
    let state: String
    let pid: UInt32
    let processName: String
}

struct HostEntry: Identifiable {
    let id = UUID()
    let ip: String
    let hostname: String
    let comment: String?
}
