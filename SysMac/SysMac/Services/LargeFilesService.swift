import Foundation

enum LargeFilesService {
    static func findLargeFiles(path: String, minSize: UInt64 = 50 * 1024 * 1024, limit: Int = 100) -> [LargeFile] {
        let url = URL(fileURLWithPath: path)
        let fm = FileManager.default

        guard let enumerator = fm.enumerator(at: url, includingPropertiesForKeys: [.fileSizeKey, .contentModificationDateKey, .isRegularFileKey], options: []) else {
            return []
        }

        var files: [LargeFile] = []

        for case let fileURL as URL in enumerator {
            guard let values = try? fileURL.resourceValues(forKeys: [.fileSizeKey, .contentModificationDateKey, .isRegularFileKey]),
                  values.isRegularFile == true,
                  let size = values.fileSize,
                  UInt64(size) >= minSize else { continue }

            let modified = values.contentModificationDate?.unixTimestamp ?? 0

            files.append(LargeFile(
                path: fileURL.path,
                name: fileURL.lastPathComponent,
                size: UInt64(size),
                modified: UInt64(modified)
            ))
        }

        files.sort { $0.size > $1.size }
        if files.count > limit {
            files = Array(files.prefix(limit))
        }
        return files
    }
}
