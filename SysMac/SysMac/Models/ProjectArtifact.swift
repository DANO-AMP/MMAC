import Foundation

struct ProjectArtifact: Identifiable {
    let id = UUID()
    let projectPath: String
    let projectName: String
    let artifactType: String
    let artifactPath: String
    let size: UInt64
    let lastModified: String
    let isRecent: Bool
}
