# SysMac Performance, Security & Data Integrity Improvements

**Date:** 2026-04-01
**Scope:** Performance-critical fixes, security hardening, and data integrity improvements
**Approach:** Performance-First -- fix critical user-facing issues, then security/data integrity

---

## Section 1: Subprocess Batching (P0)

### Problem
- `NetworkService.getConnections()` spawns `/bin/ps` once per connection (~200+ forks every 3s)
- `PortScannerService.scan()` spawns 3 processes per listening port (~45+ forks every 3s)
- `ProcessService.getAllProcesses()` spawns 2 separate `ps` processes every 3s

### Solution

#### NetworkService.swift
Replace per-PID `getProcessName()` loop with single batch call:
```swift
// Single fork: get all PIDs and names at once
let result = ShellHelper.run("/bin/ps", arguments: ["-axo", "pid=,comm="])
let processMap = Dictionary(uniqueKeysWithValues:
    result.output.split(separator: "\n").compactMap { line -> (UInt32, String)? in
        let parts = line.split(separator: " ", maxSplits: 1, omittingEmptySubsequences: true)
        guard parts.count == 2, let pid = UInt32(parts[0]) else { return nil }
        return (pid, String(parts[1]))
    }
)
// O(1) lookup per connection instead of fork
```

#### PortScannerService.swift
Consolidate 3 calls per port into 3 total calls:
```swift
// Batch all PIDs into single ps calls
let pidList = ports.map(\.pid).unique.map(String.init).joined(separator: ",")
let statsResult = ShellHelper.run("/bin/ps", arguments: ["-p", pidList, "-o", "pid=,%cpu,rss="])
let cmdResult = ShellHelper.run("/bin/ps", arguments: ["-p", pidList, "-o", "pid=,args="])
let cwdResult = ShellHelper.run("/usr/sbin/lsof", arguments: ["-p", pidList, "-d", "cwd", "-Fn"])
```

#### ProcessService.swift
Merge 2 `ps` calls into 1:
```swift
// Single call with full args column instead of truncated comm
let result = ShellHelper.run("/bin/ps", arguments: ["-axo", "pid,ppid,%cpu,rss,%mem,user,state,wq,args=", "-r"])
```

### Impact
~250 subprocesses/3s reduced to ~5 subprocesses/3s. UI no longer freezes during refresh.

---

## Section 2: Scanning Safety (P0)

### Problem
- `DuplicateService.scanDuplicates(path:homeDir)` computes full SHA-256 of every file with no cancellation, no progress, no size limit
- `LargeFilesService` accumulates ALL matching files in memory before sorting and truncating
- `OrphanedService.scanOrphanedFiles()` calls `directorySize()` for every non-matching Library item

### Solution

#### DuplicateService.swift
1. **Pre-filter by partial hash**: Hash first 4KB + last 4KB + file size before full SHA-256
2. **Max file size**: Skip files > 500MB (VM images, disk images)
3. **Cancellation support**: Accept a closure `() -> Bool` that checks `Task.isCancelled`
4. **Progress callback**: Accept `(String) -> Void` for current directory being scanned

#### LargeFilesService.swift
Replace unbounded array with in-place top-N tracking:
```swift
static func findLargeFiles(path: String, minSize: UInt64 = 50 * 1024 * 1024, limit: Int = 50) -> [LargeFile] {
    var topFiles: [LargeFile] = []
    for case let fileURL as URL in enumerator {
        guard let size = values.fileSize, size >= minSize else { continue }
        let file = LargeFile(name: fileURL.lastPathComponent, path: fileURL.path, size: size)
        if topFiles.count < limit {
            topFiles.append(file)
            if topFiles.count == limit { topFiles.sort { $0.size > $1.size } }
        } else if size > topFiles.last!.size {
            topFiles[topFiles.count - 1] = file
            topFiles.sort { $0.size > $1.size }
        }
    }
    return topFiles
}
```

#### OrphanedService.swift
Add cheap pre-check before expensive `directorySize()`:
```swift
// Check top-level directory metadata size before recursing
guard let topSize = try? url.resourceValues(forKeys: [.totalFileSizeKey]).totalFileSize,
      topSize >= 1_048_576 else { continue }
```

### Impact
Duplicate scan: minutes to seconds. Large files scan: bounded memory. Orphaned scan: fewer unnecessary traversals.

---

## Section 3: Concurrent Cleaning + Polling Optimization (P1)

### Problem
- `CleaningService.scanAll()` runs 7 directory size calculations sequentially (~10-30s)
- `MonitorService.getStats()` forks 5 subprocesses every 2s, including `iostat -c 1` that blocks 1s
- `BatteryViewModel` and `BluetoothViewModel` poll every 5s for data that changes rarely

### Solution

#### CleaningService.swift
Dispatch all 7 scans concurrently with `TaskGroup`:
```swift
static func scanAll() async -> [ScanResult] {
    await withTaskGroup(of: ScanResult.self) { group in
        for category in allCategories {
            group.addTask { scanCategory(category) }
        }
        var results: [ScanResult] = []
        for await result in group {
            results.append(result)
        }
        return results
    }
}
```

#### MonitorService.swift
- Replace `netstat | grep | awk` pipeline with native `getifaddrs()` call
- Cache fan speed result, poll every 10s instead of 2s
- Use shorter `iostat` sample or skip if no disk activity
- Skip `ioreg -c AppleSMCLMU` entirely on hardware without SMC

#### BatteryViewModel.swift / BluetoothViewModel.swift
- Battery polling: 5s -> 30s
- Bluetooth polling: 5s -> 10s

### Impact
Cleaning scan: 30s -> ~5s. Monitor: more responsive, less CPU. Battery/Bluetooth: less wasted work.

---

## Section 4: Data Integrity + Security (P0)

### Problem
- 4 `catch { /* skip */ }` blocks silently discard file deletion errors
- `OrphanedViewModel` and `LargeFilesViewModel` bypass Service layer and PathValidator
- `ShellHelper.shell()` accepts any String as shell command (footgun)

### Solution

#### 4a. Report deletion errors instead of silencing
In `OrphanedViewModel.swift`, `LargeFilesViewModel.swift`, `CleaningService.swift`:
```swift
var failedPaths: [String] = []
for path in selectedPaths {
    do {
        // ... deletion ...
    } catch {
        failedPaths.append(path)
    }
}
if !failedPaths.isEmpty {
    error = "\(failedPaths.count) elementos no se pudieron eliminar"
}
// Only clear selectedPaths on full success
guard failedPaths.isEmpty else { return }
selectedPaths.removeAll()
```

#### 4b. Move deletion to Service layer with PathValidator
- `OrphanedService.deleteFiles(paths:moveToTrash:)` -- validates via PathValidator
- `LargeFilesService.deleteFile(path:moveToTrash:)` -- validates via PathValidator
- ViewModels delegate to these service methods instead of calling FileManager directly

#### 4c. ShellHelper hardening
- Rename `shell()` to `shellUnfiltered()` with documentation warning
- Replace callers with `ShellHelper.run()` using argument arrays where possible
- `dscacheutil -flushcache` -> `ShellHelper.run("/usr/bin/dscacheutil", arguments: ["-flushcache"])`
- `osascript` calls remain with `shellUnfiltered()` (hardcoded, no user input)

### Impact
Users see real deletion status. All file operations go through PathValidator. Reduced attack surface.

---

## Out of Scope (Future Pass)
- Protocol-based Service abstraction for testability
- `PollingViewModel` base class to eliminate 280 lines of boilerplate
- `LoadingState<T>` enum for consistent error handling
- `ServiceError.Kind` enum for categorized errors
- Shared memory stats cache between MonitorService and MemoryService
- `filteredProcesses` caching in ProcessesViewModel/ConnectionsViewModel
