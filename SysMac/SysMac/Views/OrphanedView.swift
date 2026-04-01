import SwiftUI

struct OrphanedView: View {
    @StateObject private var vm = OrphanedViewModel()
    @EnvironmentObject var settings: SettingsStore

    var body: some View {
        VStack(spacing: 0) {
            HStack {
                VStack(alignment: .leading) {
                    Text("Archivos Huérfanos").font(.title2).fontWeight(.bold)
                    if let r = vm.result {
                        Text("\(r.totalCount) archivos, \(Formatters.formatSize(r.totalSize)) total")
                            .foregroundStyle(.secondary)
                    }
                }
                Spacer()
                Button {
                    Task { await vm.scan() }
                } label: {
                    Label("Escanear", systemImage: "magnifyingglass")
                }
                .disabled(vm.isLoading)

                if !vm.selectedPaths.isEmpty {
                    Button {
                        Task { await vm.deleteSelected(moveToTrash: settings.moveToTrash) }
                    } label: {
                        Label("Eliminar (\(vm.selectedPaths.count))", systemImage: "trash")
                    }
                    .tint(.red)
                }
            }
            .padding()

            if vm.isLoading {
                ProgressView("Buscando archivos huérfanos...")
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else if let result = vm.result {
                List(result.files) { file in
                    HStack {
                        Toggle("", isOn: Binding(
                            get: { vm.selectedPaths.contains(file.path) },
                            set: { if $0 { vm.selectedPaths.insert(file.path) } else { vm.selectedPaths.remove(file.path) } }
                        ))
                        .labelsHidden()

                        VStack(alignment: .leading, spacing: 2) {
                            Text(file.likelyApp).fontWeight(.medium)
                            Text(file.fileType)
                                .font(.caption).foregroundStyle(.secondary)
                        }
                        Spacer()
                        Text(Formatters.formatSize(file.size))
                            .font(.caption.monospaced())
                    }
                }
            }
        }
        .navigationTitle("Huérfanos")
        .task { await vm.scan() }
    }
}
