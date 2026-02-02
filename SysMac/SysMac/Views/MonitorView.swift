import SwiftUI
import Charts

struct MonitorView: View {
    @StateObject private var vm = MonitorViewModel()

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                header
                statsCards
                chartsSection
                networkSection
                fanAndDiskSection
                gpuSection
            }
            .padding()
        }
        .navigationTitle("Monitor del Sistema")
        .onAppear { vm.startPolling() }
        .onDisappear { vm.stopPolling() }
    }

    // MARK: - Header

    private var header: some View {
        HStack {
            VStack(alignment: .leading) {
                Text("Monitor del Sistema")
                    .font(.title2)
                    .fontWeight(.bold)
                Text("Métricas en tiempo real de tu Mac")
                    .foregroundStyle(.secondary)
            }
            Spacer()
            if vm.isLoading {
                ProgressView()
                    .scaleEffect(0.7)
            }
            Button {
                vm.isLive.toggle()
            } label: {
                HStack(spacing: 6) {
                    Circle()
                        .fill(vm.isLive ? Color.green : Color.gray)
                        .frame(width: 8, height: 8)
                    Text(vm.isLive ? "En vivo" : "Pausado")
                }
                .padding(.horizontal, 12)
                .padding(.vertical, 6)
                .background(vm.isLive ? Color.green.opacity(0.15) : Color.gray.opacity(0.15))
                .cornerRadius(8)
            }
            .buttonStyle(.plain)
        }
    }

    // MARK: - Stats Cards

    private var statsCards: some View {
        let current = vm.stats ?? SystemStats()
        let memPercent = current.memoryTotal > 0
            ? Double(current.memoryUsed) / Double(current.memoryTotal) * 100.0
            : 0.0
        let diskPercent = current.diskTotal > 0
            ? Double(current.diskUsed) / Double(current.diskTotal) * 100.0
            : 0.0

        return HStack(spacing: 12) {
            metricCard(
                title: "CPU",
                value: Formatters.formatPercentage(Double(current.cpuUsage)),
                progress: Double(current.cpuUsage) / 100.0,
                icon: "cpu",
                color: .blue
            )
            metricCard(
                title: "Memoria",
                value: Formatters.formatPercentage(memPercent),
                subtitle: "\(Formatters.formatSize(current.memoryUsed)) / \(Formatters.formatSize(current.memoryTotal))",
                progress: memPercent / 100.0,
                icon: "memorychip",
                color: .purple
            )
            metricCard(
                title: "Disco",
                value: Formatters.formatPercentage(diskPercent),
                subtitle: "\(Formatters.formatSize(current.diskUsed)) / \(Formatters.formatSize(current.diskTotal))",
                progress: diskPercent / 100.0,
                icon: "internaldrive",
                color: .orange
            )
            metricCard(
                title: "Temperatura",
                value: Formatters.formatTemperature(Double(current.cpuTemp)),
                subtitle: current.cpuTemp < 50 ? "Normal" : current.cpuTemp < 70 ? "Moderada" : "Alta",
                progress: Double(current.cpuTemp) / 100.0,
                icon: "thermometer.medium",
                color: .red
            )
        }
    }

    private func metricCard(title: String, value: String, subtitle: String? = nil, progress: Double, icon: String, color: Color) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Image(systemName: icon)
                    .foregroundStyle(color)
                Text(title)
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
            Text(value)
                .font(.title2)
                .fontWeight(.bold)
            if let subtitle {
                Text(subtitle)
                    .font(.caption2)
                    .foregroundStyle(.tertiary)
            }
            ProgressView(value: min(max(progress, 0), 1))
                .tint(color)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .cardStyle()
    }

    // MARK: - Charts

    private var chartsSection: some View {
        HStack(spacing: 12) {
            // CPU Chart
            VStack(alignment: .leading, spacing: 8) {
                Label("Uso de CPU", systemImage: "waveform.path.ecg")
                    .font(.headline)
                Chart(vm.chartData) { point in
                    LineMark(
                        x: .value("Tiempo", point.time),
                        y: .value("CPU", point.cpu)
                    )
                    .foregroundStyle(.blue)
                    .interpolationMethod(.catmullRom)
                }
                .chartYScale(domain: 0...100)
                .chartXAxis {
                    AxisMarks(values: .automatic(desiredCount: 5)) { _ in
                        AxisValueLabel()
                            .font(.caption2)
                    }
                }
                .frame(height: 180)
            }
            .cardStyle()

            // Memory Chart
            VStack(alignment: .leading, spacing: 8) {
                Label("Uso de Memoria", systemImage: "memorychip")
                    .font(.headline)
                Chart(vm.chartData) { point in
                    LineMark(
                        x: .value("Tiempo", point.time),
                        y: .value("Memoria", point.memory)
                    )
                    .foregroundStyle(.purple)
                    .interpolationMethod(.catmullRom)
                }
                .chartYScale(domain: 0...100)
                .chartXAxis {
                    AxisMarks(values: .automatic(desiredCount: 5)) { _ in
                        AxisValueLabel()
                            .font(.caption2)
                    }
                }
                .frame(height: 180)
            }
            .cardStyle()
        }
    }

    // MARK: - Network

    private var networkSection: some View {
        let current = vm.stats ?? SystemStats()
        return VStack(alignment: .leading, spacing: 8) {
            Label("Red", systemImage: "wifi")
                .font(.headline)
            HStack(spacing: 32) {
                HStack(spacing: 8) {
                    Image(systemName: "arrow.down")
                        .foregroundStyle(.green)
                    VStack(alignment: .leading) {
                        Text("Descarga")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                        Text(Formatters.formatSpeed(current.networkRx))
                            .font(.title3)
                            .fontWeight(.semibold)
                    }
                }
                HStack(spacing: 8) {
                    Image(systemName: "arrow.up")
                        .foregroundStyle(.blue)
                    VStack(alignment: .leading) {
                        Text("Subida")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                        Text(Formatters.formatSpeed(current.networkTx))
                            .font(.title3)
                            .fontWeight(.semibold)
                    }
                }
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .cardStyle()
    }

    // MARK: - Fan & Disk I/O

    private var fanAndDiskSection: some View {
        let current = vm.stats ?? SystemStats()
        return HStack(spacing: 12) {
            // Fan
            VStack(alignment: .leading, spacing: 8) {
                Label("Ventilador", systemImage: "fan")
                    .font(.headline)
                HStack {
                    VStack(alignment: .leading) {
                        Text(current.fanSpeed.map { "\($0)" } ?? "--")
                            .font(.title2)
                            .fontWeight(.bold)
                        Text("RPM")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                    Spacer()
                    if let speed = current.fanSpeed {
                        Text(speed < 2000 ? "Bajo" : speed < 4000 ? "Normal" : "Alto")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            .cardStyle()

            // Disk I/O
            VStack(alignment: .leading, spacing: 8) {
                Label("Disco I/O", systemImage: "internaldrive")
                    .font(.headline)
                HStack(spacing: 24) {
                    HStack(spacing: 6) {
                        Image(systemName: "arrow.down")
                            .foregroundStyle(.green)
                        VStack(alignment: .leading) {
                            Text(Formatters.formatSpeed(current.diskReadSpeed))
                                .font(.callout)
                                .fontWeight(.semibold)
                            Text("Lectura")
                                .font(.caption2)
                                .foregroundStyle(.secondary)
                        }
                    }
                    HStack(spacing: 6) {
                        Image(systemName: "arrow.up")
                            .foregroundStyle(.blue)
                        VStack(alignment: .leading) {
                            Text(Formatters.formatSpeed(current.diskWriteSpeed))
                                .font(.callout)
                                .fontWeight(.semibold)
                            Text("Escritura")
                                .font(.caption2)
                                .foregroundStyle(.secondary)
                        }
                    }
                }
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            .cardStyle()
        }
    }

    // MARK: - GPU

    @ViewBuilder
    private var gpuSection: some View {
        let current = vm.stats ?? SystemStats()
        if let gpuName = current.gpuName {
            VStack(alignment: .leading, spacing: 8) {
                Label("GPU", systemImage: "display")
                    .font(.headline)
                Text(gpuName)
                    .font(.title3)
                    .fontWeight(.semibold)
                if let vendor = current.gpuVendor {
                    Text(vendor)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            .cardStyle()
        }
    }
}
