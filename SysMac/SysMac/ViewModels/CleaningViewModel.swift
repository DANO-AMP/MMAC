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
        results = CleaningService.scanAll()
        isScanning = false
    }

    func clean(moveToTrash: Bool) async {
        isCleaning = true
        for result in results where selectedCategories.contains(result.category) {
            _ = CleaningService.cleanCategory(result.category, paths: result.paths, moveToTrash: moveToTrash)
        }
        selectedCategories.removeAll()
        isCleaning = false
        await scan()
    }
}
