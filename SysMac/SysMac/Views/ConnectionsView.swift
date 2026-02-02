import SwiftUI

struct ConnectionsView: View {
    @StateObject private var vm = ConnectionsViewModel()

    var body: some View {
        VStack(spacing: 0) {
            HStack {
                SearchBar(text: $vm.searchText, placeholder: "Buscar conexiones...")
                Spacer()
                Text("\(vm.filteredConnections.count) conexiones")
                    .font(.caption).foregroundStyle(.secondary)
            }
            .padding()

            Table(vm.filteredConnections) {
                TableColumn("Proto") { conn in
                    Text(conn.proto).font(.caption.monospaced())
                }.width(50)

                TableColumn("Local") { conn in
                    Text("\(conn.localAddress):\(conn.localPort)")
                        .font(.caption.monospaced()).lineLimit(1)
                }.width(min: 120, ideal: 160)

                TableColumn("Remoto") { conn in
                    Text("\(conn.remoteAddress):\(conn.remotePort)")
                        .font(.caption.monospaced()).lineLimit(1)
                }.width(min: 120, ideal: 160)

                TableColumn("Estado") { conn in
                    Text(conn.state).font(.caption)
                }.width(90)

                TableColumn("Proceso") { conn in
                    Text(conn.processName).font(.caption)
                }.width(100)

                TableColumn("PID") { conn in
                    Text("\(conn.pid)").font(.caption.monospaced())
                }.width(60)
            }
        }
        .navigationTitle("Conexiones")
        .onAppear { vm.startPolling() }
        .onDisappear { vm.stopPolling() }
    }
}
