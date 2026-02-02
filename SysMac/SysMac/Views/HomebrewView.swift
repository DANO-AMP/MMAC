import SwiftUI

struct HomebrewView: View {
    @StateObject private var vm = HomebrewViewModel()

    var body: some View {
        VStack(spacing: 0) {
            HStack {
                VStack(alignment: .leading) {
                    Text("Homebrew").font(.title2).fontWeight(.bold)
                    if let info = vm.info {
                        Text(info.installed
                            ? "v\(info.version ?? "?") - \(info.formulaeCount) fórmulas, \(info.casksCount) casks"
                            : "No instalado")
                            .foregroundStyle(.secondary)
                    }
                }
                Spacer()
                SearchBar(text: $vm.searchText, placeholder: "Buscar paquetes...")
                    .frame(maxWidth: 200)
                Toggle("Solo desactualizados", isOn: $vm.showOutdatedOnly)
                    .toggleStyle(.switch)
                Button {
                    vm.cleanup()
                } label: {
                    Label("Cleanup", systemImage: "trash")
                }
            }
            .padding()

            if vm.isLoading {
                ProgressView("Cargando paquetes...")
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                List(vm.filteredPackages) { pkg in
                    HStack {
                        VStack(alignment: .leading, spacing: 2) {
                            HStack {
                                Text(pkg.name).fontWeight(.medium)
                                Text(pkg.isCask ? "cask" : "formula")
                                    .font(.caption2)
                                    .padding(.horizontal, 4).padding(.vertical, 1)
                                    .background(pkg.isCask ? Color.purple.opacity(0.15) : Color.blue.opacity(0.15))
                                    .cornerRadius(3)
                            }
                            Text("v\(pkg.version)")
                                .font(.caption).foregroundStyle(.secondary)
                        }
                        Spacer()
                        if pkg.isOutdated {
                            if let newer = pkg.newerVersion {
                                Text("→ \(newer)")
                                    .font(.caption).foregroundStyle(.orange)
                            }
                            Button("Actualizar") { vm.upgrade(pkg.name) }
                                .buttonStyle(.bordered)
                                .controlSize(.small)
                        }
                        Button {
                            vm.uninstall(pkg.name)
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
        .navigationTitle("Homebrew")
        .task { await vm.load() }
    }
}
