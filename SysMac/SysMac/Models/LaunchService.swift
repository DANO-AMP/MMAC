import Foundation

struct LaunchService: Identifiable {
    let id = UUID()
    let label: String
    let pid: UInt32?
    let status: String
    let kind: String
    let lastExitStatus: Int32?
}

struct ServicesResult {
    let userAgents: [LaunchService]
    let userDaemons: [LaunchService]
    let systemAgents: [LaunchService]
}
