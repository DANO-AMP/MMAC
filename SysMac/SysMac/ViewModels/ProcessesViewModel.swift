import Foundation

@MainActor
final class ProcessesViewModel: ObservableObject {
    @Published private(set) var processes: [ProcessItem] = []
    @Published private(set) var isLoading = false
    @Published private(set) var error: String?
    @Published var searchText = ""
    @Published var sortBy: SortField = .cpu
    @Published var sortAscending = false

    enum SortField: String, CaseIterable {
        case cpu = "CPU"
        case memory = "Memoria"
        case name = "Nombre"
        case pid = "PID"
    }

    private var pollTask: Task<Void, Never>?

    var filteredProcesses: [ProcessItem] {
        var result = processes
        if !searchText.isEmpty {
            let query = searchText.lowercased()
            result = result.filter {
                $0.name.lowercased().contains(query) ||
                $0.user.lowercased().contains(query) ||
                String($0.pid).contains(query)
            }
        }
        result.sort { a, b in
            let cmp: Bool
            switch sortBy {
            case .cpu: cmp = a.cpuUsage > b.cpuUsage
            case .memory: cmp = a.memoryMB > b.memoryMB
            case .name: cmp = a.name.lowercased() < b.name.lowercased()
            case .pid: cmp = a.pid < b.pid
            }
            return sortAscending ? !cmp : cmp
        }
        return result
    }

    func startPolling() {
        guard pollTask == nil else { return }
        pollTask = Task { [weak self] in
            while !Task.isCancelled {
                await self?.refresh()
                try? await Task.sleep(nanoseconds: 3_000_000_000)
            }
        }
    }

    func stopPolling() {
        pollTask?.cancel()
        pollTask = nil
    }

    func refresh() async {
        isLoading = true
        processes = ProcessService.getAllProcesses()
        isLoading = false
    }

    func killProcess(pid: UInt32, force: Bool) {
        switch ProcessService.killProcess(pid: pid, force: force) {
        case .success:
            processes.removeAll { $0.pid == pid }
        case .failure(let err):
            error = err.message
        }
    }
}
