import SwiftUI

struct ProgressRing: View {
    let progress: Double  // 0.0 to 1.0
    var lineWidth: CGFloat = 8
    var size: CGFloat = 60
    var color: Color = .primary500

    var body: some View {
        ZStack {
            Circle()
                .stroke(.quaternary, lineWidth: lineWidth)
            Circle()
                .trim(from: 0, to: CGFloat(min(progress, 1.0)))
                .stroke(color, style: StrokeStyle(lineWidth: lineWidth, lineCap: .round))
                .rotationEffect(.degrees(-90))
                .animation(.easeInOut(duration: 0.3), value: progress)
        }
        .frame(width: size, height: size)
    }
}
