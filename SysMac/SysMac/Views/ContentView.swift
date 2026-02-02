import SwiftUI

struct ContentView: View {
    @State private var selection: SidebarItem = .monitor

    var body: some View {
        NavigationSplitView {
            SidebarView(selection: $selection)
        } detail: {
            detailView(for: selection)
        }
    }

    @ViewBuilder
    private func detailView(for item: SidebarItem) -> some View {
        switch item {
        case .monitor:
            MonitorView()
        case .cleaning:
            CleaningView()
        case .uninstaller:
            UninstallerView()
        case .startup:
            StartupView()
        case .processes:
            ProcessesView()
        case .memory:
            MemoryView()
        case .battery:
            BatteryView()
        case .bluetooth:
            BluetoothView()
        case .services:
            ServicesView()
        case .analyzer:
            AnalyzerView()
        case .largefiles:
            LargeFilesView()
        case .duplicates:
            DuplicatesView()
        case .projects:
            ProjectsView()
        case .orphaned:
            OrphanedView()
        case .homebrew:
            HomebrewView()
        case .ports:
            PortScannerView()
        case .connections:
            ConnectionsView()
        case .firewall:
            FirewallView()
        }
    }
}
