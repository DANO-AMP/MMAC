import Foundation

enum HomebrewService {
    static func checkHomebrew() -> HomebrewInfo {
        guard let brewPath = ShellHelper.brewPath else {
            return HomebrewInfo(installed: false, version: nil, formulaeCount: 0, casksCount: 0)
        }
        let verResult = ShellHelper.run(brewPath, arguments: ["--version"])
        guard verResult.exitCode == 0 else {
            return HomebrewInfo(installed: false, version: nil, formulaeCount: 0, casksCount: 0)
        }
        let version = verResult.output.components(separatedBy: " ").last

        let formulaeResult = ShellHelper.run(brewPath, arguments: ["list", "--formula"])
        let formulaeCount = UInt32(formulaeResult.output.components(separatedBy: "\n").filter { !$0.isEmpty }.count)

        let casksResult = ShellHelper.run(brewPath, arguments: ["list", "--cask"])
        let casksCount = UInt32(casksResult.output.components(separatedBy: "\n").filter { !$0.isEmpty }.count)

        return HomebrewInfo(installed: true, version: version, formulaeCount: formulaeCount, casksCount: casksCount)
    }

    static func listPackages() -> [BrewPackage] {
        guard let brewPath = ShellHelper.brewPath else { return [] }
        var packages: [BrewPackage] = []

        // Formulae
        let formulaeResult = ShellHelper.run(brewPath, arguments: ["list", "--formula", "--versions"])
        for line in formulaeResult.output.components(separatedBy: "\n") where !line.isEmpty {
            let parts = line.split(separator: " ", maxSplits: 1)
            guard parts.count >= 2 else { continue }
            packages.append(BrewPackage(name: String(parts[0]), version: String(parts[1]), isOutdated: false, newerVersion: nil, isCask: false))
        }

        // Casks
        let casksResult = ShellHelper.run(brewPath, arguments: ["list", "--cask", "--versions"])
        for line in casksResult.output.components(separatedBy: "\n") where !line.isEmpty {
            let parts = line.split(separator: " ", maxSplits: 1)
            guard parts.count >= 2 else { continue }
            packages.append(BrewPackage(name: String(parts[0]), version: String(parts[1]), isOutdated: false, newerVersion: nil, isCask: true))
        }

        // Check outdated
        let outdatedResult = ShellHelper.run(brewPath, arguments: ["outdated", "--verbose"])
        for line in outdatedResult.output.components(separatedBy: "\n") where !line.isEmpty {
            // Format: "package (installed) < newer"
            let parts = line.components(separatedBy: " ")
            guard let name = parts.first else { continue }
            let newerVersion = parts.last
            if let idx = packages.firstIndex(where: { $0.name == name }) {
                let pkg = packages[idx]
                packages[idx] = BrewPackage(name: pkg.name, version: pkg.version, isOutdated: true, newerVersion: newerVersion, isCask: pkg.isCask)
            }
        }

        return packages
    }

    private static func validatePackageName(_ name: String) -> Result<Void, ServiceError> {
        let allowed = CharacterSet.alphanumerics.union(CharacterSet(charactersIn: "@._-/"))
        guard !name.isEmpty, name.unicodeScalars.allSatisfy({ allowed.contains($0) }) else {
            return .failure(ServiceError("Nombre de paquete no válido: \(name)"))
        }
        return .success(())
    }

    static func upgradePackage(_ name: String) -> Result<String, ServiceError> {
        if case .failure(let error) = validatePackageName(name) {
            return .failure(error)
        }
        guard let brewPath = ShellHelper.brewPath else {
            return .failure(ServiceError("Homebrew no está instalado"))
        }
        let result = ShellHelper.run(brewPath, arguments: ["upgrade", name])
        if result.exitCode == 0 {
            return .success(result.output)
        }
        return .failure(ServiceError("Error al actualizar \(name): \(result.error)"))
    }

    static func uninstallPackage(_ name: String) -> Result<String, ServiceError> {
        if case .failure(let error) = validatePackageName(name) {
            return .failure(error)
        }
        guard let brewPath = ShellHelper.brewPath else {
            return .failure(ServiceError("Homebrew no está instalado"))
        }
        let result = ShellHelper.run(brewPath, arguments: ["uninstall", name])
        if result.exitCode == 0 {
            return .success(result.output)
        }
        return .failure(ServiceError("Error al desinstalar \(name): \(result.error)"))
    }

    static func cleanup() -> Result<String, ServiceError> {
        guard let brewPath = ShellHelper.brewPath else {
            return .failure(ServiceError("Homebrew no está instalado"))
        }
        let result = ShellHelper.run(brewPath, arguments: ["cleanup", "--prune=all"])
        if result.exitCode == 0 {
            return .success(result.output)
        }
        return .failure(ServiceError("Error en cleanup: \(result.error)"))
    }
}
