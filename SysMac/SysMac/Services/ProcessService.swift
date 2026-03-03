import Foundation
import Darwin

enum ProcessService {
    static func getAllProcesses() -> [ProcessItem] {
        let result = ShellHelper.run("/bin/ps", arguments: ["-axo", "pid,ppid,%cpu,rss,%mem,user,state,wq,comm", "-r"], environment: ["LC_ALL": "C"])
        guard result.exitCode == 0 else { return [] }

        // Batch-fetch all full commands in a single subprocess
        let fullCommands = getAllFullCommands()

        var processes: [ProcessItem] = []
        let lines = result.output.components(separatedBy: "\n")

        for line in lines.dropFirst() {
            if var proc = parsePsLine(line) {
                if let cmd = fullCommands[proc.pid] {
                    proc = ProcessItem(pid: proc.pid, ppid: proc.ppid, name: proc.name, cpuUsage: proc.cpuUsage, memoryMB: proc.memoryMB, memoryPercent: proc.memoryPercent, user: proc.user, state: proc.state, threads: proc.threads, command: cmd)
                }
                processes.append(proc)
            }
        }

        return processes
    }

    /// Fetch all process full commands in a single subprocess call
    private static func getAllFullCommands() -> [UInt32: String] {
        let result = ShellHelper.run("/bin/ps", arguments: ["-axo", "pid=,args="], environment: ["LC_ALL": "C"])
        guard result.exitCode == 0 else { return [:] }

        var dict: [UInt32: String] = [:]
        for line in result.output.components(separatedBy: "\n") {
            let trimmed = line.trimmingCharacters(in: .whitespaces)
            guard !trimmed.isEmpty else { continue }
            // Format: "  PID ARGS..." - split at first space after PID
            let parts = trimmed.split(separator: " ", maxSplits: 1, omittingEmptySubsequences: true)
            guard parts.count == 2, let pid = UInt32(parts[0]) else { continue }
            dict[pid] = String(parts[1])
        }
        return dict
    }

    static func parsePsLine(_ line: String) -> ProcessItem? {
        let parts = line.split(separator: " ", maxSplits: 8, omittingEmptySubsequences: true)
        guard parts.count >= 9 else { return nil }

        guard let pid = UInt32(parts[0]) else { return nil }
        let ppid = UInt32(parts[1]) ?? 0
        let cpuUsage = Float(parts[2]) ?? 0
        let rssKb = Float(parts[3]) ?? 0
        let memPercent = Float(parts[4]) ?? 0
        let user = String(parts[5])
        let state = parseState(String(parts[6]))
        let threads = UInt32(parts[7]) ?? 1
        let fullCommand = String(parts[8])
        let name = fullCommand.split(separator: "/").last.map(String.init) ?? fullCommand

        return ProcessItem(
            pid: pid,
            ppid: ppid,
            name: name,
            cpuUsage: cpuUsage,
            memoryMB: rssKb / 1024.0,
            memoryPercent: memPercent,
            user: user,
            state: state,
            threads: threads,
            command: fullCommand
        )
    }

    static func parseState(_ state: String) -> String {
        guard let first = state.first else { return "Desconocido" }
        switch first {
        case "R": return "Ejecutando"
        case "S": return "Suspendido"
        case "I": return "Inactivo"
        case "U": return "Espera"
        case "Z": return "Zombie"
        case "T": return "Detenido"
        default: return "Desconocido"
        }
    }

    static func killProcess(pid: UInt32, force: Bool) -> Result<Void, ServiceError> {
        sendSignal(pid: pid, signal: force ? "SIGKILL" : "SIGTERM")
    }

    static func sendSignal(pid: UInt32, signal: String) -> Result<Void, ServiceError> {
        guard pid >= 100 else {
            return .failure(ServiceError("No se puede enviar señal al proceso \(pid): es un proceso del sistema"))
        }

        let flag: String
        switch signal.uppercased() {
        case "SIGTERM", "TERM", "15": flag = "-15"
        case "SIGKILL", "KILL", "9": flag = "-9"
        case "SIGSTOP", "STOP", "17": flag = "-STOP"
        case "SIGCONT", "CONT", "19": flag = "-CONT"
        case "SIGHUP", "HUP", "1": flag = "-1"
        case "SIGINT", "INT", "2": flag = "-2"
        default: return .failure(ServiceError("Señal desconocida: \(signal)"))
        }

        let result = ShellHelper.run("/bin/kill", arguments: [flag, "\(pid)"])
        if result.exitCode == 0 {
            return .success(())
        } else {
            return .failure(ServiceError("Error al enviar señal: \(result.error)"))
        }
    }
}
