import Foundation

@MainActor
final class FirewallViewModel: ObservableObject {
    @Published private(set) var status: FirewallStatus?
    @Published private(set) var processConnections: [ProcessConnections] = []
    @Published private(set) var isLoading = false

    func refresh() async {
        isLoading = true
        let (newStatus, newConnections) = await Task.detached {
            (FirewallService.getFirewallStatus(), FirewallService.getOutgoingConnections())
        }.value
        status = newStatus
        processConnections = newConnections
        isLoading = false
    }
}
