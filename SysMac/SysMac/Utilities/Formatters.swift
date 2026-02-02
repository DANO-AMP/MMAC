import Foundation

enum Formatters {
    static func formatSize(_ bytes: UInt64) -> String {
        guard bytes > 0 else { return "0 B" }
        let k: Double = 1024
        let sizes = ["B", "KB", "MB", "GB", "TB"]
        let i = min(Int(log(Double(bytes)) / log(k)), sizes.count - 1)
        let value = Double(bytes) / pow(k, Double(i))
        let formatted = String(format: "%.1f", value)
        // Remove trailing .0
        let clean = formatted.hasSuffix(".0") ? String(formatted.dropLast(2)) : formatted
        return "\(clean) \(sizes[i])"
    }

    static func formatSize(_ bytes: Int64) -> String {
        guard bytes > 0 else { return "0 B" }
        return formatSize(UInt64(bytes))
    }

    static func formatSpeed(_ bytesPerSec: UInt64) -> String {
        guard bytesPerSec > 0 else { return "0 B/s" }
        let k: Double = 1024
        let sizes = ["B/s", "KB/s", "MB/s", "GB/s"]
        let i = min(Int(log(Double(bytesPerSec)) / log(k)), sizes.count - 1)
        let value = Double(bytesPerSec) / pow(k, Double(i))
        let formatted = String(format: "%.1f", value)
        let clean = formatted.hasSuffix(".0") ? String(formatted.dropLast(2)) : formatted
        return "\(clean) \(sizes[i])"
    }

    static func formatPercentage(_ value: Double, decimals: Int = 1) -> String {
        guard value.isFinite else { return "0%" }
        return String(format: "%.\(decimals)f%%", value)
    }

    static func formatTemperature(_ celsius: Double) -> String {
        guard celsius.isFinite else { return "N/A" }
        return String(format: "%.1f\u{00B0}C", celsius)
    }

    static func formatDate(_ date: Date) -> String {
        let formatter = DateFormatter()
        formatter.dateStyle = .medium
        formatter.timeStyle = .none
        return formatter.string(from: date)
    }

    static func formatTime(_ date: Date) -> String {
        let formatter = DateFormatter()
        formatter.dateStyle = .none
        formatter.timeStyle = .medium
        return formatter.string(from: date)
    }

    static func formatDateTime(_ date: Date) -> String {
        let formatter = DateFormatter()
        formatter.dateStyle = .medium
        formatter.timeStyle = .medium
        return formatter.string(from: date)
    }

    static func formatDuration(hours: Int, minutes: Int) -> String {
        return "\(hours):\(String(format: "%02d", minutes))"
    }
}
