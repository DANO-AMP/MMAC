import SwiftUI

struct UninstallerView: View {
    @StateObject private var vm = UninstallerViewModel()
    @EnvironmentObject var settings: SettingsStore
    @State private var appToUninstall: AppInfo?

    var body: some View {
        VStack(spacing: 0) {
            HStack {
                VStack(alignment: .leading) {
                    Text("Desinstalar Apps").font(.title2).fontWeight(.bold)
                    Text("\(vm.filteredApps.count) aplicaciones")
                        .foregroundStyle(.secondary)
                }
                Spacer()
                SearchBar(text: $vm.searchText, placeholder: "Buscar apps...")
                    .frame(maxWidth: 250)
            }
            .padding()

            if vm.isLoading {
                ProgressView("Cargando aplicaciones...")
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                List(vm.filteredApps) { app in
                    HStack {
                        VStack(alignment: .leading, spacing: 2) {
                            HStack {
                                Text(app.name).fontWeight(.medium)
                                if let version = app.version {
                                    Text("v\(version)")
                                        .font(.caption).foregroundStyle(.secondary)
                                }
                            }
                            Text(app.bundleId)
                                .font(.caption).foregroundStyle(.tertiary)
                            if !app.remnants.isEmpty {
                                Text("\(app.remnants.count) residuos (\(Formatters.formatSize(app.remnantsSize)))")
                                    .font(.caption).foregroundStyle(.orange)
                            }
                        }
                        Spacer()
                        Text(Formatters.formatSize(app.size))
                            .font(.caption.monospaced())
                        Button {
                            appToUninstall = app
                        } label: {
                            Image(systemName: "trash")
                                .foregroundStyle(.red)
                        }
                        .buttonStyle(.borderless)
                    }
                }
            }

            if let error = vm.error {
                ErrorBanner(message: error)
                    .padding()
            }
        }
        .navigationTitle("Desinstalar")
        .task { await vm.loadApps() }
        .confirmationDialog(
            "¿Desinstalar \(appToUninstall?.name ?? "")?",
            isPresented: Binding(get: { appToUninstall != nil }, set: { if !$0 { appToUninstall = nil } }),
            titleVisibility: .visible
        ) {
            Button("Desinstalar", role: .destructive) {
                if let app = appToUninstall {
                    vm.uninstall(app: app, moveToTrash: settings.moveToTrash)
                }
            }
            Button("Cancelar", role: .cancel) {}
        } message: {
            if let app = appToUninstall {
                Text("Se eliminará la app (\(Formatters.formatSize(app.size))) y \(app.remnants.count) residuos (\(Formatters.formatSize(app.remnantsSize)))")
            }
        }
    }
}
