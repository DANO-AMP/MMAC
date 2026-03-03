import Foundation

@MainActor
final class ServicesViewModel: ObservableObject {
    @Published private(set) var result: ServicesResult?
    @Published private(set) var isLoading = false
    @Published private(set) var error: String?
    @Published var searchText = ""

    func load() async {
        isLoading = true
        let services = await Task.detached { DaemonsService.listServices() }.value
        result = services
        isLoading = false
    }

    func startService(label: String) {
        let l = label
        Task {
            let result = await Task.detached { DaemonsService.startService(label: l) }.value
            switch result {
            case .success: await load()
            case .failure(let err): error = err.message
            }
        }
    }

    func stopService(label: String) {
        let l = label
        Task {
            let result = await Task.detached { DaemonsService.stopService(label: l) }.value
            switch result {
            case .success: await load()
            case .failure(let err): error = err.message
            }
        }
    }
}
