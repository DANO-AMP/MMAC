import Foundation

@MainActor
final class ConnectionsViewModel: ObservableObject {
    @Published private(set) var connections: [NetworkConnection] = []
    @Published private(set) var isLoading = false
    @Published var searchText = ""

    private var pollTask: Task<Void, Never>?

    var filteredConnections: [NetworkConnection] {
        guard !searchText.isEmpty else { return connections }
        let q = searchText.lowercased()
        return connections.filter {
            $0.processName.lowercased().contains(q) ||
            $0.localAddress.contains(q) ||
            $0.remoteAddress.contains(q) ||
            String($0.localPort).contains(q)
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
        connections = NetworkService.getConnections()
        isLoading = false
    }
}
