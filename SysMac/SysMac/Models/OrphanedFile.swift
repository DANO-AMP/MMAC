import Foundation

struct OrphanedFile: Identifiable {
    let id = UUID()
    let path: String
    let size: UInt64
    let likelyApp: String
    let lastAccessed: Int64
    let fileType: String
}

struct OrphanedScanResult {
    let files: [OrphanedFile]
    let totalSize: UInt64
    let totalCount: UInt32
}
