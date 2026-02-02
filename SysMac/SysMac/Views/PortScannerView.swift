import SwiftUI

struct PortScannerView: View {
    @StateObject private var vm = PortScannerViewModel()

    var body: some View {
        VStack(spacing: 0) {
            HStack {
                SearchBar(text: $vm.searchText, placeholder: "Buscar puertos...")
                Spacer()
                Text("\(vm.filteredPorts.count) puertos abiertos")
                    .font(.caption).foregroundStyle(.secondary)
            }
            .padding()

            Table(vm.filteredPorts) {
                TableColumn("Puerto") { port in
                    Text("\(port.port)")
                        .font(.caption.monospaced())
                        .fontWeight(.semibold)
                }.width(60)

                TableColumn("Proceso") { port in
                    Text(port.processName).lineLimit(1)
                }.width(min: 100, ideal: 140)

                TableColumn("Tipo") { port in
                    Text(port.serviceType)
                        .font(.caption)
                        .padding(.horizontal, 6).padding(.vertical, 2)
                        .background(Color.blue.opacity(0.15))
                        .cornerRadius(4)
                }.width(100)

                TableColumn("Proto") { port in
                    Text(port.proto).font(.caption.monospaced())
                }.width(50)

                TableColumn("CPU %") { port in
                    Text(String(format: "%.1f", port.cpuUsage)).font(.caption.monospaced())
                }.width(55)

                TableColumn("Mem (MB)") { port in
                    Text(String(format: "%.1f", port.memoryMB)).font(.caption.monospaced())
                }.width(65)

                TableColumn("PID") { port in
                    Text("\(port.pid)").font(.caption.monospaced())
                }.width(55)
            }
        }
        .navigationTitle("Puertos")
        .onAppear { vm.startPolling() }
        .onDisappear { vm.stopPolling() }
    }
}
