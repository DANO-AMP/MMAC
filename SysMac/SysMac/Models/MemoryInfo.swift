import Foundation

struct MemoryInfo {
    let total: UInt64
    let used: UInt64
    let free: UInt64
    let active: UInt64
    let inactive: UInt64
    let wired: UInt64
    let compressed: UInt64
    let appMemory: UInt64
    let cached: UInt64
}
