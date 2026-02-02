import Foundation

@MainActor
final class BatteryViewModel: ObservableObject {
    @Published private(set) var batteryInfo: BatteryInfo?
    @Published private(set) var isLoading = false
    @Published private(set) var noBattery = false

    private var pollTask: Task<Void, Never>?

    func startPolling() {
        guard pollTask == nil else { return }
        pollTask = Task { [weak self] in
            while !Task.isCancelled {
                await self?.refresh()
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
        if let info = BatteryService.getBatteryInfo() {
            batteryInfo = info
            noBattery = false
        } else {
            noBattery = true
        }
        isLoading = false
    }
}
