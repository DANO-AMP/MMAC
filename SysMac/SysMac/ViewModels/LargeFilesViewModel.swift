import Foundation

@MainActor
final class LargeFilesViewModel: ObservableObject {
    @Published private(set) var files: [LargeFile] = []
    @Published private(set) var isLoading = false
    @Published private(set) var error: String?
    @Published var minSizeMB: Double = 50
    @Published var searchPath: String

    init() {
        searchPath = FileManager.default.homeDirectoryForCurrentUser.path
    }

    func scan() async {
        isLoading = true
        error = nil
        let minBytes = UInt64(minSizeMB * 1024 * 1024)
        let path = searchPath
        let found = await Task.detached { LargeFilesService.findLargeFiles(path: path, minSize: minBytes) }.value
        files = found
        isLoading = false
    }

    var totalSize: UInt64 { files.reduce(0) { $0 + $1.size } }

    func deleteFile(_ file: LargeFile) {
        let result = LargeFilesService.deleteFile(path: file.path, moveToTrash: true)
        if case .failure(let err) = result {
            error = err.message
        } else {
            files.removeAll { $0.id == file.id }
        }
    }
}
