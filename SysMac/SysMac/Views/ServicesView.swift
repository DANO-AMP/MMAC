import SwiftUI

struct ServicesView: View {
    @StateObject private var vm = ServicesViewModel()

    var body: some View {
        VStack(spacing: 0) {
            HStack {
                Text("Servicios del Sistema").font(.title2).fontWeight(.bold)
                Spacer()
                SearchBar(text: $vm.searchText, placeholder: "Buscar servicios...")
                    .frame(maxWidth: 250)
                if vm.isLoading { ProgressView().scaleEffect(0.6) }
            }
            .padding()

            if let result = vm.result {
                List {
                    serviceSection("User Agents", services: filtered(result.userAgents))
                    serviceSection("User Daemons", services: filtered(result.userDaemons))
                    serviceSection("System Agents", services: filtered(result.systemAgents))
                }
            }

            if let error = vm.error {
                ErrorBanner(message: error).padding()
            }
        }
        .navigationTitle("Servicios")
        .task { await vm.load() }
    }

    private func filtered(_ services: [LaunchService]) -> [LaunchService] {
        guard !vm.searchText.isEmpty else { return services }
        let q = vm.searchText.lowercased()
        return services.filter { $0.label.lowercased().contains(q) }
    }

    private func serviceSection(_ title: String, services: [LaunchService]) -> some View {
        Section("\(title) (\(services.count))") {
            ForEach(services) { svc in
                HStack {
                    Circle()
                        .fill(svc.status == "running" ? Color.green : svc.status == "error" ? Color.red : Color.gray)
                        .frame(width: 8, height: 8)
                    VStack(alignment: .leading, spacing: 2) {
                        Text(svc.label).lineLimit(1)
                        HStack(spacing: 8) {
                            Text(svc.status).font(.caption).foregroundStyle(.secondary)
                            if let pid = svc.pid { Text("PID: \(pid)").font(.caption).foregroundStyle(.tertiary) }
                        }
                    }
                    Spacer()
                    if svc.status == "running" {
                        Button("Detener") { vm.stopService(label: svc.label) }
                            .controlSize(.small)
                    } else {
                        Button("Iniciar") { vm.startService(label: svc.label) }
                            .controlSize(.small)
                    }
                }
            }
        }
    }
}
