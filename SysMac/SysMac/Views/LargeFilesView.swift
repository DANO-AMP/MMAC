import SwiftUI

struct LargeFilesView: View {
    @StateObject private var vm = LargeFilesViewModel()
    @State private var fileToDelete: LargeFile?

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
                        Button {
                            fileToDelete = file
                        } label: {
                            Image(systemName: "trash")
                                .foregroundStyle(.red)
                        }
                        .buttonStyle(.borderless)
                    }
                }
            }
        }
        .navigationTitle("Archivos Grandes")
        .task { await vm.scan() }
        .confirmationDialog(
            "Eliminar archivo",
            isPresented: Binding(get: { fileToDelete != nil }, set: { if !$0 { fileToDelete = nil } }),
            presenting: fileToDelete
        ) { file in
            Button("Mover a la papelera", role: .destructive) {
                vm.deleteFile(file)
                fileToDelete = nil
            }
        } message: { file in
            Text("\(file.name)\n\(Formatters.formatSize(file.size))")
        }
    }
}
