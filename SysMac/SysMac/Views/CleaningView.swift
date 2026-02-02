import SwiftUI

struct CleaningView: View {
    @StateObject private var vm = CleaningViewModel()
    @EnvironmentObject var settings: SettingsStore

    private let categoryLabels: [String: String] = [
        "cache": "Cachés del sistema",
        "logs": "Archivos de log",
        "browser": "Datos de navegador",
        "trash": "Papelera",
        "crash_reports": "Reportes de error",
        "xcode": "Datos de Xcode",
        "packages": "Cachés de paquetes",
    ]

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                HStack {
                    VStack(alignment: .leading) {
                        Text("Limpieza del Sistema")
                            .font(.title2).fontWeight(.bold)
                        Text("Libera espacio eliminando archivos innecesarios")
                            .foregroundStyle(.secondary)
                    }
                    Spacer()
                    Button {
                        Task { await vm.scan() }
                    } label: {
                        Label("Escanear", systemImage: "magnifyingglass")
                    }
                    .disabled(vm.isScanning)
                }

                if vm.isScanning {
                    ProgressView("Escaneando...")
                        .frame(maxWidth: .infinity)
                        .padding()
                }

                if !vm.results.isEmpty {
                    HStack {
                        Text("Total: \(Formatters.formatSize(vm.totalSize))")
                            .font(.headline)
                        Spacer()
                        if !vm.selectedCategories.isEmpty {
                            Text("Seleccionado: \(Formatters.formatSize(vm.selectedSize))")
                                .foregroundStyle(.blue)
                        }
                    }

                    ForEach(vm.results) { result in
                        HStack {
                            Toggle(isOn: Binding(
                                get: { vm.selectedCategories.contains(result.category) },
                                set: { if $0 { vm.selectedCategories.insert(result.category) } else { vm.selectedCategories.remove(result.category) } }
                            )) {
                                VStack(alignment: .leading, spacing: 2) {
                                    Text(categoryLabels[result.category] ?? result.category)
                                        .fontWeight(.medium)
                                    Text("\(result.items) elementos")
                                        .font(.caption).foregroundStyle(.secondary)
                                }
                            }
                            Spacer()
                            Text(Formatters.formatSize(result.size))
                                .fontWeight(.semibold)
                                .foregroundStyle(result.size > 1_073_741_824 ? .red : result.size > 104_857_600 ? .orange : .primary)
                        }
                        .padding(10)
                        .background(.quaternary.opacity(0.3))
                        .cornerRadius(8)
                    }

                    if !vm.selectedCategories.isEmpty {
                        Button {
                            Task { await vm.clean(moveToTrash: settings.moveToTrash) }
                        } label: {
                            Label("Limpiar seleccionados", systemImage: "trash")
                        }
                        .buttonStyle(.borderedProminent)
                        .tint(.red)
                        .disabled(vm.isCleaning)
                    }
                }
            }
            .padding()
        }
        .navigationTitle("Limpieza")
        .task { await vm.scan() }
    }
}
