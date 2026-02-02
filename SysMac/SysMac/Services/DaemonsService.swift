import Foundation

enum DaemonsService {
    static func listServices() -> ServicesResult {
        var userAgents: [LaunchService] = []
        let userDaemons: [LaunchService] = []
        var systemAgents: [LaunchService] = []

        // User agents
        let userAgentResult = ShellHelper.shell("launchctl list 2>/dev/null")
        if userAgentResult.exitCode == 0 {
            for line in userAgentResult.output.components(separatedBy: "\n").dropFirst() {
                if let svc = parseLaunchctlLine(line, kind: "User Agent") {
                    if !svc.label.hasPrefix("com.apple.") {
                        userAgents.append(svc)
                    }
                }
            }
        }

        // System agents (requires gui/ domain)
        let sysAgentResult = ShellHelper.shell("launchctl print system/ 2>/dev/null | grep -E '^\\s+[0-9-]+' | head -100")
        if sysAgentResult.exitCode == 0 {
            for line in sysAgentResult.output.components(separatedBy: "\n") {
                if let svc = parsePrintLine(line, kind: "System Agent") {
                    if !svc.label.hasPrefix("com.apple.") {
                        systemAgents.append(svc)
                    }
                }
            }
        }

        return ServicesResult(userAgents: userAgents, userDaemons: userDaemons, systemAgents: systemAgents)
    }

    static func parseLaunchctlLine(_ line: String, kind: String) -> LaunchService? {
        // Format: PID\tStatus\tLabel
        let parts = line.split(separator: "\t", omittingEmptySubsequences: false)
        guard parts.count >= 3 else { return nil }

        let pidStr = parts[0].trimmingCharacters(in: .whitespaces)
        let statusStr = parts[1].trimmingCharacters(in: .whitespaces)
        let label = parts[2].trimmingCharacters(in: .whitespaces)

        guard !label.isEmpty else { return nil }

        let pid = UInt32(pidStr)
        let exitStatus = Int32(statusStr)

        let status: String
        if pid != nil {
            status = "running"
        } else if exitStatus == 0 || exitStatus == nil {
            status = "stopped"
        } else {
            status = "error"
        }

        return LaunchService(label: label, pid: pid, status: status, kind: kind, lastExitStatus: exitStatus)
    }

    private static func parsePrintLine(_ line: String, kind: String) -> LaunchService? {
        let trimmed = line.trimmingCharacters(in: .whitespaces)
        let parts = trimmed.split(separator: " ", maxSplits: 1, omittingEmptySubsequences: true)
        guard parts.count >= 2 else { return nil }

        let statusStr = String(parts[0])
        let label = String(parts[1])

        let pid: UInt32? = UInt32(statusStr)
        let exitStatus = Int32(statusStr)

        let status: String
        if pid != nil && statusStr != "0" && statusStr != "-" {
            status = "running"
        } else {
            status = "stopped"
        }

        return LaunchService(label: label, pid: pid, status: status, kind: kind, lastExitStatus: exitStatus)
    }

    static func startService(label: String) -> Result<Void, ServiceError> {
        let result = ShellHelper.shell("launchctl start \(label)")
        return result.exitCode == 0 ? .success(()) : .failure(ServiceError("Error al iniciar: \(result.error)"))
    }

    static func stopService(label: String) -> Result<Void, ServiceError> {
        let result = ShellHelper.shell("launchctl stop \(label)")
        return result.exitCode == 0 ? .success(()) : .failure(ServiceError("Error al detener: \(result.error)"))
    }
}
