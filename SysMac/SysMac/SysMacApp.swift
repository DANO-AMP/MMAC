import SwiftUI

@main
struct SysMacApp: App {
    @StateObject private var settingsStore = SettingsStore()

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(settingsStore)
                .preferredColorScheme(settingsStore.theme == "dark" ? .dark : settingsStore.theme == "light" ? .light : nil)
                .frame(minWidth: 900, minHeight: 600)
        }
        .windowStyle(.titleBar)
        .defaultSize(width: 1200, height: 800)
        .commands {
            CommandGroup(replacing: .appInfo) {
                Button("Acerca de SysMac") {
                    NSApplication.shared.orderFrontStandardAboutPanel(
                        options: [
                            .applicationName: "SysMac",
                            .applicationVersion: Bundle.main.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String ?? "1.0",
                            .version: Bundle.main.object(forInfoDictionaryKey: "CFBundleVersion") as? String ?? "1",
                            .credits: NSAttributedString(
                                string: "Utilidad de sistema para macOS",
                                attributes: [.font: NSFont.systemFont(ofSize: 11)]
                            ),
                        ]
                    )
                }
            }
        }

        Settings {
            SettingsView()
                .environmentObject(settingsStore)
        }
    }
}
