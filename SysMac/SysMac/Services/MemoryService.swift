import Foundation
import Darwin

enum MemoryService {
    static func getMemoryInfo() -> MemoryInfo {
        let total = Foundation.ProcessInfo.processInfo.physicalMemory

        var stats = vm_statistics64()
        var count = mach_msg_type_number_t(MemoryLayout<vm_statistics64>.stride / MemoryLayout<integer_t>.stride)

        let result = withUnsafeMutablePointer(to: &stats) { ptr in
            ptr.withMemoryRebound(to: integer_t.self, capacity: Int(count)) { intPtr in
                host_statistics64(mach_host_self(), HOST_VM_INFO64, intPtr, &count)
            }
        }

        guard result == KERN_SUCCESS else {
            return MemoryInfo(total: total, used: 0, free: total, active: 0, inactive: 0, wired: 0, compressed: 0, appMemory: 0, cached: 0)
        }

        let pageSize = UInt64(vm_kernel_page_size)
        let active = UInt64(stats.active_count) * pageSize
        let inactive = UInt64(stats.inactive_count) * pageSize
        let wired = UInt64(stats.wire_count) * pageSize
        let compressed = UInt64(stats.compressor_page_count) * pageSize
        let free = UInt64(stats.free_count) * pageSize
        let cached = UInt64(stats.external_page_count) * pageSize
        let used = active + wired + compressed
        let appMemory = active + wired

        return MemoryInfo(
            total: total,
            used: used,
            free: free,
            active: active,
            inactive: inactive,
            wired: wired,
            compressed: compressed,
            appMemory: appMemory,
            cached: cached
        )
    }

    static func purgeMemory() -> Result<String, ServiceError> {
        .failure(ServiceError("Para purgar memoria, ejecuta en Terminal: sudo purge\n\nEsta operación requiere permisos de administrador y no puede ejecutarse directamente desde la aplicación por razones de seguridad."))
    }
}
