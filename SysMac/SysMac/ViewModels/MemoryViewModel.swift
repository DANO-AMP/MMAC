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
                guard let self else { break }
                await self.refresh()
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
        let info = await Task.detached { MemoryService.getMemoryInfo() }.value
        memoryInfo = info
        isLoading = false
    }

    func purge() {
        Task {
            let result = await Task.detached { MemoryService.purgeMemory() }.value
            switch result {
            case .success:
                error = nil
            case .failure(let err):
                error = err.message
            }
        }
    }
}
