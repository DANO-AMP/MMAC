import Foundation

@MainActor
final class OrphanedViewModel: ObservableObject {
    @Published private(set) var result: OrphanedScanResult?
    @Published private(set) var isLoading = false
    @Published private(set) var error: String?
    @Published var selectedPaths: Set<String> = []

    func scan() async {
        isLoading = true
        error = nil
        let scanned = await Task.detached { OrphanedService.scanOrphanedFiles() }.value
        result = scanned
        isLoading = false
    }

    func deleteSelected(moveToTrash: Bool) async {
        let paths = selectedPaths
        let (deleted, failed) = await Task.detached {
            OrphanedService.deleteFiles(paths: paths, moveToTrash: moveToTrash)
        }.value

        if !failed.isEmpty {
            error = "\(failed.count) elementos no se pudieron eliminar"
        }
        selectedPaths = selectedPaths.filter { !failed.contains($0) }
        Task { await scan() }
    }
}
