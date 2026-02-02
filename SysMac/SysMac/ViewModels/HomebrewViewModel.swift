import Foundation

@MainActor
final class HomebrewViewModel: ObservableObject {
    @Published private(set) var info: HomebrewInfo?
    @Published private(set) var packages: [BrewPackage] = []
    @Published private(set) var isLoading = false
    @Published private(set) var error: String?
    @Published var searchText = ""
    @Published var showOutdatedOnly = false

    var filteredPackages: [BrewPackage] {
        var result = packages
        if showOutdatedOnly { result = result.filter(\.isOutdated) }
        if !searchText.isEmpty {
            let q = searchText.lowercased()
            result = result.filter { $0.name.lowercased().contains(q) }
        }
        return result
    }

    func load() async {
        isLoading = true
        error = nil
        info = HomebrewService.checkHomebrew()
        if info?.installed == true {
            packages = HomebrewService.listPackages()
        }
        isLoading = false
    }

    func upgrade(_ name: String) {
        switch HomebrewService.upgradePackage(name) {
        case .success: Task { await load() }
        case .failure(let err): error = err.message
        }
    }

    func uninstall(_ name: String) {
        switch HomebrewService.uninstallPackage(name) {
        case .success: packages.removeAll { $0.name == name }
        case .failure(let err): error = err.message
        }
    }

    func cleanup() {
        switch HomebrewService.cleanup() {
        case .success: break
        case .failure(let err): error = err.message
        }
    }
}
