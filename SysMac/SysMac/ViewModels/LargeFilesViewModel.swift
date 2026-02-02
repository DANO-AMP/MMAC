import Foundation

@MainActor
final class LargeFilesViewModel: ObservableObject {
    @Published private(set) var files: [LargeFile] = []
    @Published private(set) var isLoading = false
    @Published var minSizeMB: Double = 50
    @Published var searchPath: String

    init() {
        searchPath = FileManager.default.homeDirectoryForCurrentUser.path
    }

    func scan() async {
        isLoading = true
        let minBytes = UInt64(minSizeMB * 1024 * 1024)
        files = LargeFilesService.findLargeFiles(path: searchPath, minSize: minBytes)
        isLoading = false
    }

    var totalSize: UInt64 { files.reduce(0) { $0 + $1.size } }
}
