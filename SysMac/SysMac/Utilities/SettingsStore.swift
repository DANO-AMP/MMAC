import Foundation
import SwiftUI

@MainActor
final class SettingsStore: ObservableObject {
    @AppStorage("theme") var theme: String = "dark"
    @AppStorage("notifications") var notifications: Bool = true
    @AppStorage("autoScan") var autoScan: Bool = false
@AppStorage("confirmDelete") var confirmDelete: Bool = true
    @AppStorage("moveToTrash") var moveToTrash: Bool = true
}
