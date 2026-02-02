import Foundation

@MainActor
final class StartupViewModel: ObservableObject {
    @Published private(set) var items: [StartupItem] = []
    @Published private(set) var isLoading = false
    @Published private(set) var error: String?

    func load() async {
        isLoading = true
        items = StartupService.getStartupItems()
        isLoading = false
    }

    func toggle(item: StartupItem) {
        guard !item.path.isEmpty else { return }
        switch StartupService.toggleStartupItem(path: item.path, enable: !item.enabled) {
        case .success: Task { await load() }
        case .failure(let err): error = err.message
        }
    }
}
