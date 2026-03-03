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
        let scanned = await Task.detached { ProjectsService.scan() }.value
        artifacts = scanned
        isLoading = false
    }

    func deleteSelected(moveToTrash: Bool) {
        let paths = selectedPaths
        let trash = moveToTrash
        Task {
            let results = await Task.detached {
                paths.map { path in
                    (path, ProjectsService.deleteArtifact(path: path, moveToTrash: trash))
                }
            }.value
            for (_, result) in results {
                if case .failure(let err) = result {
                    error = err.message
                }
            }
            selectedPaths.removeAll()
            await scan()
        }
    }
}
