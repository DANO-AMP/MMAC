import Foundation

enum StartupService {
    static func getStartupItems() -> [StartupItem] {
        var items: [StartupItem] = []
        let home = FileManager.default.homeDirectoryForCurrentUser
        let fm = FileManager.default

        // User LaunchAgents
        let userAgentsPath = home.appendingPathComponent("Library/LaunchAgents")
        if let contents = try? fm.contentsOfDirectory(at: userAgentsPath, includingPropertiesForKeys: nil) {
            for url in contents where url.pathExtension == "plist" {
                let name = url.deletingPathExtension().lastPathComponent
                let enabled = isLaunchAgentEnabled(url)
                items.append(StartupItem(name: name, path: url.path, kind: "LaunchAgent", enabled: enabled))
            }
        }

        // System LaunchAgents (filter com.apple.)
        let sysAgentsPath = URL(fileURLWithPath: "/Library/LaunchAgents")
        if let contents = try? fm.contentsOfDirectory(at: sysAgentsPath, includingPropertiesForKeys: nil) {
            for url in contents where url.pathExtension == "plist" {
                let name = url.deletingPathExtension().lastPathComponent
                guard !name.hasPrefix("com.apple.") else { continue }
                let enabled = isLaunchAgentEnabled(url)
                items.append(StartupItem(name: name, path: url.path, kind: "LaunchAgent", enabled: enabled))
            }
        }

        // Login items via osascript
        let loginResult = ShellHelper.shell("osascript -e 'tell application \"System Events\" to get the name of every login item'")
        if loginResult.exitCode == 0 && !loginResult.output.isEmpty {
            let loginNames = loginResult.output.components(separatedBy: ", ")
            for name in loginNames where !name.isEmpty {
                items.append(StartupItem(name: name.trimmingCharacters(in: .whitespaces), path: "", kind: "LoginItem", enabled: true))
            }
        }

        return items
    }

    private static func isLaunchAgentEnabled(_ url: URL) -> Bool {
        guard let data = try? Data(contentsOf: url),
              let plist = try? PropertyListSerialization.propertyList(from: data, format: nil) as? [String: Any] else {
            return true // default to enabled if we can't parse
        }
        if let disabled = plist["Disabled"] as? Bool {
            return !disabled
        }
        return true
    }

    static func toggleStartupItem(path: String, enable: Bool) -> Result<Void, ServiceError> {
        // Validate path
        let url = URL(fileURLWithPath: path)
        guard FileManager.default.fileExists(atPath: path),
              url.pathExtension == "plist" else {
            return .failure(ServiceError("Ruta no válida: \(path)"))
        }

        let action = enable ? "load" : "unload"
        let result = ShellHelper.run("/bin/launchctl", arguments: [action, path])
        if result.exitCode == 0 {
            return .success(())
        }
        return .failure(ServiceError("Error al \(action): \(result.error)"))
    }
}
