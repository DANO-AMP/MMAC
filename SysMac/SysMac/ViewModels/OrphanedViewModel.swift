import Foundation

@MainActor
final class OrphanedViewModel: ObservableObject {
    @Published private(set) var result: OrphanedScanResult?
    @Published private(set) var isLoading = false
    @Published var selectedPaths: Set<String> = []

    func scan() async {
        isLoading = true
        let scanned = await Task.detached { OrphanedService.scanOrphanedFiles() }.value
        result = scanned
        isLoading = false
    }

    func deleteSelected(moveToTrash: Bool) {
        let fm = FileManager.default
        for path in selectedPaths {
            let url = URL(fileURLWithPath: path)
            do {
                if moveToTrash {
                    try fm.trashItem(at: url, resultingItemURL: nil)
                } else {
                    try fm.removeItem(at: url)
                }
            } catch { /* skip */ }
        }
        selectedPaths.removeAll()
        Task { await scan() }
    }
}
