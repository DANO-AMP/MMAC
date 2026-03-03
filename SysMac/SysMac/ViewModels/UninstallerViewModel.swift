import Foundation

@MainActor
final class UninstallerViewModel: ObservableObject {
    @Published private(set) var apps: [AppInfo] = []
    @Published private(set) var isLoading = false
    @Published private(set) var error: String?
    @Published var searchText = ""

    var filteredApps: [AppInfo] {
        guard !searchText.isEmpty else { return apps }
        let query = searchText.lowercased()
        return apps.filter {
            $0.name.lowercased().contains(query) ||
            $0.bundleId.lowercased().contains(query)
        }
    }

    func loadApps() async {
        isLoading = true
        let loaded = await Task.detached { UninstallerService.listApps() }.value
        apps = loaded
        isLoading = false
    }

    func uninstall(app: AppInfo, moveToTrash: Bool) {
        let remnantPaths = app.remnants.map(\.path)
        let appPath = app.path
        let trash = moveToTrash
        Task {
            let result = await Task.detached {
                UninstallerService.uninstallApp(path: appPath, remnantPaths: remnantPaths, moveToTrash: trash)
            }.value
            switch result {
            case .success:
                apps.removeAll { $0.path == appPath }
            case .failure(let err):
                error = err.message
            }
        }
    }
}
