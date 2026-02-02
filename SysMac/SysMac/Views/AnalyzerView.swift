import SwiftUI

struct AnalyzerView: View {
    @StateObject private var vm = AnalyzerViewModel()

    var body: some View {
        VStack(spacing: 0) {
            HStack {
                Button { vm.goUp() } label: {
                    Image(systemName: "arrow.up")
                }
                .buttonStyle(.borderless)
                Text(vm.currentPath)
                    .font(.caption.monospaced())
                    .lineLimit(1)
                    .truncationMode(.middle)
                Spacer()
                if vm.isLoading { ProgressView().scaleEffect(0.6) }
            }
            .padding()

            List(vm.items) { item in
                Button {
                    if item.isDir {
                        let home = FileManager.default.homeDirectoryForCurrentUser.path
                        let realPath = item.path.replacingOccurrences(of: "~", with: home)
                        vm.navigateTo(realPath)
                    }
                } label: {
                    HStack {
                        Image(systemName: item.isDir ? "folder.fill" : "doc")
                            .foregroundStyle(item.isDir ? .blue : .secondary)
                            .frame(width: 20)
                        Text(item.name)
                            .lineLimit(1)
                        Spacer()
                        Text(Formatters.formatSize(item.size))
                            .font(.caption.monospaced())
                            .foregroundStyle(.secondary)
                    }
                }
                .buttonStyle(.plain)
            }
        }
        .navigationTitle("Analizador de Disco")
        .task { await vm.analyze() }
    }
}
