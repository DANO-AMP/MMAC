import Foundation

enum PortScannerService {
    static func scan() -> [PortInfo] {
        let result = ShellHelper.run("/usr/sbin/lsof", arguments: ["-i", "-P", "-n"])
        guard result.exitCode == 0 else { return [] }

        var portMap: [UInt16: PortInfo] = [:]

        for line in result.output.components(separatedBy: "\n").dropFirst() {
            guard line.contains("LISTEN") || line.contains("(LISTEN)") else { continue }

            let parts = line.split(separator: " ", omittingEmptySubsequences: true)
            guard parts.count >= 9 else { continue }

            let processName = String(parts[0])
            let pid = UInt32(parts[1]) ?? 0

            // Parse the address:port from the NAME column (last)
            let nameField = String(parts.last ?? "")
            guard let colonIdx = nameField.lastIndex(of: ":") else { continue }
            let portStr = nameField[nameField.index(after: colonIdx)...]
            guard let port = UInt16(portStr) else { continue }
            let localAddr = String(nameField[..<colonIdx])

            let proto = String(parts[7]).uppercased().contains("UDP") ? "UDP" : "TCP"

            if portMap[port] == nil {
                let serviceType = detectServiceType(processName: processName, port: port)
                let (cpu, mem) = getProcessStats(pid)

                portMap[port] = PortInfo(
                    port: port,
                    pid: pid,
                    processName: processName,
                    serviceType: serviceType,
                    proto: proto,
                    localAddress: localAddr,
                    workingDir: getWorkingDir(pid),
                    command: getCommand(pid),
                    cpuUsage: cpu,
                    memoryMB: mem
                )
            }
        }

        return Array(portMap.values).sorted { $0.port < $1.port }
    }

    static func detectServiceType(processName: String, port: UInt16) -> String {
        let lower = processName.lowercased()
        if lower.contains("node") { return "Node.js" }
        if lower.contains("python") { return "Python" }
        if lower.contains("java") { return "Java" }
        if lower.contains("ruby") { return "Ruby" }
        if lower.contains("php") { return "PHP" }
        if lower.contains("postgres") || lower.contains("psql") { return "PostgreSQL" }
        if lower.contains("mysql") || lower.contains("mysqld") { return "MySQL" }
        if lower.contains("redis") { return "Redis" }
        if lower.contains("mongo") { return "MongoDB" }
        if lower.contains("nginx") { return "Nginx" }
        if lower.contains("apache") || lower.contains("httpd") { return "Apache" }
        if lower.contains("docker") { return "Docker" }

        switch port {
        case 80, 8080: return "HTTP"
        case 443: return "HTTPS"
        case 22: return "SSH"
        case 3000: return "Dev Server"
        case 5432: return "PostgreSQL"
        case 3306: return "MySQL"
        case 6379: return "Redis"
        case 27017: return "MongoDB"
        default: return "Other"
        }
    }

    private static func getProcessStats(_ pid: UInt32) -> (Float, Float) {
        let result = ShellHelper.run("/bin/ps", arguments: ["-p", "\(pid)", "-o", "%cpu,rss="], environment: ["LC_ALL": "C"])
        let parts = result.output.split(separator: " ", omittingEmptySubsequences: true)
        guard parts.count >= 2 else { return (0, 0) }
        let cpu = Float(parts[0]) ?? 0
        let rssKb = Float(parts[1]) ?? 0
        return (cpu, rssKb / 1024.0)
    }

    private static func getWorkingDir(_ pid: UInt32) -> String? {
        let result = ShellHelper.run("/usr/sbin/lsof", arguments: ["-p", "\(pid)", "-d", "cwd", "-Fn"])
        for line in result.output.components(separatedBy: "\n") {
            if line.hasPrefix("n") && !line.hasPrefix("n/") == false {
                return String(line.dropFirst())
            }
        }
        return nil
    }

    private static func getCommand(_ pid: UInt32) -> String? {
        let result = ShellHelper.run("/bin/ps", arguments: ["-p", "\(pid)", "-o", "args="])
        let cmd = result.output.trimmingCharacters(in: .whitespacesAndNewlines)
        return cmd.isEmpty ? nil : cmd
    }
}
