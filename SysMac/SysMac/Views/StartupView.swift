import SwiftUI

struct StartupView: View {
    @StateObject private var vm = StartupViewModel()

    var body: some View {
        VStack(spacing: 0) {
            HStack {
                VStack(alignment: .leading) {
                    Text("Elementos de Inicio").font(.title2).fontWeight(.bold)
                    Text("\(vm.items.count) elementos")
                        .foregroundStyle(.secondary)
                }
                Spacer()
                if vm.isLoading { ProgressView().scaleEffect(0.6) }
                Button {
                    Task { await vm.load() }
                } label: {
                    Label("Actualizar", systemImage: "arrow.clockwise")
                }
            }
            .padding()

            List(vm.items) { item in
                HStack {
                    Toggle("", isOn: Binding(
                        get: { item.enabled },
                        set: { _ in vm.toggle(item: item) }
                    ))
                    .labelsHidden()

                    VStack(alignment: .leading, spacing: 2) {
                        Text(item.name).fontWeight(.medium)
                        HStack(spacing: 8) {
                            Text(item.kind)
                                .font(.caption)
                                .padding(.horizontal, 6).padding(.vertical, 2)
                                .background(Color.blue.opacity(0.15))
                                .cornerRadius(4)
                            if !item.path.isEmpty {
                                Text(item.path)
                                    .font(.caption).foregroundStyle(.tertiary)
                                    .lineLimit(1).truncationMode(.middle)
                            }
                        }
                    }
                    Spacer()
                    Text(item.enabled ? "Activo" : "Inactivo")
                        .font(.caption)
                        .foregroundStyle(item.enabled ? .green : .secondary)
                }
            }

            if let error = vm.error {
                ErrorBanner(message: error).padding()
            }
        }
        .navigationTitle("Inicio")
        .task { await vm.load() }
    }
}
