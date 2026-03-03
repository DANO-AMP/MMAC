import Foundation

enum UninstallerService {
    static func listApps() -> [AppInfo] {
        let fm = FileManager.default
        let home = fm.homeDirectoryForCurrentUser
        var apps: [AppInfo] = []

        for appsDir in [URL(fileURLWithPath: "/Applications"), home.appendingPathComponent("Applications")] {
            guard let contents = try? fm.contentsOfDirectory(at: appsDir, includingPropertiesForKeys: nil) else { continue }
            for appURL in contents where appURL.pathExtension == "app" {
                if let info = parseAppInfo(appURL) {
                    apps.append(info)
                }
            }
        }

        apps.sort { $0.name.lowercased() < $1.name.lowercased() }
        return apps
    }

    private static func parseAppInfo(_ appURL: URL) -> AppInfo? {
        let plistURL = appURL.appendingPathComponent("Contents/Info.plist")
        let fm = FileManager.default

        guard fm.fileExists(atPath: plistURL.path),
              let data = try? Data(contentsOf: plistURL),
              let plist = try? PropertyListSerialization.propertyList(from: data, format: nil) as? [String: Any] else {
            // No Info.plist, still list it
            let name = appURL.deletingPathExtension().lastPathComponent
            let size = FileUtilities.directorySize(at: appURL)
            return AppInfo(name: name, bundleId: "", path: appURL.path, size: size, version: nil, remnants: [], remnantsSize: 0)
        }

        let name = (plist["CFBundleName"] as? String) ?? appURL.deletingPathExtension().lastPathComponent
        let bundleId = (plist["CFBundleIdentifier"] as? String) ?? ""
        let version = plist["CFBundleShortVersionString"] as? String
        let size = FileUtilities.directorySize(at: appURL)

        let remnants = findRemnants(bundleId: bundleId, appName: name)
        let remnantsSize = remnants.reduce(0) { $0 + $1.size }

        return AppInfo(name: name, bundleId: bundleId, path: appURL.path, size: size, version: version, remnants: remnants, remnantsSize: remnantsSize)
    }

    private static func findRemnants(bundleId: String, appName: String) -> [RemnantFile] {
        guard !bundleId.isEmpty else { return [] }

        let home = FileManager.default.homeDirectoryForCurrentUser
        let fm = FileManager.default

        let searchDirs = [
            ("Application Support", home.appendingPathComponent("Library/Application Support")),
            ("Caches", home.appendingPathComponent("Library/Caches")),
            ("Preferences", home.appendingPathComponent("Library/Preferences")),
            ("Containers", home.appendingPathComponent("Library/Containers")),
            ("Group Containers", home.appendingPathComponent("Library/Group Containers")),
            ("Saved Application State", home.appendingPathComponent("Library/Saved Application State")),
            ("Logs", home.appendingPathComponent("Library/Logs")),
            ("Cookies", home.appendingPathComponent("Library/Cookies")),
            ("WebKit", home.appendingPathComponent("Library/WebKit")),
            ("HTTPStorages", home.appendingPathComponent("Library/HTTPStorages")),
            ("LaunchAgents", home.appendingPathComponent("Library/LaunchAgents")),
            ("Application Scripts", home.appendingPathComponent("Library/Application Scripts")),
        ]

        let nameNoSpaces = appName.replacingOccurrences(of: " ", with: "")
        let nameDots = appName.replacingOccurrences(of: " ", with: ".")
        let patterns = [bundleId.lowercased(), nameNoSpaces.lowercased(), nameDots.lowercased()]

        var remnants: [RemnantFile] = []

        for (typeName, dir) in searchDirs {
            guard let contents = try? fm.contentsOfDirectory(at: dir, includingPropertiesForKeys: nil) else { continue }
            for item in contents {
                let lower = item.lastPathComponent.lowercased()
                if patterns.contains(where: { lower.contains($0) }) {
                    let size = FileUtilities.directorySize(at: item)
                    remnants.append(RemnantFile(path: item.path, size: size, remnantType: typeName))
                }
            }
        }

        return remnants
    }

    static func uninstallApp(path: String, remnantPaths: [String], moveToTrash: Bool) -> Result<UInt64, ServiceError> {
        let fm = FileManager.default
        var freed: UInt64 = 0

        switch PathValidator.validateForDeletion(path) {
        case .failure(let error):
            return .failure(error)
        case .success(let appURL):
            let appSize = FileUtilities.directorySize(at: appURL)
            do {
                if moveToTrash {
                    try fm.trashItem(at: appURL, resultingItemURL: nil)
                } else {
                    try fm.removeItem(at: appURL)
                }
                freed += appSize
            } catch {
                return .failure(ServiceError("Error al eliminar app: \(error.localizedDescription)"))
            }
        }

        for remnantPath in remnantPaths {
            guard case .success(let validatedURL) = PathValidator.validateForDeletion(remnantPath) else { continue }
            let size = FileUtilities.directorySize(at: validatedURL)
            do {
                if moveToTrash {
                    try fm.trashItem(at: validatedURL, resultingItemURL: nil)
                } else {
                    try fm.removeItem(at: validatedURL)
                }
                freed += size
            } catch { /* skip */ }
        }

        return .success(freed)
    }
}
