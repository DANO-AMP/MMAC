import Foundation

enum CleaningService {
    static func scanAll() -> [ScanResult] {
        let home = FileManager.default.homeDirectoryForCurrentUser
        return [
            scanCaches(home),
            scanLogs(home),
            scanBrowserData(home),
            scanTrash(home),
            scanCrashReports(home),
            scanXcodeData(home),
            scanPackageCaches(home),
        ]
    }

    static func cleanCategory(_ category: String, paths: [String], moveToTrash: Bool) -> UInt64 {
        var freed: UInt64 = 0
        let fm = FileManager.default
        for path in paths {
            guard fm.fileExists(atPath: path) else { continue }
            guard case .success(let validatedURL) = PathValidator.validateForDeletion(path) else { continue }
            let size = FileUtilities.directorySize(at: validatedURL)
            do {
                if moveToTrash && category != "trash" {
                    try fm.trashItem(at: validatedURL, resultingItemURL: nil)
                } else {
                    try fm.removeItem(at: validatedURL)
                }
                freed += size
            } catch { /* skip */ }
        }
        return freed
    }

    // MARK: - Categories

    private static func scanCaches(_ home: URL) -> ScanResult {
        let paths = [
            home.appendingPathComponent("Library/Caches"),
        ]
        return scanCategory("cache", displayPaths: paths)
    }

    private static func scanLogs(_ home: URL) -> ScanResult {
        let paths = [
            home.appendingPathComponent("Library/Logs"),
        ]
        return scanCategory("logs", displayPaths: paths)
    }

    private static func scanBrowserData(_ home: URL) -> ScanResult {
        let paths = [
            home.appendingPathComponent("Library/Caches/Google/Chrome"),
            home.appendingPathComponent("Library/Caches/com.apple.Safari"),
            home.appendingPathComponent("Library/Caches/Firefox"),
            home.appendingPathComponent("Library/Caches/company.thebrowser.Browser"),  // Arc
        ]
        return scanCategory("browser", displayPaths: paths)
    }

    private static func scanTrash(_ home: URL) -> ScanResult {
        let paths = [
            home.appendingPathComponent(".Trash"),
        ]
        return scanCategory("trash", displayPaths: paths)
    }

    private static func scanCrashReports(_ home: URL) -> ScanResult {
        let paths = [
            home.appendingPathComponent("Library/Logs/DiagnosticReports"),
            URL(fileURLWithPath: "/Library/Logs/DiagnosticReports"),
        ]
        return scanCategory("crash_reports", displayPaths: paths)
    }

    private static func scanXcodeData(_ home: URL) -> ScanResult {
        let paths = [
            home.appendingPathComponent("Library/Developer/Xcode/DerivedData"),
            home.appendingPathComponent("Library/Developer/Xcode/Archives"),
            home.appendingPathComponent("Library/Developer/Xcode/iOS DeviceSupport"),
        ]
        return scanCategory("xcode", displayPaths: paths)
    }

    private static func scanPackageCaches(_ home: URL) -> ScanResult {
        let paths = [
            home.appendingPathComponent(".npm/_cacache"),
            home.appendingPathComponent(".yarn/cache"),
            home.appendingPathComponent("Library/pnpm/store"),
            home.appendingPathComponent("Library/Caches/pip"),
            home.appendingPathComponent(".cargo/registry"),
            home.appendingPathComponent(".composer/cache"),
            home.appendingPathComponent("Library/Caches/CocoaPods"),
            home.appendingPathComponent(".gradle/caches"),
            home.appendingPathComponent(".m2/repository"),
        ]
        return scanCategory("packages", displayPaths: paths)
    }

    // MARK: - Helpers

    private static func scanCategory(_ category: String, displayPaths: [URL]) -> ScanResult {
        var totalSize: UInt64 = 0
        var items: UInt32 = 0
        var existingPaths: [String] = []
        let fm = FileManager.default

        for url in displayPaths {
            guard fm.fileExists(atPath: url.path) else { continue }
            let size = FileUtilities.directorySize(at: url)
            if size > 0 {
                totalSize += size
                items += 1
                existingPaths.append(url.path)
            }
        }

        return ScanResult(category: category, size: totalSize, items: items, paths: existingPaths)
    }

}
