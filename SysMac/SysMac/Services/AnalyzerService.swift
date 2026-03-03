import Foundation

enum AnalyzerService {
    static func analyze(path: String, maxDepth: Int = 10) -> [DiskItem] {
        let url = URL(fileURLWithPath: path)
        guard isAllowedPath(path) else { return [] }

        let fm = FileManager.default
        guard let contents = try? fm.contentsOfDirectory(at: url, includingPropertiesForKeys: [.fileSizeKey, .isDirectoryKey]) else {
            return []
        }

        let home = fm.homeDirectoryForCurrentUser.path
        var items: [DiskItem] = []

        for itemURL in contents {
            // Skip hidden files at home root
            if path == home && itemURL.lastPathComponent.hasPrefix(".") { continue }

            // Skip symlinks to avoid counting space multiple times
            guard let values = try? itemURL.resourceValues(forKeys: [.isDirectoryKey, .isSymbolicLinkKey]) else { continue }
            if values.isSymbolicLink == true { continue }
            let isDir = values.isDirectory ?? false
            let size = isDir ? FileUtilities.directorySize(at: itemURL) : fileSize(itemURL)

            let displayPath = itemURL.path.replacingOccurrences(of: home, with: "~")

            items.append(DiskItem(name: itemURL.lastPathComponent, path: displayPath, size: size, isDir: isDir))
        }

        items.sort { $0.size > $1.size }
        return items
    }

    private static func isAllowedPath(_ path: String) -> Bool {
        let fm = FileManager.default
        let home = fm.homeDirectoryForCurrentUser.path
        let allowed = [home, "/Applications", "/tmp"]
        return allowed.contains(where: { path.hasPrefix($0) })
    }


    private static func fileSize(_ url: URL) -> UInt64 {
        guard let values = try? url.resourceValues(forKeys: [.fileSizeKey]),
              let size = values.fileSize else { return 0 }
        return UInt64(size)
    }
}
