import Foundation
import SwiftUI

final class SettingsStore: ObservableObject {
    @AppStorage("theme") var theme: String = "dark"
    @AppStorage("notifications") var notifications: Bool = true
    @AppStorage("autoScan") var autoScan: Bool = false
    @AppStorage("protectRecent") var protectRecent: Bool = true
    @AppStorage("recentDays") var recentDays: Int = 7
    @AppStorage("confirmDelete") var confirmDelete: Bool = true
    @AppStorage("moveToTrash") var moveToTrash: Bool = true
}
