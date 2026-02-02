import SwiftUI

struct BluetoothView: View {
    @StateObject private var vm = BluetoothViewModel()

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                HStack {
                    VStack(alignment: .leading) {
                        Text("Bluetooth")
                            .font(.title2)
                            .fontWeight(.bold)
                        Text("Dispositivos y estado del controlador")
                            .foregroundStyle(.secondary)
                    }
                    Spacer()
                    if vm.isLoading { ProgressView().scaleEffect(0.7) }
                }

                if let error = vm.error {
                    ErrorBanner(message: error)
                }

                if let info = vm.info {
                    // Controller status
                    HStack(spacing: 12) {
                        statusCard("Bluetooth", value: info.enabled ? "Encendido" : "Apagado", color: info.enabled ? .green : .red)
                        statusCard("Descubrible", value: info.discoverable ? "Sí" : "No", color: info.discoverable ? .blue : .gray)
                        if let addr = info.address {
                            statusCard("Dirección", value: addr, color: .secondary)
                        }
                    }

                    // Connected devices
                    let connected = info.devices.filter(\.isConnected)
                    let disconnected = info.devices.filter { !$0.isConnected }

                    if !connected.isEmpty {
                        Text("Conectados (\(connected.count))")
                            .font(.headline)
                        ForEach(connected) { device in
                            deviceRow(device)
                        }
                    }

                    if !disconnected.isEmpty {
                        Text("No conectados (\(disconnected.count))")
                            .font(.headline)
                            .padding(.top, 8)
                        ForEach(disconnected) { device in
                            deviceRow(device)
                        }
                    }

                    if info.devices.isEmpty {
                        VStack(spacing: 8) {
                            Image(systemName: "wave.3.right")
                                .font(.system(size: 36))
                                .foregroundStyle(.secondary)
                            Text("No se encontraron dispositivos")
                                .foregroundStyle(.secondary)
                        }
                        .frame(maxWidth: .infinity)
                        .padding(.vertical, 24)
                    }
                }
            }
            .padding()
        }
        .navigationTitle("Bluetooth")
        .onAppear { vm.startPolling() }
        .onDisappear { vm.stopPolling() }
    }

    private func statusCard(_ title: String, value: String, color: Color) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(title).font(.caption).foregroundStyle(.secondary)
            Text(value).font(.callout).fontWeight(.semibold).foregroundStyle(color)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .cardStyle()
    }

    private func deviceRow(_ device: BluetoothDevice) -> some View {
        HStack {
            Image(systemName: iconForType(device.deviceType))
                .frame(width: 28)
                .foregroundStyle(.secondary)
            VStack(alignment: .leading, spacing: 2) {
                Text(device.name).fontWeight(.medium)
                HStack(spacing: 8) {
                    Text(device.deviceType).font(.caption).foregroundStyle(.secondary)
                    if let vendor = device.vendor {
                        Text(vendor).font(.caption).foregroundStyle(.tertiary)
                    }
                }
            }
            Spacer()
            if let battery = device.batteryPercent {
                HStack(spacing: 4) {
                    Image(systemName: batteryIcon(battery))
                        .foregroundStyle(battery > 20 ? .green : .red)
                    Text("\(battery)%")
                        .font(.caption)
                }
            }
            Circle()
                .fill(device.isConnected ? Color.green : Color.gray)
                .frame(width: 8, height: 8)
        }
        .padding(10)
        .background(.quaternary.opacity(0.3))
        .cornerRadius(8)
    }

    private func iconForType(_ type: String) -> String {
        switch type {
        case "Headphones": return "headphones"
        case "Keyboard": return "keyboard"
        case "Mouse": return "computermouse"
        case "Watch": return "applewatch"
        case "iOS Device": return "iphone"
        case "Speaker": return "hifispeaker"
        case "Controller": return "gamecontroller"
        default: return "wave.3.right"
        }
    }

    private func batteryIcon(_ percent: UInt8) -> String {
        if percent > 75 { return "battery.100" }
        if percent > 50 { return "battery.75" }
        if percent > 25 { return "battery.50" }
        return "battery.25"
    }
}
