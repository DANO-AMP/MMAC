import Foundation
import CryptoKit

enum DuplicateService {
    static func scanDuplicates(path: String, minSize: UInt64 = 1024) -> DuplicateScanResult {
        let url = URL(fileURLWithPath: path)
        let fm = FileManager.default
        var sizeMap: [UInt64: [URL]] = [:]
        var filesScanned: UInt32 = 0

        guard let enumerator = fm.enumerator(at: url, includingPropertiesForKeys: [.fileSizeKey, .isRegularFileKey], options: []) else {
            return DuplicateScanResult(groups: [], totalWasted: 0, filesScanned: 0)
        }

        // Pass 1: group by size
        for case let fileURL as URL in enumerator {
            guard let values = try? fileURL.resourceValues(forKeys: [.fileSizeKey, .isRegularFileKey]),
                  values.isRegularFile == true,
                  let size = values.fileSize,
                  UInt64(size) >= minSize else { continue }

            filesScanned += 1
            sizeMap[UInt64(size), default: []].append(fileURL)
        }

        // Pass 2: SHA256 hash for same-size files
        var hashMap: [String: [String]] = [:]
        var hashSizes: [String: UInt64] = [:]

        for (size, urls) in sizeMap where urls.count > 1 {
            for fileURL in urls {
                if let hash = sha256Hash(fileURL) {
                    hashMap[hash, default: []].append(fileURL.path)
                    hashSizes[hash] = size
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
