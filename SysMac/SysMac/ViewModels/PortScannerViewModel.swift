import Foundation

@MainActor
final class PortScannerViewModel: ObservableObject {
    @Published private(set) var ports: [PortInfo] = []
    @Published private(set) var isLoading = false
    @Published var searchText = ""

    private var pollTask: Task<Void, Never>?

    var filteredPorts: [PortInfo] {
        guard !searchText.isEmpty else { return ports }
        let q = searchText.lowercased()
        return ports.filter {
            $0.processName.lowercased().contains(q) ||
            $0.serviceType.lowercased().contains(q) ||
            String($0.port).contains(q)
        }
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

    func stopPolling() { pollTask?.cancel(); pollTask = nil }

    func refresh() async {
        isLoading = true
        ports = PortScannerService.scan()
        isLoading = false
    }
}
