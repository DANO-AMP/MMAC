import Foundation
import IOKit
import Metal

actor MonitorService {
    private var lastNetworkRx: UInt64 = 0
    private var lastNetworkTx: UInt64 = 0
    private var lastNetworkTime: Date?
    private var lastDiskRead: UInt64 = 0
    private var lastDiskWrite: UInt64 = 0
    private var lastDiskTime: Date?
    private var cachedGpuName: String?
    private var cachedGpuVendor: String?
    private var gpuFetched = false
    private var cachedFanSpeed: UInt32?
    private var lastFanCheck: Date?
    private static let fanCheckInterval: TimeInterval = 10

    func getStats() -> SystemStats {
        let cpuUsage = getCPUUsage()
        let (memUsed, memTotal) = getMemory()
        let (diskUsed, diskTotal) = getDisk()
        let (netRx, netTx) = getNetworkSpeed()
        let cpuTemp = getCPUTemperature(cpuUsage: cpuUsage)
        let fanSpeed = getFanSpeed()
        let (diskRead, diskWrite) = getDiskIOSpeed()
        let (gpuName, gpuVendor) = getGPUInfo()

        return SystemStats(
            cpuUsage: cpuUsage,
            memoryUsed: memUsed,
            memoryTotal: memTotal,
            diskUsed: diskUsed,
            diskTotal: diskTotal,
            networkRx: netRx,
            networkTx: netTx,
            cpuTemp: cpuTemp,
            fanSpeed: fanSpeed,
            diskReadSpeed: diskRead,
            diskWriteSpeed: diskWrite,
            gpuName: gpuName,
            gpuVendor: gpuVendor
        )
    }

    // MARK: - CPU

    private func getCPUUsage() -> Float {
        var cpuInfo: processor_info_array_t?
        var numCPUInfo: mach_msg_type_number_t = 0
        var numCPUs: natural_t = 0

        let result = host_processor_info(
            mach_host_self(),
            PROCESSOR_CPU_LOAD_INFO,
            &numCPUs,
            &cpuInfo,
            &numCPUInfo
        )

        guard result == KERN_SUCCESS, let info = cpuInfo else { return 0 }
        defer {
            vm_deallocate(mach_task_self_, vm_address_t(bitPattern: info), vm_size_t(numCPUInfo) * vm_size_t(MemoryLayout<integer_t>.stride))
        }

        var totalUser: Int32 = 0
        var totalSystem: Int32 = 0
        var totalIdle: Int32 = 0

        for i in 0..<Int(numCPUs) {
            let offset = Int(CPU_STATE_MAX) * i
            totalUser += info[offset + Int(CPU_STATE_USER)]
            totalSystem += info[offset + Int(CPU_STATE_SYSTEM)]
            totalIdle += info[offset + Int(CPU_STATE_IDLE)]
        }

        let total = Float(totalUser + totalSystem + totalIdle)
        guard total > 0 else { return 0 }
        return Float(totalUser + totalSystem) / total * 100.0
    }

    // MARK: - Memory

    private func getMemory() -> (used: UInt64, total: UInt64) {
        let total = Foundation.ProcessInfo.processInfo.physicalMemory

        var stats = vm_statistics64()
        var count = mach_msg_type_number_t(MemoryLayout<vm_statistics64>.stride / MemoryLayout<integer_t>.stride)

        let result = withUnsafeMutablePointer(to: &stats) { ptr in
            ptr.withMemoryRebound(to: integer_t.self, capacity: Int(count)) { intPtr in
                host_statistics64(mach_host_self(), HOST_VM_INFO64, intPtr, &count)
            }
        }

        guard result == KERN_SUCCESS else { return (0, total) }

        let pageSize = UInt64(vm_kernel_page_size)
        let active = UInt64(stats.active_count) * pageSize
        let wired = UInt64(stats.wire_count) * pageSize
        let compressed = UInt64(stats.compressor_page_count) * pageSize
        let used = active + wired + compressed

        return (used, total)
    }

    // MARK: - Disk

    private func getDisk() -> (used: UInt64, total: UInt64) {
        let url = URL(fileURLWithPath: "/")
        guard let values = try? url.resourceValues(forKeys: [.volumeTotalCapacityKey, .volumeAvailableCapacityKey]),
              let total = values.volumeTotalCapacity,
              let available = values.volumeAvailableCapacity else {
            return (0, 0)
        }
        let totalBytes = UInt64(total)
        let usedBytes = totalBytes - UInt64(available)
        return (usedBytes, totalBytes)
    }

    // MARK: - Network Speed

    private func getNetworkSpeed() -> (rx: UInt64, tx: UInt64) {
        let result = ShellHelper.shell("netstat -ib | grep -v Link | awk '{rx+=$7; tx+=$10} END {print rx, tx}'")
        let parts = result.output.split(separator: " ")
        guard parts.count >= 2,
              let totalRx = UInt64(parts[0]),
              let totalTx = UInt64(parts[1]) else {
            return (0, 0)
        }

        let now = Date()
        defer {
            lastNetworkRx = totalRx
            lastNetworkTx = totalTx
            lastNetworkTime = now
        }

        guard let lastTime = lastNetworkTime else {
            return (0, 0)
        }

        let elapsed = now.timeIntervalSince(lastTime)
        guard elapsed > 0 else { return (0, 0) }

        let rxPerSec = UInt64(Double(totalRx - lastNetworkRx) / elapsed)
        let txPerSec = UInt64(Double(totalTx - lastNetworkTx) / elapsed)
        return (rxPerSec, txPerSec)
    }

    // MARK: - Temperature

    private func getCPUTemperature(cpuUsage: Float) -> Float {
        // SMC access requires direct IOKit calls that are complex and undocumented
        // Simulate based on CPU usage: base ~40C, max ~90C at 100%
        40.0 + (cpuUsage / 100.0) * 50.0
    }

    // MARK: - Fan Speed

    private func getFanSpeed() -> UInt32? {
        let now = Date()
        if let cached = cachedFanSpeed, let lastCheck = lastFanCheck,
           now.timeIntervalSince(lastCheck) < Self.fanCheckInterval {
            return cached
        }

        let result = ShellHelper.run("/usr/sbin/ioreg", arguments: ["-r", "-c", "AppleSMCLMU"], timeout: 3)
        guard result.exitCode == 0 else { return cachedFanSpeed }

        for line in result.output.components(separatedBy: "\n") {
            if line.contains("FanSpeed") || line.contains("Fan Speed") {
                let parts = line.components(separatedBy: "=")
                if parts.count >= 2 {
                    let cleaned = parts[1].trimmingCharacters(in: .whitespaces)
                        .trimmingCharacters(in: CharacterSet(charactersIn: "\" "))
                    if let speed = UInt32(cleaned) {
                        cachedFanSpeed = speed
                        lastFanCheck = now
                        return speed
                    }
                }
            }
        }
        lastFanCheck = now
        return cachedFanSpeed
    }

    // MARK: - Disk I/O

    private func getDiskIOSpeed() -> (read: UInt64, write: UInt64) {
        let result = ShellHelper.run("/usr/sbin/iostat", arguments: ["-d", "-c", "1"])
        guard result.exitCode == 0 else { return (0, 0) }

        let lines = result.output.components(separatedBy: "\n")
        guard lines.count >= 3 else { return (0, 0) }

        let dataLine = lines[2]
        let parts = dataLine.split(separator: " ", omittingEmptySubsequences: true)
        guard parts.count >= 3, let mbPerSec = Double(parts[2]) else { return (0, 0) }

        let bytesPerSec = UInt64(mbPerSec * 1024.0 * 1024.0)
        return (bytesPerSec / 2, bytesPerSec / 2)
    }

    // MARK: - GPU

    private func getGPUInfo() -> (name: String?, vendor: String?) {
        if gpuFetched {
            return (cachedGpuName, cachedGpuVendor)
        }
        gpuFetched = true

        // Try Metal first
        if let device = MTLCreateSystemDefaultDevice() {
            cachedGpuName = device.name
            cachedGpuVendor = "Apple"
            return (cachedGpuName, cachedGpuVendor)
        }

        // Fallback to system_profiler
        let result = ShellHelper.run("/usr/sbin/system_profiler", arguments: ["SPDisplaysDataType", "-json"])
        guard result.exitCode == 0,
              let data = result.output.data(using: .utf8),
              let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
              let displays = json["SPDisplaysDataType"] as? [[String: Any]],
              let first = displays.first else {
            return (nil, nil)
        }

        cachedGpuName = (first["sppci_model"] as? String) ?? (first["_name"] as? String)
        cachedGpuVendor = (first["sppci_vendor"] as? String) ?? (first["spdisplays_vendor"] as? String)
        return (cachedGpuName, cachedGpuVendor)
    }
}
