import SwiftUI

struct SettingsView: View {
    @EnvironmentObject var settings: SettingsStore

    var body: some View {
        Form {
            Section("General") {
                Toggle("Notificaciones", isOn: $settings.notifications)
                Toggle("Escaneo automático", isOn: $settings.autoScan)
            }

            Section("Limpieza") {
Toggle("Confirmar antes de eliminar", isOn: $settings.confirmDelete)
                Toggle("Mover a la papelera", isOn: $settings.moveToTrash)
            }
        }
        .formStyle(.grouped)
        .frame(width: 450, height: 300)
    }
}
