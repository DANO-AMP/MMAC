import SwiftUI

struct ProjectsView: View {
    @StateObject private var vm = ProjectsViewModel()
    @EnvironmentObject var settings: SettingsStore

    var body: some View {
        VStack(spacing: 0) {
            HStack {
                VStack(alignment: .leading) {
                    Text("Artefactos de Proyectos").font(.title2).fontWeight(.bold)
                    Text("\(vm.artifacts.count) artefactos, \(Formatters.formatSize(vm.totalSize)) total")
                        .foregroundStyle(.secondary)
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
                        vm.deleteSelected(moveToTrash: settings.moveToTrash)
                    } label: {
                        Label("Eliminar (\(vm.selectedPaths.count))", systemImage: "trash")
                    }
                    .tint(.red)
                }
            }
            .padding()

            if vm.isLoading {
                ProgressView("Buscando artefactos de proyectos...")
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                List(vm.artifacts) { artifact in
                    HStack {
                        Toggle("", isOn: Binding(
                            get: { vm.selectedPaths.contains(artifact.artifactPath) },
                            set: { if $0 { vm.selectedPaths.insert(artifact.artifactPath) } else { vm.selectedPaths.remove(artifact.artifactPath) } }
                        ))
                        .labelsHidden()

                        VStack(alignment: .leading, spacing: 2) {
                            HStack {
                                Text(artifact.projectName).fontWeight(.medium)
                                Text(artifact.artifactType)
                                    .font(.caption)
                                    .padding(.horizontal, 6).padding(.vertical, 2)
                                    .background(Color.blue.opacity(0.15))
                                    .cornerRadius(4)
                                if artifact.isRecent {
                                    Text("Reciente")
                                        .font(.caption2)
                                        .foregroundStyle(.orange)
                                }
                            }
                            Text(artifact.artifactPath)
                                .font(.caption).foregroundStyle(.tertiary)
                                .lineLimit(1).truncationMode(.middle)
                        }
                        Spacer()
                        Text(Formatters.formatSize(artifact.size))
                            .font(.caption.monospaced())
                            .fontWeight(.semibold)
                    }
                }
            }
        }
        .navigationTitle("Proyectos")
        .task { await vm.scan() }
    }
}
