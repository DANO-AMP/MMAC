import Foundation
import SwiftUI

// MARK: - Color Extensions

extension Color {
    static let darkBg = Color(red: 0.07, green: 0.07, blue: 0.09)
    static let darkCard = Color(red: 0.1, green: 0.1, blue: 0.12)
    static let darkBorder = Color(red: 0.15, green: 0.15, blue: 0.18)
    static let primary400 = Color(red: 0.38, green: 0.56, blue: 1.0)
    static let primary500 = Color(red: 0.25, green: 0.47, blue: 0.98)
    static let primary600 = Color(red: 0.2, green: 0.4, blue: 0.9)
    static let primary700 = Color(red: 0.15, green: 0.33, blue: 0.8)
}

// MARK: - View Extensions

extension View {
    func cardStyle() -> some View {
        self
            .padding()
            .background(Color.darkCard)
            .cornerRadius(12)
            .overlay(
                RoundedRectangle(cornerRadius: 12)
                    .stroke(Color.darkBorder, lineWidth: 1)
            )
    }
}

// MARK: - Date Extensions

extension Date {
    var unixTimestamp: Int64 {
        Int64(timeIntervalSince1970)
    }

    static func fromUnixTimestamp(_ timestamp: Int64) -> Date {
        Date(timeIntervalSince1970: TimeInterval(timestamp))
    }

    static func fromUnixTimestamp(_ timestamp: UInt64) -> Date {
        Date(timeIntervalSince1970: TimeInterval(timestamp))
    }
}
