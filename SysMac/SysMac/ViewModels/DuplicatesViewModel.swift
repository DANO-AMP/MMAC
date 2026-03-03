import Foundation

@MainActor
final class DuplicatesViewModel: ObservableObject {
    @Published private(set) var result: DuplicateScanResult?
    @Published private(set) var isLoading = false
    @Published var searchPath: String

    init() {
        searchPath = FileManager.default.homeDirectoryForCurrentUser.path
    }

    func scan() async {
        isLoading = true
        let path = searchPath
        let scanned = await Task.detached { DuplicateService.scanDuplicates(path: path) }.value
        result = scanned
        isLoading = false
    }
}
