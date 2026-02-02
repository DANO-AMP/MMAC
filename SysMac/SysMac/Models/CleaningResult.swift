import Foundation

struct ScanResult: Identifiable {
    let id = UUID()
    let category: String
    let size: UInt64
    let items: UInt32
    let paths: [String]
}
