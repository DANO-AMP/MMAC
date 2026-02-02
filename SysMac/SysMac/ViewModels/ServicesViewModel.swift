import Foundation

@MainActor
final class ServicesViewModel: ObservableObject {
    @Published private(set) var result: ServicesResult?
    @Published private(set) var isLoading = false
    @Published private(set) var error: String?
    @Published var searchText = ""

    func load() async {
        isLoading = true
        result = DaemonsService.listServices()
        isLoading = false
    }

    func startService(label: String) {
        switch DaemonsService.startService(label: label) {
        case .success: Task { await load() }
        case .failure(let err): error = err.message
        }
    }

    func stopService(label: String) {
        switch DaemonsService.stopService(label: label) {
        case .success: Task { await load() }
        case .failure(let err): error = err.message
        }
    }
}
