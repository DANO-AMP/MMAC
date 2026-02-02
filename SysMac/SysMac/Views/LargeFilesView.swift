import SwiftUI

struct LargeFilesView: View {
    @StateObject private var vm = LargeFilesViewModel()

    var body: some View {
        VStack(spacing: 0) {
            HStack {
                VStack(alignment: .leading) {
                    Text("Archivos Grandes").font(.title2).fontWeight(.bold)
                    Text("Total: \(Formatters.formatSize(vm.totalSize)) en \(vm.files.count) archivos")
                        .foregroundStyle(.secondary)
                }
                Spacer()
                HStack {
                    Text("Min:")
                    TextField("MB", value: $vm.minSizeMB, format: .number)
                        .frame(width: 60)
                        .textFieldStyle(.roundedBorder)
                    Text("MB")
                }
                Button {
                    Task { await vm.scan() }
                } label: {
                    Label("Buscar", systemImage: "magnifyingglass")
                }
                .disabled(vm.isLoading)
            }
            .padding()

            if vm.isLoading {
                ProgressView("Buscando archivos grandes...")
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                List(vm.files) { file in
                    HStack {
                        Image(systemName: "doc")
                            .foregroundStyle(.secondary)
                        VStack(alignment: .leading, spacing: 2) {
                            Text(file.name).lineLimit(1)
                            Text(file.path)
                                .font(.caption).foregroundStyle(.tertiary)
                                .lineLimit(1).truncationMode(.middle)
                        }
                        Spacer()
                        Text(Formatters.formatSize(file.size))
                            .font(.caption.monospaced())
                            .fontWeight(.semibold)
                    }
                }
            }
        }
        .navigationTitle("Archivos Grandes")
        .task { await vm.scan() }
    }
}
