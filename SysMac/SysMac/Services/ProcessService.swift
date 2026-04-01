import Foundation
import Darwin

enum ProcessService {
    static func getAllProcesses() -> [ProcessItem] {
        let result = ShellHelper.run("/bin/ps", arguments: ["-axo", "pid,ppid,%cpu,rss,%mem,user,state,wq,args=", "-r"], environment: ["LC_ALL": "C"])
        guard result.exitCode == 0 else { return [] }

        var processes: [ProcessItem] = []
        let lines = result.output.components(separatedBy: "\n")

        for line in lines.dropFirst() {
            if let proc = parsePsLine(line) {
                processes.append(proc)
            }
        }

        return processes
    }

    static func parsePsLine(_ line: String) -> ProcessItem? {
        let scanner = Scanner(string: line)
        scanner.charactersToBeSkipped = CharacterSet.whitespaces

        guard let pidStr = scanner.scanUpToString(" "),
              let pid = UInt32(pidStr),
              let ppidStr = scanner.scanUpToString(" "),
              let ppid = UInt32(ppidStr),
              let cpuStr = scanner.scanUpToString(" "),
              let cpuUsage = Float(cpuStr),
              let rssStr = scanner.scanUpToString(" "),
              let rssKb = Float(rssStr),
              let memStr = scanner.scanUpToString(" "),
              let memPercent = Float(memStr),
              let user = scanner.scanUpToString(" "),
              let stateStr = scanner.scanUpToString(" "),
              let threadsStr = scanner.scanUpToString(" "),
              let threads = UInt32(threadsStr) else { return nil }

        // Everything remaining is the full command
        let command = scanner.remainingString?.trimmingCharacters(in: .whitespaces) ?? ""
        let name = command.split(separator: "/").last.map(String.init) ?? command

        return ProcessItem(
            pid: pid,
            ppid: ppid,
            name: name,
            cpuUsage: cpuUsage,
            memoryMB: rssKb / 1024.0,
            memoryPercent: memPercent,
            user: user,
            state: parseState(stateStr),
            threads: threads,
            command: command
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
