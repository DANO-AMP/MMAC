import Foundation

@MainActor
final class CleaningViewModel: ObservableObject {
    @Published private(set) var results: [ScanResult] = []
    @Published private(set) var isScanning = false
    @Published private(set) var isCleaning = false
    @Published private(set) var error: String?
    @Published var selectedCategories: Set<String> = []

    var totalSize: UInt64 { results.reduce(0) { $0 + $1.size } }
    var selectedSize: UInt64 { results.filter { selectedCategories.contains($0.category) }.reduce(0) { $0 + $1.size } }

    func scan() async {
        isScanning = true
        error = nil
        let scanned = await CleaningService.scanAll()
        results = scanned
        isScanning = false
    }

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
}
