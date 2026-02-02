import Foundation

struct DuplicateGroup: Identifiable {
    let id = UUID()
    let hash: String
    let size: UInt64
    let files: [String]
}

struct DuplicateScanResult {
    let groups: [DuplicateGroup]
    let totalWasted: UInt64
    let filesScanned: UInt32
}
