import Foundation

struct FirewallStatus {
    let enabled: Bool
    let stealthMode: Bool
    let blockAllIncoming: Bool
}

struct OutgoingConnection: Identifiable {
    let id = UUID()
    let processName: String
    let pid: UInt32
    let remoteHost: String
    let remotePort: UInt16
    let localPort: UInt16
    let connectionState: String
}

struct ProcessConnections: Identifiable {
    let id = UUID()
    let processName: String
    let pid: UInt32
    let connectionCount: UInt32
    let connections: [OutgoingConnection]
}
