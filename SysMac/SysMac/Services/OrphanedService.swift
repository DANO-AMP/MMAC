import Foundation

enum OrphanedService {
    static func scanOrphanedFiles() -> OrphanedScanResult {
        let home = FileManager.default.homeDirectoryForCurrentUser
        let fm = FileManager.default

        // Get installed app bundle IDs and names
        var installedIds = Set<String>()
        var installedNames = Set<String>()

        for appsDir in [URL(fileURLWithPath: "/Applications"), home.appendingPathComponent("Applications")] {
            guard let apps = try? fm.contentsOfDirectory(at: appsDir, includingPropertiesForKeys: nil) else { continue }
            for app in apps where app.pathExtension == "app" {
                installedNames.insert(app.deletingPathExtension().lastPathComponent.lowercased())
                let plistURL = app.appendingPathComponent("Contents/Info.plist")
                if let data = try? Data(contentsOf: plistURL),
                   let plist = try? PropertyListSerialization.propertyList(from: data, format: nil) as? [String: Any],
                   let bundleId = plist["CFBundleIdentifier"] as? String {
                    installedIds.insert(bundleId.lowercased())
                }
            }
        }

        // System whitelist
        let systemPrefixes = ["com.apple.", "com.microsoft.", "com.google."]

        // Scan Library locations
        let libraryPaths = [
            home.appendingPathComponent("Library/Application Support"),
            home.appendingPathComponent("Library/Caches"),
            home.appendingPathComponent("Library/Preferences"),
            home.appendingPathComponent("Library/Containers"),
            home.appendingPathComponent("Library/Group Containers"),
            home.appendingPathComponent("Library/Saved Application State"),
        ]

        var orphanedFiles: [OrphanedFile] = []
        var totalSize: UInt64 = 0

        for libPath in libraryPaths {
            guard let contents = try? fm.contentsOfDirectory(at: libPath, includingPropertiesForKeys: [.fileSizeKey, .contentAccessDateKey]) else { continue }

            for item in contents {
                let name = item.lastPathComponent.lowercased()

                // Skip system items
                if systemPrefixes.contains(where: { name.hasPrefix($0) }) { continue }

                // Check if matches any installed app
                let matchesInstalled = installedIds.contains(name) || installedNames.contains(where: { name.contains($0) })
                if matchesInstalled { continue }

                // Calculate size
                let size = directorySize(item)
                guard size >= 1_048_576 else { continue } // 1MB minimum

                let accessed = (try? item.resourceValues(forKeys: [.contentAccessDateKey]).contentAccessDate?.unixTimestamp) ?? 0

                let fileType = libPath.lastPathComponent

                orphanedFiles.append(OrphanedFile(
                    path: item.path,
                    size: size,
                    likelyApp: item.lastPathComponent,
                    lastAccessed: accessed,
                    fileType: fileType
                ))
                totalSize += size
            }
        }

        orphanedFiles.sort { $0.size > $1.size }

        return OrphanedScanResult(files: orphanedFiles, totalSize: totalSize, totalCount: UInt32(orphanedFiles.count))
    }

    private static func directorySize(_ url: URL) -> UInt64 {
        let fm = FileManager.default
        var isDir: ObjCBool = false
        guard fm.fileExists(atPath: url.path, isDirectory: &isDir) else { return 0 }

        if !isDir.boolValue {
            return (try? url.resourceValues(forKeys: [.fileSizeKey]).fileSize).flatMap { UInt64($0) } ?? 0
        }

        guard let enumerator = fm.enumerator(at: url, includingPropertiesForKeys: [.fileSizeKey, .isRegularFileKey]) else { return 0 }
        var total: UInt64 = 0
        for case let fileURL as URL in enumerator {
            guard let values = try? fileURL.resourceValues(forKeys: [.fileSizeKey, .isRegularFileKey]),
                  values.isRegularFile == true,
                  let size = values.fileSize else { continue }
            total += UInt64(size)
        }
        return total
    }
}
