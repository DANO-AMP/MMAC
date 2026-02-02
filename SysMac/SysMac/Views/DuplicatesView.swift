import SwiftUI

struct DuplicatesView: View {
    @StateObject private var vm = DuplicatesViewModel()

    var body: some View {
        VStack(spacing: 0) {
            HStack {
                VStack(alignment: .leading) {
                    Text("Archivos Duplicados").font(.title2).fontWeight(.bold)
                    if let r = vm.result {
                        Text("\(r.groups.count) grupos, \(Formatters.formatSize(r.totalWasted)) desperdiciados, \(r.filesScanned) archivos escaneados")
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
            }
            .padding()

            if vm.isLoading {
                ProgressView("Buscando duplicados (esto puede tardar)...")
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else if let result = vm.result {
                List(result.groups) { group in
                    DisclosureGroup {
                        ForEach(group.files, id: \.self) { path in
                            Text(path)
                                .font(.caption.monospaced())
                                .lineLimit(1)
                                .truncationMode(.middle)
                        }
                    } label: {
                        HStack {
                            Text("\(group.files.count) archivos")
                            Spacer()
                            Text(Formatters.formatSize(group.size))
                                .font(.caption.monospaced())
                        }
                    }
                }
            }
        }
        .navigationTitle("Duplicados")
    }
}
