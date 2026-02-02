import SwiftUI

struct BatteryView: View {
    @StateObject private var vm = BatteryViewModel()

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                HStack {
                    VStack(alignment: .leading) {
                        Text("Batería")
                            .font(.title2)
                            .fontWeight(.bold)
                        Text("Estado e información de la batería")
                            .foregroundStyle(.secondary)
                    }
                    Spacer()
                    if vm.isLoading { ProgressView().scaleEffect(0.7) }
                }

                if vm.noBattery {
                    VStack(spacing: 12) {
                        Image(systemName: "battery.0")
                            .font(.system(size: 48))
                            .foregroundStyle(.secondary)
                        Text("No se detectó batería")
                            .foregroundStyle(.secondary)
                        Text("Este equipo no tiene batería o no se pudo acceder a la información.")
                            .font(.caption)
                            .foregroundStyle(.tertiary)
                            .multilineTextAlignment(.center)
                    }
                    .frame(maxWidth: .infinity)
                    .padding(.vertical, 40)
                } else if let bat = vm.batteryInfo {
                    // Main charge display
                    HStack(spacing: 24) {
                        ZStack {
                            let color: Color = bat.chargePercent > 50 ? .green : bat.chargePercent > 20 ? .orange : .red
                            ProgressRing(progress: Double(bat.chargePercent) / 100.0, lineWidth: 12, size: 120, color: color)
                            VStack {
                                Text("\(Int(bat.chargePercent))%")
                                    .font(.title)
                                    .fontWeight(.bold)
                                Text(bat.powerSource)
                                    .font(.caption2)
                                    .foregroundStyle(.secondary)
                            }
                        }

                        VStack(alignment: .leading, spacing: 8) {
                            infoRow("Estado", value: bat.condition)
                            infoRow("Ciclos", value: "\(bat.cycleCount)")
                            infoRow("Salud", value: Formatters.formatPercentage(Double(bat.healthPercent)))
                            if let time = bat.timeRemaining {
                                infoRow("Tiempo", value: time)
                            }
                        }
                    }
                    .cardStyle()

                    // Details
                    HStack(spacing: 12) {
                        detailCard("Temperatura", value: Formatters.formatTemperature(Double(bat.temperature)), icon: "thermometer.medium")
                        detailCard("Voltaje", value: String(format: "%.2f V", bat.voltage), icon: "bolt")
                        detailCard("Amperaje", value: "\(bat.amperage) mA", icon: "bolt.fill")
                    }

                    HStack(spacing: 12) {
                        detailCard("Capacidad Max", value: "\(bat.maxCapacity) mAh", icon: "battery.75")
                        detailCard("Capacidad Diseño", value: "\(bat.designCapacity) mAh", icon: "battery.100")
                    }
                }
            }
            .padding()
        }
        .navigationTitle("Batería")
        .onAppear { vm.startPolling() }
        .onDisappear { vm.stopPolling() }
    }

    private func infoRow(_ label: String, value: String) -> some View {
        HStack {
            Text(label).foregroundStyle(.secondary)
            Spacer()
            Text(value).fontWeight(.medium)
        }
    }

    private func detailCard(_ title: String, value: String, icon: String) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack(spacing: 4) {
                Image(systemName: icon).foregroundStyle(.secondary)
                Text(title).font(.caption).foregroundStyle(.secondary)
            }
            Text(value).font(.callout).fontWeight(.semibold)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .cardStyle()
    }
}
