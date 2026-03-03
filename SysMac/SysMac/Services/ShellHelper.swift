import Foundation

enum ShellHelper {
    /// Run a shell command and return (stdout, stderr, exitCode)
    @discardableResult
    static func run(_ command: String, arguments: [String] = [], environment: [String: String]? = nil, timeout: TimeInterval = 10) -> (output: String, error: String, exitCode: Int32) {
        let process = Process()
        process.executableURL = URL(fileURLWithPath: command)
        process.arguments = arguments

        if let env = environment {
            var processEnv = ProcessInfo.processInfo.environment
            for (key, value) in env {
                processEnv[key] = value
            }
            process.environment = processEnv
        }

        let outputPipe = Pipe()
        let errorPipe = Pipe()
        process.standardOutput = outputPipe
        process.standardError = errorPipe

        do {
            try process.run()
        } catch {
            return ("", "Failed to run \(command): \(error.localizedDescription)", -1)
        }

        var timedOut = false
        let timeoutWork = DispatchWorkItem {
            timedOut = true
            process.terminate()
        }
        DispatchQueue.global().asyncAfter(deadline: .now() + timeout, execute: timeoutWork)

        // Read pipes BEFORE waitUntilExit to avoid deadlock when output exceeds pipe buffer
        let outputData = outputPipe.fileHandleForReading.readDataToEndOfFile()
        let errorData = errorPipe.fileHandleForReading.readDataToEndOfFile()
        process.waitUntilExit()
        timeoutWork.cancel()

        if timedOut {
            return ("", "Command timed out after \(Int(timeout))s: \(command)", -1)
        }

        let output = String(data: outputData, encoding: .utf8)?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        let errorOutput = String(data: errorData, encoding: .utf8)?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""

        return (output, errorOutput, process.terminationStatus)
    }

    /// Run a command using /bin/sh -c for shell features (pipes, redirects)
    @discardableResult
    static func shell(_ command: String) -> (output: String, error: String, exitCode: Int32) {
        run("/bin/sh", arguments: ["-c", command])
    }

    /// Find the full path of a command
    static func which(_ command: String) -> String? {
        let result = run("/usr/bin/which", arguments: [command])
        return result.exitCode == 0 && !result.output.isEmpty ? result.output : nil
    }

    /// Get the brew path (handles both Intel and Apple Silicon)
    static var brewPath: String? {
        if FileManager.default.fileExists(atPath: "/opt/homebrew/bin/brew") {
            return "/opt/homebrew/bin/brew"
        } else if FileManager.default.fileExists(atPath: "/usr/local/bin/brew") {
            return "/usr/local/bin/brew"
        }
        return nil
    }
}
