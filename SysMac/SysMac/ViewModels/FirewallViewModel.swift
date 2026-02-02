import Foundation

@MainActor
final class FirewallViewModel: ObservableObject {
    @Published private(set) var status: FirewallStatus?
    @Published private(set) var processConnections: [ProcessConnections] = []
    @Published private(set) var isLoading = false

    func refresh() async {
        isLoading = true
        status = FirewallService.getFirewallStatus()
        processConnections = FirewallService.getOutgoingConnections()
        isLoading = false
    }
}
