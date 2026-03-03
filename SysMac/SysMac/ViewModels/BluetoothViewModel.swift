import Foundation

@MainActor
final class BluetoothViewModel: ObservableObject {
    @Published private(set) var info: BluetoothInfo?
    @Published private(set) var isLoading = false
    @Published private(set) var error: String?

    private var pollTask: Task<Void, Never>?

    func startPolling() {
        guard pollTask == nil else { return }
        pollTask = Task { [weak self] in
            while !Task.isCancelled {
                guard let self else { break }
                await self.refresh()
                try? await Task.sleep(nanoseconds: 5_000_000_000)
            }
        }
    }

    func stopPolling() {
        pollTask?.cancel()
        pollTask = nil
    }

    func refresh() async {
        isLoading = true
        error = nil
        let result = await Task.detached { BluetoothService.getBluetoothInfo() }.value
        switch result {
        case .success(let btInfo):
            info = btInfo
        case .failure(let err):
            error = err.message
        }
        isLoading = false
    }
}
