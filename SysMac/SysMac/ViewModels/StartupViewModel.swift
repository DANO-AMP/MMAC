import Foundation

@MainActor
final class StartupViewModel: ObservableObject {
    @Published private(set) var items: [StartupItem] = []
    @Published private(set) var isLoading = false
    @Published private(set) var error: String?

    func load() async {
        isLoading = true
        let loaded = await Task.detached { StartupService.getStartupItems() }.value
        items = loaded
        isLoading = false
    }

    func toggle(item: StartupItem) {
        guard !item.path.isEmpty else { return }
        let path = item.path
        let enable = !item.enabled
        Task {
            let result = await Task.detached { StartupService.toggleStartupItem(path: path, enable: enable) }.value
            switch result {
            case .success: await load()
            case .failure(let err): error = err.message
            }
        }
    }
}
