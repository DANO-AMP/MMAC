import Foundation

struct DiskItem: Identifiable {
    let id = UUID()
    let name: String
    let path: String
    let size: UInt64
    let isDir: Bool
}
