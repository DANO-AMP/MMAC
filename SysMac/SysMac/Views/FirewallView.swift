import SwiftUI

struct FirewallView: View {
    @StateObject private var vm = FirewallViewModel()

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                HStack {
                    Text("Firewall").font(.title2).fontWeight(.bold)
                    Spacer()
                    if vm.isLoading { ProgressView().scaleEffect(0.7) }
                    Button {
                        Task { await vm.refresh() }
                    } label: {
                        Label("Actualizar", systemImage: "arrow.clockwise")
                    }
                }

                if let status = vm.status {
                    HStack(spacing: 12) {
                        statusCard("Firewall", enabled: status.enabled)
                        statusCard("Modo Stealth", enabled: status.stealthMode)
                        statusCard("Bloquear entrantes", enabled: status.blockAllIncoming)
                    }
                }

                if !vm.processConnections.isEmpty {
                    Text("Conexiones salientes por proceso (\(vm.processConnections.count))")
                        .font(.headline)

                    ForEach(vm.processConnections) { proc in
                        DisclosureGroup {
                            ForEach(proc.connections) { conn in
                                HStack {
                                    Text("\(conn.remoteHost):\(conn.remotePort)")
                                        .font(.caption.monospaced())
                                    Spacer()
                                    Text(conn.connectionState)
                                        .font(.caption)
                                        .foregroundStyle(.secondary)
                                }
                                .padding(.vertical, 2)
                            }
                        } label: {
                            HStack {
                                Text(proc.processName).fontWeight(.medium)
                                Text("PID: \(proc.pid)")
                                    .font(.caption).foregroundStyle(.secondary)
                                Spacer()
                                Text("\(proc.connectionCount) conexiones")
                                    .font(.caption).foregroundStyle(.secondary)
                            }
                        }
                        .padding(8)
                        .background(.quaternary.opacity(0.3))
                        .cornerRadius(8)
                    }
                }
            }
            .padding()
        }
        .navigationTitle("Firewall")
        .task { await vm.refresh() }
    }

    private func statusCard(_ title: String, enabled: Bool) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(title).font(.caption).foregroundStyle(.secondary)
            HStack(spacing: 6) {
                Circle().fill(enabled ? Color.green : Color.red).frame(width: 8, height: 8)
                Text(enabled ? "Activado" : "Desactivado")
                    .fontWeight(.semibold)
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .cardStyle()
    }
}
