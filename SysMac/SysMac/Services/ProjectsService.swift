import Foundation

enum ProjectsService {
    private static let artifactPatterns = [
        "node_modules", "target", "build", "dist", ".next",
        "__pycache__", "venv", ".venv", "vendor", "Pods",
    ]

    private static let projectDirNames = [
        "Projects", "Developer", "Development", "Code", "Sites",
        "repos", "workspace", "src", "dev", "github",
    ]

    static func scan() -> [ProjectArtifact] {
        let home = FileManager.default.homeDirectoryForCurrentUser
        let fm = FileManager.default
        var artifacts: [ProjectArtifact] = []

        for dirName in projectDirNames {
            let dirURL = home.appendingPathComponent(dirName)
            guard fm.fileExists(atPath: dirURL.path) else { continue }
            scanDirectory(dirURL, into: &artifacts, depth: 0, maxDepth: 3)
        }

        // Also scan Desktop and Documents
        for extra in ["Desktop", "Documents"] {
            let dirURL = home.appendingPathComponent(extra)
            guard fm.fileExists(atPath: dirURL.path) else { continue }
            scanDirectory(dirURL, into: &artifacts, depth: 0, maxDepth: 2)
        }

        artifacts.sort { $0.size > $1.size }
        return artifacts
    }

    private static func scanDirectory(_ url: URL, into artifacts: inout [ProjectArtifact], depth: Int, maxDepth: Int) {
        guard depth < maxDepth else { return }
        let fm = FileManager.default
        guard let contents = try? fm.contentsOfDirectory(at: url, includingPropertiesForKeys: [.isDirectoryKey]) else { return }

        for item in contents {
            guard let values = try? item.resourceValues(forKeys: [.isDirectoryKey]),
                  values.isDirectory == true else { continue }

            let name = item.lastPathComponent

            if artifactPatterns.contains(name) {
                let size = FileUtilities.directorySize(at: item)
                guard size >= 1_048_576 else { continue } // 1MB min

                let modified = (try? item.resourceValues(forKeys: [.contentModificationDateKey]).contentModificationDate) ?? Date.distantPast
                let formatter = ISO8601DateFormatter()
                let modifiedStr = formatter.string(from: modified)
                let isRecent = Date().timeIntervalSince(modified) < 7 * 24 * 3600

                artifacts.append(ProjectArtifact(
                    projectPath: url.path,
                    projectName: url.lastPathComponent,
                    artifactType: name,
                    artifactPath: item.path,
                    size: size,
                    lastModified: modifiedStr,
                    isRecent: isRecent
                ))
            } else {
                scanDirectory(item, into: &artifacts, depth: depth + 1, maxDepth: maxDepth)
            }
        }
    }

    static func deleteArtifact(path: String, moveToTrash: Bool) -> Result<UInt64, ServiceError> {
        switch PathValidator.validateForDeletion(path) {
        case .failure(let error):
            return .failure(error)
        case .success(let validatedURL):
            let fm = FileManager.default
            let size = FileUtilities.directorySize(at: validatedURL)
            do {
                if moveToTrash {
                    try fm.trashItem(at: validatedURL, resultingItemURL: nil)
                } else {
                    try fm.removeItem(at: validatedURL)
                }
                return .success(size)
            } catch {
                return .failure(ServiceError("Error al eliminar: \(error.localizedDescription)"))
            }
        }
    }
}
