import Foundation

struct StartupItem: Identifiable {
    let id = UUID()
    let name: String
    let path: String
    let kind: String  // "LaunchAgent", "LaunchDaemon", "LoginItem"
    let enabled: Bool
}
