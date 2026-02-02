import SwiftUI

struct MemoryView: View {
    @StateObject private var vm = MemoryViewModel()

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                // Header
                HStack {
                    VStack(alignment: .leading) {
                        Text("Memoria")
                            .font(.title2)
                            .fontWeight(.bold)
                        Text("Información detallada de la memoria del sistema")
                            .foregroundStyle(.secondary)
                    }
                    Spacer()
                    if vm.isLoading {
                        ProgressView().scaleEffect(0.7)
                    }
                }

                if let mem = vm.memoryInfo {
                    let usedPercent = mem.total > 0 ? Double(mem.used) / Double(mem.total) * 100.0 : 0

                    // Main gauge
                    HStack(spacing: 24) {
                        ZStack {
                            ProgressRing(progress: usedPercent / 100.0, lineWidth: 12, size: 120, color: usedPercent > 80 ? .red : usedPercent > 60 ? .orange : .blue)
                            VStack {
                                Text(Formatters.formatPercentage(usedPercent, decimals: 0))
                                    .font(.title2)
                                    .fontWeight(.bold)
                                Text("Usado")
                                    .font(.caption2)
                                    .foregroundStyle(.secondary)
                            }
                        }

                        VStack(alignment: .leading, spacing: 8) {
                            memRow("Total", value: Formatters.formatSize(mem.total))
                            memRow("Usado", value: Formatters.formatSize(mem.used))
                            memRow("Libre", value: Formatters.formatSize(mem.free))
                        }
                    }
                    .cardStyle()

                    // Breakdown
                    HStack(spacing: 12) {
                        statCard("Activa", value: Formatters.formatSize(mem.active), color: .blue)
                        statCard("Inactiva", value: Formatters.formatSize(mem.inactive), color: .gray)
                        statCard("Wired", value: Formatters.formatSize(mem.wired), color: .orange)
                        statCard("Comprimida", value: Formatters.formatSize(mem.compressed), color: .purple)
                    }

                    HStack(spacing: 12) {
                        statCard("App Memory", value: Formatters.formatSize(mem.appMemory), color: .green)
                        statCard("Cache", value: Formatters.formatSize(mem.cached), color: .cyan)
                    }

                    // Purge button
                    Button {
                        vm.purge()
                    } label: {
                        Label("Purgar memoria", systemImage: "arrow.clockwise")
                    }
                    .buttonStyle(.bordered)

                    if let error = vm.error {
                        ErrorBanner(message: error)
                    }
                }
            }
            .padding()
        }
        .navigationTitle("Memoria")
        .onAppear { vm.startPolling() }
        .onDisappear { vm.stopPolling() }
    }

    private func memRow(_ label: String, value: String) -> some View {
        HStack {
            Text(label)
                .foregroundStyle(.secondary)
            Spacer()
            Text(value)
                .fontWeight(.medium)
        }
    }

    private func statCard(_ title: String, value: String, color: Color) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack(spacing: 4) {
                Circle().fill(color).frame(width: 8, height: 8)
                Text(title).font(.caption).foregroundStyle(.secondary)
            }
            Text(value).font(.callout).fontWeight(.semibold)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .cardStyle()
    }
}
