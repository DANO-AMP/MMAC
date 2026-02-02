import SwiftUI

enum SidebarItem: String, CaseIterable, Identifiable {
    // Sistema
    case cleaning
    case uninstaller
    case startup
    case monitor
    case processes
    case memory
    case battery
    case bluetooth
    case services
    // Archivos
    case analyzer
    case largefiles
    case duplicates
    case projects
    case orphaned
    // Paquetes
    case homebrew
    // Red
    case ports
    case connections
    case firewall

    var id: String { rawValue }

    var label: String {
        switch self {
        case .cleaning: return "Limpieza"
        case .uninstaller: return "Desinstalar"
        case .startup: return "Inicio"
        case .monitor: return "Monitor"
        case .processes: return "Procesos"
        case .memory: return "Memoria"
        case .battery: return "Batería"
        case .bluetooth: return "Bluetooth"
        case .services: return "Servicios"
        case .analyzer: return "Analizador"
        case .largefiles: return "Grandes"
        case .duplicates: return "Duplicados"
        case .projects: return "Proyectos"
        case .orphaned: return "Huérfanos"
        case .homebrew: return "Homebrew"
        case .ports: return "Puertos"
        case .connections: return "Conexiones"
        case .firewall: return "Firewall"
        }
    }

    var icon: String {
        switch self {
        case .cleaning: return "sparkles"
        case .uninstaller: return "macwindow"
        case .startup: return "bolt.circle"
        case .monitor: return "waveform.path.ecg"
        case .processes: return "cpu"
        case .memory: return "memorychip"
        case .battery: return "battery.100"
        case .bluetooth: return "wave.3.right"
        case .services: return "gearshape.2"
        case .analyzer: return "internaldrive"
        case .largefiles: return "doc.richtext"
        case .duplicates: return "doc.on.doc"
        case .projects: return "folder"
        case .orphaned: return "questionmark.folder"
        case .homebrew: return "cup.and.saucer"
        case .ports: return "globe"
        case .connections: return "network"
        case .firewall: return "shield"
        }
    }
}

struct SidebarSection: Identifiable {
    let id = UUID()
    let title: String
    let items: [SidebarItem]
}

let sidebarSections: [SidebarSection] = [
    SidebarSection(title: "Sistema", items: [.cleaning, .uninstaller, .startup, .monitor, .processes, .memory, .battery, .bluetooth, .services]),
    SidebarSection(title: "Archivos", items: [.analyzer, .largefiles, .duplicates, .projects, .orphaned]),
    SidebarSection(title: "Paquetes", items: [.homebrew]),
    SidebarSection(title: "Red", items: [.ports, .connections, .firewall]),
]

struct SidebarView: View {
    @Binding var selection: SidebarItem

    var body: some View {
        List(selection: $selection) {
            ForEach(sidebarSections) { section in
                Section(section.title) {
                    ForEach(section.items) { item in
                        Label(item.label, systemImage: item.icon)
                            .tag(item)
                    }
                }
            }
        }
        .listStyle(.sidebar)
        .frame(minWidth: 180)
    }
}
