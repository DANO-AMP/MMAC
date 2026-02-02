import Foundation

struct ChartDataPoint: Identifiable {
    let id = UUID()
    let time: String
    let cpu: Double
    let memory: Double
    let networkRx: UInt64
    let networkTx: UInt64
}

@MainActor
final class MonitorViewModel: ObservableObject {
    @Published private(set) var stats: SystemStats?
    @Published private(set) var isLoading = false
    @Published private(set) var error: String?
    @Published private(set) var chartData: [ChartDataPoint] = []
    @Published var isLive = true

    private var pollTask: Task<Void, Never>?
    private let service = MonitorService()

    func startPolling() {
        guard pollTask == nil else { return }
        pollTask = Task { [weak self] in
            while !Task.isCancelled {
                await self?.refresh()
                try? await Task.sleep(nanoseconds: 2_000_000_000) // 2s
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

        let newStats = await service.getStats()
        stats = newStats

        if isLive {
            let formatter = DateFormatter()
            formatter.dateFormat = "HH:mm:ss"
            let timeStr = formatter.string(from: Date())

            let memPercent: Double
            if newStats.memoryTotal > 0 {
                memPercent = Double(newStats.memoryUsed) / Double(newStats.memoryTotal) * 100.0
            } else {
                memPercent = 0
            }

            let point = ChartDataPoint(
                time: timeStr,
                cpu: Double(newStats.cpuUsage),
                memory: memPercent,
                networkRx: newStats.networkRx,
                networkTx: newStats.networkTx
            )

            chartData.append(point)
            if chartData.count > 30 {
                chartData.removeFirst(chartData.count - 30)
            }
        }

        isLoading = false
    }
}
