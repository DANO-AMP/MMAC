import Foundation

@MainActor
final class AnalyzerViewModel: ObservableObject {
    @Published private(set) var items: [DiskItem] = []
    @Published private(set) var isLoading = false
    @Published var currentPath: String

    init() {
        currentPath = FileManager.default.homeDirectoryForCurrentUser.path
    }

    func analyze() async {
        isLoading = true
        items = AnalyzerService.analyze(path: currentPath)
        isLoading = false
    }

    func navigateTo(_ path: String) {
        let home = FileManager.default.homeDirectoryForCurrentUser.path
        currentPath = path.replacingOccurrences(of: "~", with: home)
        Task { await analyze() }
    }

    func goUp() {
        let url = URL(fileURLWithPath: currentPath)
        currentPath = url.deletingLastPathComponent().path
        Task { await analyze() }
    }
}
