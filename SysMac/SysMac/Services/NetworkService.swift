import Foundation

enum NetworkService {
    static func getConnections() -> [NetworkConnection] {
        var connections: [NetworkConnection] = []

        // Batch all process lookups into a single /bin/ps call instead of per-PID forks
        var processMap: [UInt32: String] = [:]
        let psResult = ShellHelper.run("/bin/ps", arguments: ["-axo", "pid=,comm="])
        if psResult.exitCode == 0 {
            for line in psResult.output.components(separatedBy: "\n") {
                let parts = line.split(separator: " ", maxSplits: 1, omittingEmptySubsequences: true)
                guard parts.count == 2, let pid = UInt32(parts[0]) else { continue }
                let name = parts[1].split(separator: "/").last.map(String.init) ?? String(parts[1])
                processMap[pid] = name
            }
        }

        for proto in ["tcp", "udp"] {
            let result = ShellHelper.run("/usr/sbin/netstat", arguments: ["-anvp", proto])
            guard result.exitCode == 0 else { continue }

            for line in result.output.components(separatedBy: "\n").dropFirst(2) {
                if let conn = parseNetstatLine(line, proto: proto, processMap: processMap) {
                    connections.append(conn)
                }
            }
        }

        return connections
    }

    private static func parseNetstatLine(_ line: String, proto: String, processMap: [UInt32: String]) -> NetworkConnection? {
        let parts = line.split(separator: " ", omittingEmptySubsequences: true)
        guard parts.count >= 9 else { return nil }

        let localFull = String(parts[3])
        let remoteFull = String(parts[4])
        let state = proto == "tcp" ? String(parts[5]) : ""
        let pid = UInt32(parts.last ?? "0") ?? 0

        let (localAddr, localPort) = parseAddress(localFull)
        let (remoteAddr, remotePort) = parseAddress(remoteFull)

        return NetworkConnection(
            proto: proto.uppercased(),
            localAddress: localAddr,
            localPort: localPort,
            remoteAddress: remoteAddr,
            remotePort: remotePort,
            state: state,
            pid: pid,
            processName: pid > 0 ? processMap[pid] ?? "" : ""
        )
    }

    static func parseAddress(_ addr: String) -> (String, UInt16) {
        // Formats: 192.168.1.1.443, ::1.8888, *.*
        if addr == "*.*" { return ("*", 0) }

        if let lastDot = addr.lastIndex(of: ".") {
            let portStr = addr[addr.index(after: lastDot)...]
            let host = String(addr[..<lastDot])
            return (host, UInt16(portStr) ?? 0)
        }

        return (addr, 0)
    }

    static func getHostsFile() -> [HostEntry] {
        guard let contents = try? String(contentsOfFile: "/etc/hosts", encoding: .utf8) else { return [] }
        var entries: [HostEntry] = []
        for line in contents.components(separatedBy: "\n") {
            let trimmed = line.trimmingCharacters(in: .whitespaces)
            if trimmed.isEmpty || trimmed.hasPrefix("#") { continue }
            let parts = trimmed.split(separator: " ", maxSplits: 2, omittingEmptySubsequences: true)
            guard parts.count >= 2 else { continue }
            let comment = parts.count >= 3 ? String(parts[2]).trimmingCharacters(in: .whitespaces) : nil
            entries.append(HostEntry(ip: String(parts[0]), hostname: String(parts[1]), comment: comment?.hasPrefix("#") == true ? comment : nil))
        }
        return entries
    }

    static func flushDNS() -> Result<Void, ServiceError> {
        let result = ShellHelper.run("/usr/bin/dscacheutil", arguments: ["-flushcache"])
        if result.exitCode == 0 {
            return .success(())
        }
        return .failure(ServiceError("Error al limpiar DNS: \(result.error)"))
    }
}
