import Foundation

enum BatteryService {
    static func getBatteryInfo() -> BatteryInfo? {
        let result = ShellHelper.run("/usr/sbin/ioreg", arguments: ["-r", "-c", "AppleSmartBattery", "-d", "1"])
        guard result.exitCode == 0 else { return nil }

        let text = result.output

        guard text.contains("BatteryInstalled"), text.contains("\"BatteryInstalled\" = Yes") else {
            return nil
        }

        let isCharging = extractBool(text, key: "IsCharging")
        let isFullyCharged = extractBool(text, key: "FullyCharged")
        let externalConnected = extractBool(text, key: "ExternalConnected")

        let currentCapacity = extractInt(text, key: "\"CurrentCapacity\"") ?? 0
        let rawMaxCapacity = extractInt(text, key: "\"AppleRawMaxCapacity\"") ?? 0
        let designCapacity = extractInt(text, key: "\"DesignCapacity\"") ?? 0
        let cycleCount = extractInt(text, key: "\"CycleCount\"") ?? 0

        // Temperature in deciKelvin -> Celsius
        let tempRaw = extractInt(text, key: "\"Temperature\"") ?? 2932
        let temperature = Float(tempRaw) / 10.0 - 273.15

        // Voltage in mV -> V
        let voltageMv = extractInt(text, key: "\"Voltage\"") ?? 0
        let voltage = Float(voltageMv) / 1000.0

        let amperage = Int32(extractInt(text, key: "\"Amperage\"") ?? 0)

        let timeRemainingMins = extractInt(text, key: "\"TimeRemaining\"")

        let chargePercent = Float(currentCapacity)
        let healthPercent = designCapacity > 0
            ? Float(rawMaxCapacity) / Float(designCapacity) * 100.0
            : 100.0

        let timeRemaining: String? = timeRemainingMins.map { mins in
            if mins == 65535 {
                return "Calculando..."
            }
            let h = mins / 60
            let m = mins % 60
            return isCharging
                ? "\(h):\(String(format: "%02d", m)) hasta carga completa"
                : "\(h):\(String(format: "%02d", m)) restante"
        }

        let powerSource: String
        if externalConnected {
            powerSource = isCharging ? "Cargando" : "Conectado (no cargando)"
        } else {
            powerSource = "Batería"
        }

        let condition: String
        if healthPercent >= 80 {
            condition = "Normal"
        } else if healthPercent >= 60 {
            condition = "Servicio recomendado"
        } else {
            condition = "Reemplazar pronto"
        }

        return BatteryInfo(
            isPresent: true,
            isCharging: isCharging,
            isFullyCharged: isFullyCharged,
            chargePercent: chargePercent,
            cycleCount: UInt32(cycleCount),
            maxCapacity: UInt32(rawMaxCapacity),
            designCapacity: UInt32(designCapacity),
            healthPercent: healthPercent,
            temperature: temperature,
            voltage: voltage,
            amperage: amperage,
            timeRemaining: timeRemaining,
            powerSource: powerSource,
            condition: condition
        )
    }

    private static func extractBool(_ text: String, key: String) -> Bool {
        for line in text.components(separatedBy: "\n") {
            if line.contains("\"\(key)\"") {
                return line.contains("= Yes")
            }
        }
        return false
    }

    private static func extractInt(_ text: String, key: String) -> Int64? {
        for line in text.components(separatedBy: "\n") {
            if line.contains(key), line.contains("=") {
                let parts = line.components(separatedBy: "=")
                if parts.count >= 2 {
                    let trimmed = parts[1].trimmingCharacters(in: .whitespaces)
                    return Int64(trimmed)
                }
            }
        }
        return nil
    }
}
