import Foundation

@MainActor
final class DuplicatesViewModel: ObservableObject {
    @Published private(set) var result: DuplicateScanResult?
    @Published private(set) var isLoading = false
    @Published private(set) var currentDirectory: String?
    @Published var searchPath: String

    private var scanTask: Task<Void, Never>?

    init() {
        searchPath = FileManager.default.homeDirectoryForCurrentUser.path
    }

    func scan() async {
        scanTask?.cancel()
        isLoading = true
        result = nil
        currentDirectory = nil
        let path = searchPath

        scanTask = Task {
            let scanned = await Task.detached {
                DuplicateService.scanDuplicates(
                    path: path,
                    isCancelled: { Task.isCancelled },
                    progress: { dir in
                        Task { @MainActor in
                            self.currentDirectory = dir
                        }
                    }
                )
            }.value

            if !Task.isCancelled {
                result = scanned
            }
            isLoading = false
        }
    }

    func cancelScan() {
        scanTask?.cancel()
    }
}
