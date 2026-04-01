import Foundation
import CryptoKit

enum DuplicateService {
    private static let maxFileSize: UInt64 = 500 * 1024 * 1024 // 500MB

    static func scanDuplicates(path: String, minSize: UInt64 = 1024, isCancelled: (() -> Bool)? = nil, progress: ((String) -> Void)? = nil) -> DuplicateScanResult {
        let url = URL(fileURLWithPath: path)
        let fm = FileManager.default
        var sizeMap: [UInt64: [URL]] = [:]
        var filesScanned: UInt32 = 0

        guard let enumerator = fm.enumerator(at: url, includingPropertiesForKeys: [.fileSizeKey, .isRegularFileKey], options: [.skipsHiddenFiles]) else {
            return DuplicateScanResult(groups: [], totalWasted: 0, filesScanned: 0)
        }

        // Pass 1: group by size
        for case let fileURL as URL in enumerator {
            if let cancelled = isCancelled, cancelled() { break }

            guard let values = try? fileURL.resourceValues(forKeys: [.fileSizeKey, .isRegularFileKey]),
                  values.isRegularFile == true,
                  let size = values.fileSize,
                  UInt64(size) >= minSize,
                  UInt64(size) <= maxFileSize else { continue }

            filesScanned += 1
            sizeMap[UInt64(size), default: []].append(fileURL)

            if filesScanned % 1000 == 0 {
                progress?(fileURL.deletingLastPathComponent().lastPathComponent)
            }
        }

        // Pass 2: partial hash pre-filter, then full SHA256 for candidates
        var hashMap: [String: [String]] = [:]
        var hashSizes: [String: UInt64] = [:]

        for (size, urls) in sizeMap where urls.count > 1 {
            var partialMap: [String: [URL]] = [:]
            for fileURL in urls {
                if let cancelled = isCancelled, cancelled() { break }
                if let partial = partialHash(fileURL) {
                    let key = "\(size):\(partial)"
                    partialMap[key, default: []].append(fileURL)
                }
            }

            for (_, candidates) in partialMap where candidates.count > 1 {
                for fileURL in candidates {
                    if let cancelled = isCancelled, cancelled() { break }
                    if let hash = sha256Hash(fileURL) {
                        hashMap[hash, default: []].append(fileURL.path)
                        hashSizes[hash] = size
                    }
                }
            }
        }

        // Build groups
        var groups: [DuplicateGroup] = []
        var totalWasted: UInt64 = 0

        for (hash, files) in hashMap where files.count > 1 {
            let size = hashSizes[hash] ?? 0
            let wasted = size * UInt64(files.count - 1)
            totalWasted += wasted
            groups.append(DuplicateGroup(hash: hash, size: size, files: files))
        }

        groups.sort { ($0.size * UInt64($0.files.count - 1)) > ($1.size * UInt64($1.files.count - 1)) }

        return DuplicateScanResult(groups: groups, totalWasted: totalWasted, filesScanned: filesScanned)
    }

    /// Partial hash: first 4KB + last 4KB of file
    private static func partialHash(_ url: URL) -> String? {
        guard let handle = try? FileHandle(forReadingFrom: url),
              let size = try? url.resourceValues(forKeys: [.fileSizeKey]).fileSize,
              size > 8192 else { return nil }
        defer { handle.closeFile() }

        let head = try? handle.read(upToCount: 4096)
        try? handle.seek(toOffset: UInt64(size) - 4096)
        let tail = try? handle.read(upToCount: 4096)

        guard let h = head, let t = tail else { return nil }

        var hasher = SHA256()
        hasher.update(data: h)
        hasher.update(data: t)
        let digest = hasher.finalize()
        return digest.prefix(8).map { String(format: "%02x", $0) }.joined()
    }

    private static func sha256Hash(_ url: URL) -> String? {
        guard let handle = try? FileHandle(forReadingFrom: url) else { return nil }
        defer { handle.closeFile() }

        var hasher = SHA256()
        let bufferSize = 65536

        while true {
            let data = handle.readData(ofLength: bufferSize)
            if data.isEmpty { break }
            hasher.update(data: data)
        }

        let digest = hasher.finalize()
        return digest.map { String(format: "%02x", $0) }.joined()
    }
}
