import Foundation

@MainActor
final class MemoryViewModel: ObservableObject {
    @Published private(set) var memoryInfo: MemoryInfo?
    @Published private(set) var isLoading = false
    @Published private(set) var error: String?

    private var pollTask: Task<Void, Never>?

    func startPolling() {
        guard pollTask == nil else { return }
        pollTask = Task { [weak self] in
            while !Task.isCancelled {
                await self?.refresh()
                try? await Task.sleep(nanoseconds: 2_000_000_000)
            }
        }
    }

    func stopPolling() {
        pollTask?.cancel()
        pollTask = nil
    }

    func refresh() async {
        isLoading = true
        memoryInfo = MemoryService.getMemoryInfo()
        isLoading = false
    }

    func purge() {
        switch MemoryService.purgeMemory() {
        case .success:
            error = nil
        case .failure(let err):
            error = err.message
        }
    }
}
