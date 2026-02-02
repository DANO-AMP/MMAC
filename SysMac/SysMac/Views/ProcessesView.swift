import SwiftUI

struct ProcessesView: View {
    @StateObject private var vm = ProcessesViewModel()
    @State private var selectedPid: UInt32?
    @State private var showKillConfirm = false

    var body: some View {
        VStack(spacing: 0) {
            // Toolbar
            HStack {
                SearchBar(text: $vm.searchText, placeholder: "Buscar procesos...")
                Spacer()
                Picker("Ordenar", selection: $vm.sortBy) {
                    ForEach(ProcessesViewModel.SortField.allCases, id: \.self) { field in
                        Text(field.rawValue).tag(field)
                    }
                }
                .pickerStyle(.segmented)
                .frame(width: 300)

                Button {
                    vm.sortAscending.toggle()
                } label: {
                    Image(systemName: vm.sortAscending ? "arrow.up" : "arrow.down")
                }
                .buttonStyle(.borderless)

                Text("\(vm.filteredProcesses.count) procesos")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
            .padding()

            // Table
            Table(vm.filteredProcesses, selection: $selectedPid) {
                TableColumn("PID") { proc in
                    Text("\(proc.pid)")
                        .font(.caption.monospaced())
                }
                .width(60)

                TableColumn("Nombre") { proc in
                    Text(proc.name)
                        .lineLimit(1)
                }
                .width(min: 120, ideal: 180)

                TableColumn("CPU %") { proc in
                    Text(String(format: "%.1f", proc.cpuUsage))
                        .font(.caption.monospaced())
                        .foregroundStyle(proc.cpuUsage > 50 ? .red : proc.cpuUsage > 20 ? .orange : .primary)
                }
                .width(60)

                TableColumn("Mem (MB)") { proc in
                    Text(String(format: "%.1f", proc.memoryMB))
                        .font(.caption.monospaced())
                }
                .width(70)

                TableColumn("Usuario") { proc in
                    Text(proc.user)
                        .font(.caption)
                }
                .width(80)

                TableColumn("Estado") { proc in
                    Text(proc.state)
                        .font(.caption)
                }
                .width(80)

                TableColumn("Hilos") { proc in
                    Text("\(proc.threads)")
                        .font(.caption.monospaced())
                }
                .width(50)
            }

            // Bottom bar
            HStack {
                if vm.isLoading {
                    ProgressView()
                        .scaleEffect(0.6)
                }
                Spacer()
                if let pid = selectedPid {
                    Button("Terminar") {
                        showKillConfirm = true
                    }
                    .confirmationDialog("¿Terminar proceso \(pid)?", isPresented: $showKillConfirm) {
                        Button("Terminar (SIGTERM)") { vm.killProcess(pid: pid, force: false) }
                        Button("Forzar (SIGKILL)", role: .destructive) { vm.killProcess(pid: pid, force: true) }
                        Button("Cancelar", role: .cancel) {}
                    }
                }
            }
            .padding(.horizontal)
            .padding(.vertical, 8)
        }
        .navigationTitle("Procesos")
        .onAppear { vm.startPolling() }
        .onDisappear { vm.stopPolling() }
    }
}
