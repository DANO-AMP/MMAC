import Foundation

enum PortScannerService {
    static func scan() -> [PortInfo] {
        let result = ShellHelper.run("/usr/sbin/lsof", arguments: ["-i", "-P", "-n"])
        guard result.exitCode == 0 else { return [] }

        var portMap: [UInt16: PortInfo] = [:]
        var allPids: Set<UInt32> = []

        // First pass: collect all listening ports with placeholder details
        for line in result.output.components(separatedBy: "\n").dropFirst() {
            guard line.contains("LISTEN") || line.contains("(LISTEN)") else { continue }

            let parts = line.split(separator: " ", omittingEmptySubsequences: true)
            guard parts.count >= 9 else { continue }

            let processName = String(parts[0])
            let pid = UInt32(parts[1]) ?? 0

            // Parse the address:port from the NAME column (index 8, not last - last is "(LISTEN)")
            let nameField = String(parts[8])
            guard let colonIdx = nameField.lastIndex(of: ":") else { continue }
            let portStr = nameField[nameField.index(after: colonIdx)...]
            guard let port = UInt16(portStr) else { continue }
            let localAddr = String(nameField[..<colonIdx])

            let proto = String(parts[7]).uppercased().contains("UDP") ? "UDP" : "TCP"

            if portMap[port] == nil {
                let serviceType = detectServiceType(processName: processName, port: port)

                portMap[port] = PortInfo(
                    port: port,
                    pid: pid,
                    processName: processName,
                    serviceType: serviceType,
                    proto: proto,
                    localAddress: localAddr,
                    workingDir: nil,
                    command: nil,
                    cpuUsage: 0,
                    memoryMB: 0
                )

                if pid != 0 { allPids.insert(pid) }
            }
        }

        // Batch-fetch details for all PIDs in just 3 subprocess calls
        guard !allPids.isEmpty else {
            return Array(portMap.values).sorted { $0.port < $1.port }
        }

        let pidList = allPids.map(String.init).joined(separator: ",")
        let statsMap = batchProcessStats(pidList)
        let dirMap = batchWorkingDirs(pidList)
        let cmdMap = batchCommands(pidList)

        // Merge batch results back into port entries
        for port in portMap.keys {
            let existing = portMap[port]!
            if existing.pid == 0 { continue }
            let (cpu, mem) = statsMap[existing.pid] ?? (0, 0)
            portMap[port] = PortInfo(
                port: existing.port,
                pid: existing.pid,
                processName: existing.processName,
                serviceType: existing.serviceType,
                proto: existing.proto,
                localAddress: existing.localAddress,
                workingDir: dirMap[existing.pid],
                command: cmdMap[existing.pid],
                cpuUsage: cpu,
                memoryMB: mem
            )
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

    private static func batchProcessStats(_ pidList: String) -> [UInt32: (Float, Float)] {
        let result = ShellHelper.run("/bin/ps", arguments: ["-p", pidList, "-o", "pid=,%cpu,rss="], environment: ["LC_ALL": "C"])
        var map: [UInt32: (Float, Float)] = [:]
        for line in result.output.components(separatedBy: "\n") {
            let parts = line.split(separator: " ", omittingEmptySubsequences: true)
            guard parts.count >= 3, let pid = UInt32(parts[0]) else { continue }
            let cpu = Float(parts[1]) ?? 0
            let rssKb = Float(parts[2]) ?? 0
            map[pid] = (cpu, rssKb / 1024.0)
        }
        return map
    }

    private static func batchWorkingDirs(_ pidList: String) -> [UInt32: String] {
        let result = ShellHelper.run("/usr/sbin/lsof", arguments: ["-p", pidList, "-d", "cwd", "-Fn"])
        var map: [UInt32: String] = [:]
        var currentPid: UInt32?
        for line in result.output.components(separatedBy: "\n") {
            if line.hasPrefix("p") {
                currentPid = UInt32(line.dropFirst())
            } else if line.hasPrefix("n/"), let pid = currentPid {
                map[pid] = String(line.dropFirst())
                currentPid = nil
            }
        }
        return map
    }

    private static func batchCommands(_ pidList: String) -> [UInt32: String] {
        let result = ShellHelper.run("/bin/ps", arguments: ["-p", pidList, "-o", "pid=,args="], environment: ["LC_ALL": "C"])
        var map: [UInt32: String] = [:]
        for line in result.output.components(separatedBy: "\n") {
            let parts = line.split(separator: " ", maxSplits: 1, omittingEmptySubsequences: true)
            guard parts.count == 2, let pid = UInt32(parts[0]) else { continue }
            let cmd = String(parts[1]).trimmingCharacters(in: .whitespacesAndNewlines)
            if !cmd.isEmpty { map[pid] = cmd }
        }
        return map
    }
}
