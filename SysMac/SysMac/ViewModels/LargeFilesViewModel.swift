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
        let path = searchPath
        let found = await Task.detached { LargeFilesService.findLargeFiles(path: path, minSize: minBytes) }.value
        files = found
        isLoading = false
    }

    var totalSize: UInt64 { files.reduce(0) { $0 + $1.size } }

    func deleteFile(_ file: LargeFile) {
        do {
            try FileManager.default.trashItem(at: URL(fileURLWithPath: file.path), resultingItemURL: nil)
            files.removeAll { $0.id == file.id }
        } catch {
            // silently fail - file may be protected
        }
    }
}
