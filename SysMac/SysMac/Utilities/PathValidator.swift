import Foundation

enum PathValidator {
    private static let protectedPaths: Set<String> = [
        "/", "/System", "/usr", "/bin", "/sbin",
        "/Library", "/private", "/var", "/etc",
        "/tmp", "/dev", "/opt", "/Users",
    ]

    static func validateForDeletion(_ path: String) -> Result<URL, ServiceError> {
        let url = URL(fileURLWithPath: path)
        let resolved = url.resolvingSymlinksInPath()
        let resolvedPath = resolved.path

        let homeDir = NSHomeDirectory()

        // Don't allow deleting the home directory itself
        if resolvedPath == homeDir {
            return .failure(ServiceError("No se puede eliminar el directorio home"))
        }

        // Allow paths under home directory or /Applications
        if resolvedPath.hasPrefix(homeDir + "/") || resolvedPath.hasPrefix("/Applications/") {
            return .success(resolved)
        }

        // Check against protected paths
        for protected in protectedPaths {
            if resolvedPath == protected || resolvedPath.hasPrefix(protected + "/") {
                return .failure(ServiceError("Ruta protegida del sistema: \(resolvedPath)"))
            }
        }

        return .success(resolved)
    }
}
