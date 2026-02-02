import Foundation

struct BrewPackage: Identifiable {
    let id = UUID()
    let name: String
    let version: String
    let isOutdated: Bool
    let newerVersion: String?
    let isCask: Bool
}

struct HomebrewInfo {
    let installed: Bool
    let version: String?
    let formulaeCount: UInt32
    let casksCount: UInt32
}
