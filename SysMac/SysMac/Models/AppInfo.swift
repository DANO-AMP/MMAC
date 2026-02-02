import Foundation

struct RemnantFile: Identifiable {
    let id = UUID()
    let path: String
    let size: UInt64
    let remnantType: String
}

struct AppInfo: Identifiable {
    let id = UUID()
    let name: String
    let bundleId: String
    let path: String
    let size: UInt64
    let version: String?
    let remnants: [RemnantFile]
    let remnantsSize: UInt64
}
