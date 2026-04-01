import Foundation

enum LargeFilesService {
    static func findLargeFiles(path: String, minSize: UInt64 = 50 * 1024 * 1024, limit: Int = 100) -> [LargeFile] {
        let url = URL(fileURLWithPath: path)
        let fm = FileManager.default

        guard let enumerator = fm.enumerator(at: url, includingPropertiesForKeys: [.fileSizeKey, .contentModificationDateKey, .isRegularFileKey], options: []) else {
            return []
        }

        var topFiles: [LargeFile] = []

        for case let fileURL as URL in enumerator {
            guard let values = try? fileURL.resourceValues(forKeys: [.fileSizeKey, .contentModificationDateKey, .isRegularFileKey]),
                  values.isRegularFile == true,
                  let size = values.fileSize,
                  UInt64(size) >= minSize else { continue }

            let modified = values.contentModificationDate?.unixTimestamp ?? 0
            let file = LargeFile(
                path: fileURL.path,
                name: fileURL.lastPathComponent,
                size: UInt64(size),
                modified: UInt64(modified)
            )

            if topFiles.count < limit {
                topFiles.append(file)
                if topFiles.count == limit {
                    topFiles.sort { $0.size > $1.size }
                }
            } else if size > topFiles.last!.size {
                topFiles[topFiles.count - 1] = file
                topFiles.sort { $0.size > $1.size }
            }
        }

        return topFiles
    }
}
