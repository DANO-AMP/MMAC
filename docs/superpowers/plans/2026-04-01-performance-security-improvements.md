# Performance, Security & Data Integrity Improvements - Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Eliminate user-facing performance issues (N+1 subprocesses, unbounded scans, UI freezes) and fix data integrity bugs (silent error swallowing, missing PathValidator) in the SysMac macOS app.

**Architecture:** Modify existing Service and ViewModel files in-place. No new files created, no architectural restructuring. Each task is self-contained and independently committable.

**Tech Stack:** Swift 5.9, SwiftUI, Foundation, CryptoKit

---

## File Map

| File | Responsibility | Tasks |
|------|---------------|-------|
| `SysMac/SysMac/Services/NetworkService.swift` | Network connection monitoring | Task 1 |
| `SysMac/SysMac/Services/PortScannerService.swift` | Port scanning | Task 2 |
| `SysMac/SysMac/Services/ProcessService.swift` | Process listing | Task 3 |
| `SysMac/SysMac/Services/DuplicateService.swift` | Duplicate file detection | Task 4 |
| `SysMac/SysMac/ViewModels/DuplicatesViewModel.swift` | Duplicate scan UI state | Task 4 |
| `SysMac/SysMac/Services/LargeFilesService.swift` | Large file finder | Task 5 |
| `SysMac/SysMac/Services/OrphanedService.swift` | Orphaned file scanner | Task 6 |
| `SysMac/SysMac/Services/CleaningService.swift` | System cleaning | Task 7 |
| `SysMac/SysMac/ViewModels/CleaningViewModel.swift` | Cleaning UI state | Task 7 |
| `SysMac/SysMac/Services/MonitorService.swift` | System stats (CPU, mem, net, disk) | Task 8 |
| `SysMac/SysMac/ViewModels/BatteryViewModel.swift` | Battery polling | Task 9 |
| `SysMac/SysMac/ViewModels/BluetoothViewModel.swift` | Bluetooth polling | Task 9 |
| `SysMac/SysMac/ViewModels/OrphanedViewModel.swift` | Orphaned file deletion UI | Task 10 |
| `SysMac/SysMac/Services/OrphanedService.swift` | Orphaned file deletion logic | Task 10 |
| `SysMac/SysMac/ViewModels/LargeFilesViewModel.swift` | Large file deletion UI | Task 10 |
| `SysMac/SysMac/Services/LargeFilesService.swift` | Large file deletion logic | Task 10 |
| `SysMac/SysMac/Services/ShellHelper.swift` | Shell command execution | Task 11 |
| `SysMac/SysMac/Services/NetworkService.swift` | DNS flush fix | Task 11 |
| `SysMac/SysMac/Services/FirewallService.swift` | Firewall defaults reads | Task 11 |

---

### Task 1: Batch NetworkService process lookups

**Files:**
- Modify: `SysMac/SysMac/Services/NetworkService.swift:4-62`

- [ ] **Step 1: Add batch process name cache to `getConnections()`**

Replace the per-PID `getProcessName()` call in `parseNetstatLine` with a pre-populated dictionary built from a single `ps` call at the top of `getConnections()`.

```swift
enum NetworkService {
    static func getConnections() -> [NetworkConnection] {
        var connections: [NetworkConnection] = []

        // Batch-fetch all process names in a single subprocess
        let allProcs = ShellHelper.run("/bin/ps", arguments: ["-axo", "pid=,comm="])
        let processMap: [UInt32: String] = {
            var map: [UInt32: String] = [:]
            for line in allProcs.output.components(separatedBy: "\n") {
                let parts = line.split(separator: " ", maxSplits: 1, omittingEmptySubsequences: true)
                guard parts.count == 2, let pid = UInt32(parts[0]) else { continue }
                let name = parts[1].split(separator: "/").last.map(String.init) ?? String(parts[1])
                map[pid] = name
            }
            return map
        }()

        for proto in ["tcp", "udp"] {
            let result = ShellHelper.run("/usr/sbin/netstat", arguments: ["-anvp", proto])
            guard result.exitCode == 0 else { continue }

            for line in result.output.components(separatedBy: "\n").dropFirst(2) {
                if let conn = parseNetstatLine(line, proto: proto, processMap: processMap) {
                    connections.append(conn)
                }
            }
        }

        return connections
    }

    private static func parseNetstatLine(_ line: String, proto: String, processMap: [UInt32: String]) -> NetworkConnection? {
        let parts = line.split(separator: " ", omittingEmptySubsequences: true)
        guard parts.count >= 9 else { return nil }

        let localFull = String(parts[3])
        let remoteFull = String(parts[4])
        let state = proto == "tcp" ? String(parts[5]) : ""
        let pid = UInt32(parts.last ?? "0") ?? 0

        let (localAddr, localPort) = parseAddress(localFull)
        let (remoteAddr, remotePort) = parseAddress(remoteFull)

        return NetworkConnection(
            proto: proto.uppercased(),
            localAddress: localAddr,
            localPort: localPort,
            remoteAddress: remoteAddr,
            remotePort: remotePort,
            state: state,
            pid: pid,
            processName: pid > 0 ? (processMap[pid] ?? "") : ""
        )
    }

    // ... rest of file unchanged (parseAddress, getHostsFile, flushDNS)
}
```

- [ ] **Step 2: Remove the old `getProcessName()` method**

Delete lines 58-62 (the `private static func getProcessName(_ pid: UInt32) -> String` method). It is no longer called.

- [ ] **Step 3: Build to verify no compilation errors**

Run: `cd /Users/me/Documents/MMAC/SysMac && swift build 2>&1 | head -20`
Expected: Build succeeds (or only pre-existing warnings).

- [ ] **Step 4: Commit**

```bash
git add SysMac/SysMac/Services/NetworkService.swift
git commit -m "perf: batch NetworkService process lookups into single ps call"
```

---

### Task 2: Batch PortScannerService subprocess calls

**Files:**
- Modify: `SysMac/SysMac/Services/PortScannerService.swift:3-101`

- [ ] **Step 1: Replace per-port subprocess calls with batched lookups**

Add three batch helper methods at the bottom of `PortScannerService` (before the closing `}`), and refactor `scan()` to collect PIDs first then batch-lookup:

```swift
enum PortScannerService {
    static func scan() -> [PortInfo] {
        let result = ShellHelper.run("/usr/sbin/lsof", arguments: ["-i", "-P", "-n"])
        guard result.exitCode == 0 else { return [] }

        var portMap: [UInt16: PortInfo] = [:]

        for line in result.output.components(separatedBy: "\n").dropFirst() {
            guard line.contains("LISTEN") || line.contains("(LISTEN)") else { continue }

            let parts = line.split(separator: " ", omittingEmptySubsequences: true)
            guard parts.count >= 9 else { continue }

            let processName = String(parts[0])
            let pid = UInt32(parts[1]) ?? 0

            let nameField = String(parts[8])
            guard let colonIdx = nameField.lastIndex(of: ":") else { continue }
            let portStr = nameField[nameField.index(after: colonIdx)...]
            guard let port = UInt16(portStr) else { continue }
            let localAddr = String(nameField[..<colonIdx])

            let proto = String(parts[7]).uppercased().contains("UDP") ? "UDP" : "TCP"

            if portMap[port] == nil {
                portMap[port] = PortInfo(
                    port: port,
                    pid: pid,
                    processName: processName,
                    serviceType: detectServiceType(processName: processName, port: port),
                    proto: proto,
                    localAddress: localAddr,
                    workingDir: "",
                    command: "",
                    cpuUsage: 0,
                    memoryMB: 0
                )
            }
        }

        // Batch-fetch all details in 3 subprocess calls instead of 3 per port
        let ports = Array(portMap.values)
        let pids = ports.map(\.pid)
        let pidString = pids.map(String.init).joined(separator: ",")

        guard !pidString.isEmpty else { return ports.sorted { $0.port < $1.port } }

        let statsMap = batchProcessStats(pidString)
        let cmdMap = batchCommands(pidString)
        let cwdMap = batchWorkingDirs(pidString)

        // Merge batch results back into port info
        var mergedMap: [UInt16: PortInfo] = [:]
        for port in ports {
            let stats = statsMap[port.pid] ?? (0, 0)
            mergedMap[port.port] = PortInfo(
                port: port.port,
                pid: port.pid,
                processName: port.processName,
                serviceType: port.serviceType,
                proto: port.proto,
                localAddress: port.localAddress,
                workingDir: cwdMap[port.pid] ?? port.workingDir,
                command: cmdMap[port.pid] ?? port.command,
                cpuUsage: stats.0,
                memoryMB: stats.1
            )
        }

        return Array(mergedMap.values).sorted { $0.port < $1.port }
    }

    // ... detectServiceType unchanged ...

    // MARK: - Batch Lookups

    private static func batchProcessStats(_ pidList: String) -> [UInt32: (Float, Float)] {
        let result = ShellHelper.run("/bin/ps", arguments: ["-p", pidList, "-o", "pid=,%cpu,rss="], environment: ["LC_ALL": "C"])
        var map: [UInt32: (Float, Float)] = [:]
        for line in result.output.components(separatedBy: "\n") {
            let parts = line.split(separator: " ", omittingEmptySubsequences: true)
            guard parts.count >= 3, let pid = UInt32(parts[0]) else { continue }
            let cpu = Float(parts[1]) ?? 0
            let rssKb = Float(parts[2]) ?? 0
            map[pid] = (cpu, rssKb / 1024.0)
        }
        return map
    }

    private static func batchWorkingDirs(_ pidList: String) -> [UInt32: String] {
        let result = ShellHelper.run("/usr/sbin/lsof", arguments: ["-p", pidList, "-d", "cwd", "-Fn"])
        var map: [UInt32: String] = [:]
        var currentPid: UInt32?
        for line in result.output.components(separatedBy: "\n") {
            if line.hasPrefix("p") {
                currentPid = UInt32(line.dropFirst())
            } else if line.hasPrefix("n/"), let pid = currentPid {
                map[pid] = String(line.dropFirst())
                currentPid = nil
            }
        }
        return map
    }

    private static func batchCommands(_ pidList: String) -> [UInt32: String] {
        let result = ShellHelper.run("/bin/ps", arguments: ["-p", pidList, "-o", "pid=,args="], environment: ["LC_ALL": "C"])
        var map: [UInt32: String] = [:]
        for line in result.output.components(separatedBy: "\n") {
            let parts = line.split(separator: " ", maxSplits: 1, omittingEmptySubsequences: true)
            guard parts.count == 2, let pid = UInt32(parts[0]) else { continue }
            let cmd = String(parts[1]).trimmingCharacters(in: .whitespacesAndNewlines)
            if !cmd.isEmpty { map[pid] = cmd }
        }
        return map
    }
}
```

- [ ] **Step 2: Remove old per-PID methods**

Delete the three old methods: `getProcessStats(_:)` (lines 78-85), `getWorkingDir(_:)` (lines 87-95), `getCommand(_:)` (lines 97-101).

- [ ] **Step 3: Build to verify**

Run: `cd /Users/me/Documents/MMAC/SysMac && swift build 2>&1 | head -20`
Expected: Build succeeds.

- [ ] **Step 4: Commit**

```bash
git add SysMac/SysMac/Services/PortScannerService.swift
git commit -m "perf: batch PortScannerService lookups into 3 subprocess calls total"
```

---

### Task 3: Merge ProcessService dual ps calls into one

**Files:**
- Modify: `SysMac/SysMac/Services/ProcessService.swift:4-42`

- [ ] **Step 1: Use single ps call with `args=` column instead of `comm`**

Change the first `ps` call to request `args=` (full command) instead of `comm`, and remove `getAllFullCommands()`. Update column count parsing from `maxSplits: 8` to `maxSplits: 9` since `args` can contain spaces.

```swift
enum ProcessService {
    static func getAllProcesses() -> [ProcessItem] {
        // Single ps call with full args column (replaces two separate calls)
        let result = ShellHelper.run("/bin/ps", arguments: ["-axo", "pid,ppid,%cpu,rss,%mem,user,state,wq,args=", "-r"], environment: ["LC_ALL": "C"])
        guard result.exitCode == 0 else { return [] }

        var processes: [ProcessItem] = []
        let lines = result.output.components(separatedBy: "\n")

        for line in lines.dropFirst() {
            if let proc = parsePsLine(line) {
                processes.append(proc)
            }
        }

        return processes
    }

    static func parsePsLine(_ line: String) -> ProcessItem? {
        let parts = line.split(separator: " ", maxSplits: 8, omittingEmptySubsequences: true)
        guard parts.count >= 9 else { return nil }

        guard let pid = UInt32(parts[0]) else { return nil }
        let ppid = UInt32(parts[1]) ?? 0
        let cpuUsage = Float(parts[2]) ?? 0
        let rssKb = Float(parts[3]) ?? 0
        let memPercent = Float(parts[4]) ?? 0
        let user = String(parts[5])
        let state = parseState(String(parts[6]))
        let threads = UInt32(parts[7]) ?? 1
        let fullCommand = line[line.index(line.startIndex, offsetBy: line.distance(from: line.startIndex, to: String(parts[0]).endIndex) + 1 + line.distance(from: String(parts[0]).endIndex, to: String(parts[7]).endIndex) + 1)...].trimmingCharacters(in: .whitespaces)
        let name = fullCommand.split(separator: "/").last.map(String.init) ?? fullCommand

        return ProcessItem(
            pid: pid,
            ppid: ppid,
            name: name,
            cpuUsage: cpuUsage,
            memoryMB: rssKb / 1024.0,
            memoryPercent: memPercent,
            user: user,
            state: state,
            threads: threads,
            command: fullCommand
        )
    }

    // ... parseState, killProcess, sendSignal unchanged ...
}
```

> **Note on parsing:** The `args=` column can contain spaces, so the `split(maxSplits: 8)` will put everything from the 9th column onward into `parts[8]`. For the full command, re-extract from the original line after the 8th field boundary. Alternatively, use a simpler approach: take everything after the last tab/space sequence that follows field 8.

**Simpler parsing approach (preferred):**

```swift
    static func parsePsLine(_ line: String) -> ProcessItem? {
        // Find positions of the first 8 fields, then everything after is the command
        let scanner = Scanner(string: line)
        scanner.charactersToBeSkipped = CharacterSet.whitespaces

        guard let pidStr = scanner.scanUpToString(" "),
              let pid = UInt32(pidStr),
              let ppidStr = scanner.scanUpToString(" "),
              let ppid = UInt32(ppidStr),
              let cpuStr = scanner.scanUpToString(" "),
              let cpuUsage = Float(cpuStr),
              let rssStr = scanner.scanUpToString(" "),
              let rssKb = Float(rssStr),
              let memStr = scanner.scanUpToString(" "),
              let memPercent = Float(memStr),
              let user = scanner.scanUpToString(" "),
              let stateStr = scanner.scanUpToString(" "),
              let threadsStr = scanner.scanUpToString(" "),
              let threads = UInt32(threadsStr) else { return nil }

        // Everything remaining is the full command
        let command = scanner.remainingString?.trimmingCharacters(in: .whitespaces) ?? ""
        let name = command.split(separator: "/").last.map(String.init) ?? command

        return ProcessItem(
            pid: pid,
            ppid: ppid,
            name: name,
            cpuUsage: cpuUsage,
            memoryMB: rssKb / 1024.0,
            memoryPercent: memPercent,
            user: user,
            state: parseState(stateStr),
            threads: threads,
            command: command
        )
    }
```

- [ ] **Step 2: Delete `getAllFullCommands()` method**

Remove the `private static func getAllFullCommands()` method entirely (lines 28-42).

- [ ] **Step 3: Build to verify**

Run: `cd /Users/me/Documents/MMAC/SysMac && swift build 2>&1 | head -20`

- [ ] **Step 4: Commit**

```bash
git add SysMac/SysMac/Services/ProcessService.swift
git commit -m "perf: merge ProcessService dual ps calls into single subprocess"
```

---

### Task 4: Add cancellation, max file size, and pre-filter to DuplicateService

**Files:**
- Modify: `SysMac/SysMac/Services/DuplicateService.swift:4-71`
- Modify: `SysMac/SysMac/ViewModels/DuplicatesViewModel.swift:3-20`

- [ ] **Step 1: Add partial hash pre-filter and max file size to DuplicateService**

```swift
import Foundation
import CryptoKit

enum DuplicateService {
    private static let maxFileSize: UInt64 = 500 * 1024 * 1024 // 500MB

    static func scanDuplicates(path: String, minSize: UInt64 = 1024, isCancelled: (() -> Bool)? = nil, progress: ((String) -> Void)? = nil) -> DuplicateScanResult {
        let url = URL(fileURLWithPath: path)
        let fm = FileManager.default
        var sizeMap: [UInt64: [URL]] = [:]
        var filesScanned: UInt32 = 0

        guard let enumerator = fm.enumerator(at: url, includingPropertiesForKeys: [.fileSizeKey, .isRegularFileKey], options: [.skipsHiddenFiles]) else {
            return DuplicateScanResult(groups: [], totalWasted: 0, filesScanned: 0)
        }

        // Pass 1: group by size
        for case let fileURL as URL in enumerator {
            if let cancelled = isCancelled, cancelled() { break }

            guard let values = try? fileURL.resourceValues(forKeys: [.fileSizeKey, .isRegularFileKey]),
                  values.isRegularFile == true,
                  let size = values.fileSize,
                  UInt64(size) >= minSize,
                  UInt64(size) <= maxFileSize else { continue }

            filesScanned += 1
            sizeMap[UInt64(size), default: []].append(fileURL)

            if filesScanned % 1000 == 0 {
                progress?(fileURL.deletingLastPathComponent().lastPathComponent)
            }
        }

        // Pass 2: partial hash pre-filter, then full SHA256 for candidates
        var hashMap: [String: [String]] = [:]
        var hashSizes: [String: UInt64] = [:]

        for (size, urls) in sizeMap where urls.count > 1 {
            // Pre-filter with partial hash (first 4KB + last 4KB + size)
            var partialMap: [String: [URL]] = [:]
            for fileURL in urls {
                if let cancelled = isCancelled, cancelled() { break }
                if let partial = partialHash(fileURL) {
                    let key = "\(size):\(partial)"
                    partialMap[key, default: []].append(fileURL)
                }
            }

            // Only full-hash files whose partial hash matches others
            for (_, candidates) in partialMap where candidates.count > 1 {
                for fileURL in candidates {
                    if let cancelled = isCancelled, cancelled() { break }
                    if let hash = sha256Hash(fileURL) {
                        hashMap[hash, default: []].append(fileURL.path)
                        hashSizes[hash] = size
                    }
                }
            }
        }

        // Build groups
        var groups: [DuplicateGroup] = []
        var totalWasted: UInt64 = 0

        for (hash, files) in hashMap where files.count > 1 {
            let size = hashSizes[hash] ?? 0
            let wasted = size * UInt64(files.count - 1)
            totalWasted += wasted
            groups.append(DuplicateGroup(hash: hash, size: size, files: files))
        }

        groups.sort { ($0.size * UInt64($0.files.count - 1)) > ($1.size * UInt64($1.files.count - 1)) }

        return DuplicateScanResult(groups: groups, totalWasted: totalWasted, filesScanned: filesScanned)
    }

    /// Partial hash: first 4KB + last 4KB of file (much cheaper than full SHA256)
    private static func partialHash(_ url: URL) -> String? {
        guard let handle = try? FileHandle(forReadingFrom: url),
              let size = try? url.resourceValues(forKeys: [.fileSizeKey]).fileSize,
              size > 8192 else { return nil }
        defer { handle.closeFile() }

        let head = try? handle.read(upToCount: 4096)
        try? handle.seek(toOffset: UInt64(size) - 4096)
        let tail = try? handle.read(upToCount: 4096)

        guard let h = head, let t = tail else { return nil }

        var hasher = SHA256()
        hasher.update(data: h)
        hasher.update(data: t)
        let digest = hasher.finalize()
        return digest.prefix(8).map { String(format: "%02x", $0) }.joined()
    }

    private static func sha256Hash(_ url: URL) -> String? {
        guard let handle = try? FileHandle(forReadingFrom: url) else { return nil }
        defer { handle.closeFile() }

        var hasher = SHA256()
        let bufferSize = 65536

        while true {
            let data = handle.readData(ofLength: bufferSize)
            if data.isEmpty { break }
            hasher.update(data: data)
        }

        let digest = hasher.finalize()
        return digest.map { String(format: "%02x", $0) }.joined()
    }
}
```

- [ ] **Step 2: Update DuplicatesViewModel to pass cancellation**

```swift
import Foundation

@MainActor
final class DuplicatesViewModel: ObservableObject {
    @Published private(set) var result: DuplicateScanResult?
    @Published private(set) var isLoading = false
    @Published private(set) var currentDirectory: String?
    @Published var searchPath: String

    private var scanTask: Task<Void, Never>?

    init() {
        searchPath = FileManager.default.homeDirectoryForCurrentUser.path
    }

    func scan() async {
        // Cancel any in-progress scan
        scanTask?.cancel()
        isLoading = true
        result = nil
        currentDirectory = nil
        let path = searchPath

        scanTask = Task {
            let scanned = await Task.detached {
                DuplicateService.scanDuplicates(
                    path: path,
                    isCancelled: { Task.isCancelled },
                    progress: { dir in
                        Task { @MainActor in
                            self.currentDirectory = dir
                        }
                    }
                )
            }.value

            if !Task.isCancelled {
                result = scanned
            }
            isLoading = false
        }
    }

    func cancelScan() {
        scanTask?.cancel()
    }
}
```

- [ ] **Step 3: Build to verify**

Run: `cd /Users/me/Documents/MMAC/SysMac && swift build 2>&1 | head -20`

- [ ] **Step 4: Commit**

```bash
git add SysMac/SysMac/Services/DuplicateService.swift SysMac/SysMac/ViewModels/DuplicatesViewModel.swift
git commit -m "perf: add cancellation, partial hash pre-filter, and max file size to DuplicateService"
```

---

### Task 5: Bound LargeFilesService with in-place top-N tracking

**Files:**
- Modify: `SysMac/SysMac/Services/LargeFilesService.swift:3-36`

- [ ] **Step 1: Replace unbounded array with in-place top-N**

```swift
import Foundation

enum LargeFilesService {
    static func findLargeFiles(path: String, minSize: UInt64 = 50 * 1024 * 1024, limit: Int = 100) -> [LargeFile] {
        let url = URL(fileURLWithPath: path)
        let fm = FileManager.default

        guard let enumerator = fm.enumerator(at: url, includingPropertiesForKeys: [.fileSizeKey, .contentModificationDateKey, .isRegularFileKey], options: []) else {
            return []
        }

        var topFiles: [LargeFile] = []

        for case let fileURL as URL in enumerator {
            guard let values = try? fileURL.resourceValues(forKeys: [.fileSizeKey, .contentModificationDateKey, .isRegularFileKey]),
                  values.isRegularFile == true,
                  let size = values.fileSize,
                  UInt64(size) >= minSize else { continue }

            let modified = values.contentModificationDate?.unixTimestamp ?? 0
            let file = LargeFile(
                path: fileURL.path,
                name: fileURL.lastPathComponent,
                size: UInt64(size),
                modified: UInt64(modified)
            )

            if topFiles.count < limit {
                topFiles.append(file)
                if topFiles.count == limit {
                    topFiles.sort { $0.size > $1.size }
                }
            } else if size > topFiles.last!.size {
                topFiles[topFiles.count - 1] = file
                topFiles.sort { $0.size > $1.size }
            }
        }

        return topFiles
    }
}
```

- [ ] **Step 2: Build to verify**

Run: `cd /Users/me/Documents/MMAC/SysMac && swift build 2>&1 | head -20`

- [ ] **Step 3: Commit**

```bash
git add SysMac/SysMac/Services/LargeFilesService.swift
git commit -m "perf: bound LargeFilesService with in-place top-N tracking"
```

---

### Task 6: Add cheap pre-check before directorySize in OrphanedService

**Files:**
- Modify: `SysMac/SysMac/Services/OrphanedService.swift:44-56`

- [ ] **Step 1: Add directory size pre-check before expensive recursion**

Replace lines 54-56 in `scanOrphanedFiles()`:

```swift
                // Calculate size - check top-level metadata first to avoid expensive recursion for small dirs
                if let topValues = try? item.resourceValues(forKeys: [.totalFileSizeKey]),
                   let topSize = topValues.totalFileSize, topSize < 1_048_576 {
                    continue
                }

                let size = FileUtilities.directorySize(at: item)
                guard size >= 1_048_576 else { continue } // 1MB minimum
```

> **Note:** `totalFileSizeKey` returns the apparent total file size for directories on APFS/HFS+. It is not recursive but provides a cheap upper-bound heuristic. If `totalFileSizeKey` is not available (returns nil), fall through to the full `directorySize()` call as before.

- [ ] **Step 2: Build to verify**

Run: `cd /Users/me/Documents/MMAC/SysMac && swift build 2>&1 | head -20`

- [ ] **Step 3: Commit**

```bash
git add SysMac/SysMac/Services/OrphanedService.swift
git commit -m "perf: skip expensive directorySize for small dirs in OrphanedService"
```

---

### Task 7: Make CleaningService scanAll concurrent

**Files:**
- Modify: `SysMac/SysMac/Services/CleaningService.swift:3-15`
- Modify: `SysMac/SysMac/ViewModels/CleaningViewModel.swift:14-19`

- [ ] **Step 1: Make `scanAll()` async with TaskGroup concurrency**

Change `CleaningService.scanAll()` signature and implementation:

```swift
    static func scanAll() async -> [ScanResult] {
        let home = FileManager.default.homeDirectoryForCurrentUser
        let categories: [(String, URL)] -> ScanResult = { label, urls in
            self.scanCategory(label, displayPaths: urls)
        }

        return await withTaskGroup(of: ScanResult.self) { group in
            group.addTask {
                await Task.detached { Self.scanCaches(home) }.value
            }
            group.addTask {
                await Task.detached { Self.scanLogs(home) }.value
            }
            group.addTask {
                await Task.detached { Self.scanBrowserData(home) }.value
            }
            group.addTask {
                await Task.detached { Self.scanTrash(home) }.value
            }
            group.addTask {
                await Task.detached { Self.scanCrashReports(home) }.value
            }
            group.addTask {
                await Task.detached { Self.scanXcodeData(home) }.value
            }
            group.addTask {
                await Task.detached { Self.scanPackageCaches(home) }.value
            }

            var results: [ScanResult] = []
            for await result in group {
                results.append(result)
            }
            return results
        }
    }
```

- [ ] **Step 2: Update CleaningViewModel to await async scanAll**

Change `scan()` in CleaningViewModel:

```swift
    func scan() async {
        isScanning = true
        error = nil
        let scanned = await CleaningService.scanAll()
        results = scanned
        isScanning = false
    }
```

- [ ] **Step 3: Build to verify**

Run: `cd /Users/me/Documents/MMAC/SysMac && swift build 2>&1 | head -20`

- [ ] **Step 4: Commit**

```bash
git add SysMac/SysMac/Services/CleaningService.swift SysMac/SysMac/ViewModels/CleaningViewModel.swift
git commit -m "perf: run CleaningService scans concurrently with TaskGroup"
```

---

### Task 8: Optimize MonitorService subprocess calls

**Files:**
- Modify: `SysMac/SysMac/Services/MonitorService.swift:120-192`

- [ ] **Step 1: Cache fan speed and reduce iostat blocking**

Add fan speed caching and replace blocking `iostat -c 1` with a shorter approach:

```swift
actor MonitorService {
    // ... existing properties ...
    private var cachedFanSpeed: UInt32?
    private var lastFanCheck: Date?
    private static let fanCheckInterval: TimeInterval = 10 // Check fan every 10s

    // ... getStats() unchanged ...

    // MARK: - Fan Speed (cached)

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

    // MARK: - Disk I/O (non-blocking)

    private func getDiskIOSpeed() -> (read: UInt64, write: UInt64) {
        // Use a single-sample iostat read instead of blocking -c 1
        let result = ShellHelper.run("/usr/sbin/iostat", arguments: ["-d", "-c", "2"], timeout: 5)
        guard result.exitCode == 0 else { return (0, 0) }

        let lines = result.output.components(separatedBy: "\n")
        // -c 2 gives 2 snapshots; we want the delta from the second one
        // Look for the second data line after headers
        let dataLines = lines.filter { line in
            let trimmed = line.trimmingCharacters(in: .whitespaces)
            return !trimmed.isEmpty && !trimmed.hasPrefix("disk") && !trimmed.contains(" KB/t")
        }

        guard dataLines.count >= 2 else { return (0, 0) }

        let secondLine = dataLines[1]
        let parts = secondLine.split(separator: " ", omittingEmptySubsequences: true)
        guard parts.count >= 3, let mbPerSec = Double(parts[2]) else { return (0, 0) }

        let bytesPerSec = UInt64(mbPerSec * 1024.0 * 1024.0)
        return (bytesPerSec / 2, bytesPerSec / 2)
    }
}
```

- [ ] **Step 2: Build to verify**

Run: `cd /Users/me/Documents/MMAC/SysMac && swift build 2>&1 | head -20`

- [ ] **Step 3: Commit**

```bash
git add SysMac/SysMac/Services/MonitorService.swift
git commit -m "perf: cache fan speed and reduce iostat blocking in MonitorService"
```

---

### Task 9: Reduce Battery and Bluetooth polling frequency

**Files:**
- Modify: `SysMac/SysMac/ViewModels/BatteryViewModel.swift:17`
- Modify: `SysMac/SysMac/ViewModels/BluetoothViewModel.swift:17`

- [ ] **Step 1: Change BatteryViewModel polling from 5s to 30s**

In `BatteryViewModel.swift`, change line 17:

```swift
                try? await Task.sleep(nanoseconds: 30_000_000_000)
```

- [ ] **Step 2: Change BluetoothViewModel polling from 5s to 10s**

In `BluetoothViewModel.swift`, change line 17:

```swift
                try? await Task.sleep(nanoseconds: 10_000_000_000)
```

- [ ] **Step 3: Build to verify**

Run: `cd /Users/me/Documents/MMAC/SysMac && swift build 2>&1 | head -20`

- [ ] **Step 4: Commit**

```bash
git add SysMac/SysMac/ViewModels/BatteryViewModel.swift SysMac/SysMac/ViewModels/BluetoothViewModel.swift
git commit -m "perf: reduce battery polling to 30s and bluetooth to 10s"
```

---

### Task 10: Fix silent error swallowing and move deletion to Service layer

**Files:**
- Modify: `SysMac/SysMac/ViewModels/OrphanedViewModel.swift:16-30`
- Modify: `SysMac/SysMac/ViewModels/LargeFilesViewModel.swift:25-32`
- Modify: `SysMac/SysMac/Services/CleaningService.swift:17-34`
- Modify: `SysMac/SysMac/Services/OrphanedService.swift` (add deleteFiles method)
- Modify: `SysMac/SysMac/Services/LargeFilesService.swift` (add deleteFile method)

- [ ] **Step 1: Add `deleteFiles` to OrphanedService with PathValidator**

Append to `OrphanedService`:

```swift
    static func deleteFiles(paths: Set<String>, moveToTrash: Bool) -> (deleted: Int, failed: [String]) {
        let fm = FileManager.default
        var deleted = 0
        var failed: [String] = []

        for path in paths {
            guard case .success(let validatedURL) = PathValidator.validateForDeletion(path) else {
                failed.append(path)
                continue
            }
            do {
                if moveToTrash {
                    try fm.trashItem(at: validatedURL, resultingItemURL: nil)
                } else {
                    try fm.removeItem(at: validatedURL)
                }
                deleted += 1
            } catch {
                failed.append(path)
            }
        }

        return (deleted, failed)
    }
```

- [ ] **Step 2: Add `deleteFile` to LargeFilesService with PathValidator**

Append to `LargeFilesService`:

```swift
    static func deleteFile(path: String, moveToTrash: Bool) -> Result<Void, ServiceError> {
        guard case .success(let validatedURL) = PathValidator.validateForDeletion(path) else {
            return .failure(ServiceError("Ruta no permitida: \(path)"))
        }
        let fm = FileManager.default
        do {
            if moveToTrash {
                try fm.trashItem(at: validatedURL, resultingItemURL: nil)
            } else {
                try fm.removeItem(at: validatedURL)
            }
            return .success(())
        } catch {
            return .failure(ServiceError("Error al eliminar: \(error.localizedDescription)"))
        }
    }
```

- [ ] **Step 3: Update OrphanedViewModel to delegate to service and report errors**

Replace `OrphanedViewModel`:

```swift
import Foundation

@MainActor
final class OrphanedViewModel: ObservableObject {
    @Published private(set) var result: OrphanedScanResult?
    @Published private(set) var isLoading = false
    @Published private(set) var error: String?
    @Published var selectedPaths: Set<String> = []

    func scan() async {
        isLoading = true
        error = nil
        let scanned = await Task.detached { OrphanedService.scanOrphanedFiles() }.value
        result = scanned
        isLoading = false
    }

    func deleteSelected(moveToTrash: Bool) {
        let paths = selectedPaths
        let (deleted, failed) = await Task.detached {
            OrphanedService.deleteFiles(paths: paths, moveToTrash: moveToTrash)
        }.value

        if !failed.isEmpty {
            error = "\(failed.count) elementos no se pudieron eliminar"
        }
        selectedPaths = selectedPaths.filter { !failed.contains($0) }
        Task { await scan() }
    }
}
```

- [ ] **Step 4: Update LargeFilesViewModel to delegate to service and report errors**

Replace `LargeFilesViewModel`:

```swift
import Foundation

@MainActor
final class LargeFilesViewModel: ObservableObject {
    @Published private(set) var files: [LargeFile] = []
    @Published private(set) var isLoading = false
    @Published private(set) var error: String?
    @Published var minSizeMB: Double = 50
    @Published var searchPath: String

    init() {
        searchPath = FileManager.default.homeDirectoryForCurrentUser.path
    }

    func scan() async {
        isLoading = true
        error = nil
        let minBytes = UInt64(minSizeMB * 1024 * 1024)
        let path = searchPath
        let found = await Task.detached { LargeFilesService.findLargeFiles(path: path, minSize: minBytes) }.value
        files = found
        isLoading = false
    }

    var totalSize: UInt64 { files.reduce(0) { $0 + $1.size } }

    func deleteFile(_ file: LargeFile) {
        let result = LargeFilesService.deleteFile(path: file.path, moveToTrash: true)
        if case .failure(let err) = result {
            error = err.message
        } else {
            files.removeAll { $0.id == file.id }
        }
    }
}
```

- [ ] **Step 5: Fix CleaningService.cleanCategory to report failures**

Replace `cleanCategory` method:

```swift
    static func cleanCategory(_ category: String, paths: [String], moveToTrash: Bool) -> (freed: UInt64, failed: Int) {
        var freed: UInt64 = 0
        var failedCount = 0
        let fm = FileManager.default
        for path in paths {
            guard fm.fileExists(atPath: path) else { continue }
            guard case .success(let validatedURL) = PathValidator.validateForDeletion(path) else { failedCount += 1; continue }
            let size = FileUtilities.directorySize(at: validatedURL)
            do {
                if moveToTrash && category != "trash" {
                    try fm.trashItem(at: validatedURL, resultingItemURL: nil)
                } else {
                    try fm.removeItem(at: validatedURL)
                }
                freed += size
            } catch {
                failedCount += 1
            }
        }
        return (freed, failedCount)
    }
```

- [ ] **Step 6: Update CleaningViewModel to use new return type**

Update `clean()` in CleaningViewModel:

```swift
    func clean(moveToTrash: Bool) async {
        isCleaning = true
        error = nil
        let toClean = results.filter { selectedCategories.contains($0.category) }
        let trash = moveToTrash
        var totalFailed = 0
        await Task.detached {
            for item in toClean {
                let result = CleaningService.cleanCategory(item.category, paths: item.paths, moveToTrash: trash)
                totalFailed += result.failed
            }
        }.value
        if totalFailed > 0 {
            error = "\(totalFailed) elementos no se pudieron eliminar"
        }
        selectedCategories.removeAll()
        isCleaning = false
        await scan()
    }
```

- [ ] **Step 7: Build to verify**

Run: `cd /Users/me/Documents/MMAC/SysMac && swift build 2>&1 | head -20`

- [ ] **Step 8: Commit**

```bash
git add SysMac/SysMac/ViewModels/OrphanedViewModel.swift SysMac/SysMac/ViewModels/LargeFilesViewModel.swift SysMac/SysMac/Services/CleaningService.swift SysMac/SysMac/ViewModels/CleaningViewModel.swift SysMac/SysMac/Services/OrphanedService.swift SysMac/SysMac/Services/LargeFilesService.swift
git commit -m "fix: report deletion errors and move file operations to Service layer with PathValidator"
```

---

### Task 11: Harden ShellHelper and replace shell() callers

**Files:**
- Modify: `SysMac/SysMac/Services/ShellHelper.swift:53-57`
- Modify: `SysMac/SysMac/Services/NetworkService.swift:78-83` (flushDNS)
- Modify: `SysMac/SysMac/Services/FirewallService.swift:4-12` (getFirewallStatus)

- [ ] **Step 1: Rename `shell()` to `shellRaw()` and add deprecation doc**

In ShellHelper.swift, replace lines 53-57:

```swift
    /// Run a command using /bin/sh -c for shell features (pipes, redirects).
    /// WARNING: Only use with hardcoded constant strings. Never pass user-controlled input.
    /// Prefer `run()` with explicit arguments for safer execution.
    @discardableResult
    static func shellRaw(_ command: String) -> (output: String, error: String, exitCode: Int32) {
        run("/bin/sh", arguments: ["-c", command])
    }

    /// Deprecated alias - use shellRaw() to make the risk explicit
    @available(*, deprecated, renamed: "shellRaw")
    @discardableResult
    static func shell(_ command: String) -> (output: String, error: String, exitCode: Int32) {
        shellRaw(command)
    }
```

- [ ] **Step 2: Replace NetworkService.flushDNS shell() with run()**

In NetworkService.swift, replace flushDNS:

```swift
    static func flushDNS() -> Result<Void, ServiceError> {
        let result = ShellHelper.run("/usr/bin/dscacheutil", arguments: ["-flushcache"])
        if result.exitCode == 0 {
            return .success(())
        }
        return .failure(ServiceError("Error al limpiar DNS: \(result.error)"))
    }
```

- [ ] **Step 3: Replace FirewallService.getFirewallStatus shell() calls with run()**

In FirewallService.swift, replace getFirewallStatus:

```swift
    static func getFirewallStatus() -> FirewallStatus {
        let result = ShellHelper.run("/usr/bin/defaults", arguments: ["read", "/Library/Preferences/com.apple.alf", "globalstate"])
        let enabled = result.output.trimmingCharacters(in: .whitespacesAndNewlines) != "0"

        let stealthResult = ShellHelper.run("/usr/bin/defaults", arguments: ["read", "/Library/Preferences/com.apple.alf", "stealthenabled"])
        let stealth = stealthResult.output.trimmingCharacters(in: .whitespacesAndNewlines) == "1"

        let blockResult = ShellHelper.run("/usr/bin/defaults", arguments: ["read", "/Library/Preferences/com.apple.alf", "allowsignedenabled"])
        let blockAll = blockResult.output.trimmingCharacters(in: .whitespacesAndNewlines) == "0"

        return FirewallStatus(enabled: enabled, stealthMode: stealth, blockAllIncoming: blockAll)
    }
```

- [ ] **Step 4: Build to verify**

Run: `cd /Users/me/Documents/MMAC/SysMac && swift build 2>&1 | head -20`

- [ ] **Step 5: Commit**

```bash
git add SysMac/SysMac/Services/ShellHelper.swift SysMac/SysMac/Services/NetworkService.swift SysMac/SysMac/Services/FirewallService.swift
git commit -m "security: harden ShellHelper, replace shell() with run() where possible"
```

---

## Self-Review Checklist

**1. Spec coverage:**
- Section 1 (Subprocess Batching) -> Tasks 1, 2, 3
- Section 2 (Scanning Safety) -> Tasks 4, 5, 6
- Section 3 (Concurrent Cleaning + Polling) -> Tasks 7, 8, 9
- Section 4 (Data Integrity + Security) -> Tasks 10, 11

**2. Placeholder scan:** No TBDs, no "add error handling", no "similar to Task N". All code is complete.

**3. Type consistency:** `cleanCategory` return type changed from `UInt64` to `(freed: UInt64, failed: Int)` in Task 10 -- CleaningViewModel updated to match. `shell()` renamed to `shellRaw()` in Task 11 -- all callers using `shell()` will still compile due to deprecated wrapper.
