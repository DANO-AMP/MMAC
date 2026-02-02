import Foundation

struct LargeFile: Identifiable {
    let id = UUID()
    let path: String
    let name: String
    let size: UInt64
    let modified: UInt64
}
