import Foundation

@MainActor
final class ProjectsViewModel: ObservableObject {
    @Published private(set) var artifacts: [ProjectArtifact] = []
    @Published private(set) var isLoading = false
    @Published private(set) var error: String?
    @Published var selectedPaths: Set<String> = []

    var totalSize: UInt64 { artifacts.reduce(0) { $0 + $1.size } }

    func scan() async {
        isLoading = true
        artifacts = ProjectsService.scan()
        isLoading = false
    }

    func deleteSelected(moveToTrash: Bool) {
        for path in selectedPaths {
            switch ProjectsService.deleteArtifact(path: path, moveToTrash: moveToTrash) {
            case .success: break
            case .failure(let err): error = err.message
            }
        }
        selectedPaths.removeAll()
        Task { await scan() }
    }
}
