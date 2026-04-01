import Foundation

enum FirewallService {
    static func getFirewallStatus() -> FirewallStatus {
        let result = ShellHelper.run("/usr/bin/defaults", arguments: ["read", "/Library/Preferences/com.apple.alf", "globalstate"])
        let enabled = result.output.trimmingCharacters(in: .whitespacesAndNewlines) != "0"

        let stealthResult = ShellHelper.run("/usr/bin/defaults", arguments: ["read", "/Library/Preferences/com.apple.alf", "stealthenabled"])
        let stealth = stealthResult.output.trimmingCharacters(in: .whitespacesAndNewlines) == "1"

        let blockResult = ShellHelper.run("/usr/bin/defaults", arguments: ["read", "/Library/Preferences/com.apple.alf", "allowsignedenabled"])
        let blockAll = blockResult.output.trimmingCharacters(in: .whitespacesAndNewlines) == "0"

        return FirewallStatus(enabled: enabled, stealthMode: stealth, blockAllIncoming: blockAll)
    }

    static func getOutgoingConnections() -> [ProcessConnections] {
        let result = ShellHelper.run("/usr/sbin/lsof", arguments: ["-i", "-n", "-P"])
        guard result.exitCode == 0 else { return [] }

        var processMap: [UInt32: (name: String, conns: [OutgoingConnection])] = [:]

        for line in result.output.components(separatedBy: "\n").dropFirst() {
            let parts = line.split(separator: " ", omittingEmptySubsequences: true)
            guard parts.count >= 9 else { continue }

            let processName = String(parts[0])
            let pid = UInt32(parts[1]) ?? 0

            let nameField = String(parts.last ?? "")
            guard nameField.contains("->") else { continue }

            let connParts = nameField.components(separatedBy: "->")
            guard connParts.count == 2 else { continue }

            let (_, localPort) = parseAddr(connParts[0])
            let (remoteHost, remotePort) = parseAddr(connParts[1])

            let state: String
            if line.contains("ESTABLISHED") { state = "ESTABLISHED" }
            else if line.contains("CLOSE_WAIT") { state = "CLOSE_WAIT" }
            else if line.contains("TIME_WAIT") { state = "TIME_WAIT" }
            else { state = "CONNECTED" }

            let conn = OutgoingConnection(
                processName: processName,
                pid: pid,
                remoteHost: remoteHost,
                remotePort: remotePort,
                localPort: localPort,
                connectionState: state
            )

            processMap[pid, default: (processName, [])].conns.append(conn)
        }

        return processMap.map { (pid, data) in
            ProcessConnections(
                processName: data.name,
                pid: pid,
                connectionCount: UInt32(data.conns.count),
                connections: data.conns
            )
        }.sorted { $0.connectionCount > $1.connectionCount }
    }

    private static func parseAddr(_ addr: String) -> (String, UInt16) {
        if let colonIdx = addr.lastIndex(of: ":") {
            let host = String(addr[..<colonIdx])
            let port = UInt16(addr[addr.index(after: colonIdx)...]) ?? 0
            return (host, port)
        }
        return (addr, 0)
    }
}
