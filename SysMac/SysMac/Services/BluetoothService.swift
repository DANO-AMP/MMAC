import Foundation

enum BluetoothService {
    static func getBluetoothInfo() -> Result<BluetoothInfo, ServiceError> {
        let result = ShellHelper.run("/usr/sbin/system_profiler", arguments: ["SPBluetoothDataType", "-json"])
        guard result.exitCode == 0 else {
            return .failure(ServiceError("system_profiler falló"))
        }
        guard let data = result.output.data(using: .utf8),
              let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any] else {
            return .failure(ServiceError("Error al parsear JSON de Bluetooth"))
        }

        return parseBluetoothData(json)
    }

    private static func parseBluetoothData(_ json: [String: Any]) -> Result<BluetoothInfo, ServiceError> {
        guard let btArray = json["SPBluetoothDataType"] as? [[String: Any]],
              let btData = btArray.first else {
            return .failure(ServiceError("No se encontraron datos de Bluetooth"))
        }

        let controller = btData["controller_properties"] as? [String: Any]

        let enabled = (controller?["controller_state"] as? String) == "attrib_on"
        let discoverable = (controller?["controller_discoverable"] as? String) == "attrib_on"
        let address = controller?["controller_address"] as? String

        var devices: [BluetoothDevice] = []

        // Connected device keys
        for key in ["device_connected", "devices_list", "device_title"] {
            if let deviceList = btData[key] {
                parseDevices(from: deviceList, into: &devices, connected: true)
            }
        }

        // Not connected but paired
        if let pairedDevices = btData["device_not_connected"] {
            parseDevices(from: pairedDevices, into: &devices, connected: false)
        }

        return .success(BluetoothInfo(enabled: enabled, discoverable: discoverable, address: address, devices: devices))
    }

    private static func parseDevices(from value: Any, into devices: inout [BluetoothDevice], connected: Bool) {
        if let arr = value as? [[String: Any]] {
            // Each element is {"DeviceName": {properties}} - a single-key dict
            for item in arr {
                for (name, propValue) in item {
                    if let props = propValue as? [String: Any] {
                        if var device = parseDevice(props, connected: connected) {
                            if device.name == "Unknown Device" {
                                device = BluetoothDevice(name: name, address: device.address, deviceType: device.deviceType, batteryPercent: device.batteryPercent, isConnected: device.isConnected, isPaired: device.isPaired, vendor: device.vendor)
                            }
                            devices.append(device)
                        }
                    }
                }
            }
        } else if let dict = value as? [String: [String: Any]] {
            for (name, data) in dict {
                if var device = parseDevice(data, connected: connected) {
                    if device.name == "Unknown Device" {
                        device = BluetoothDevice(name: name, address: device.address, deviceType: device.deviceType, batteryPercent: device.batteryPercent, isConnected: device.isConnected, isPaired: device.isPaired, vendor: device.vendor)
                    }
                    devices.append(device)
                }
            }
        }
    }

    private static func parseDevice(_ obj: [String: Any], connected: Bool) -> BluetoothDevice? {
        let name = (obj["device_name"] as? String) ?? (obj["_name"] as? String) ?? "Unknown Device"
        let address = (obj["device_address"] as? String) ?? ""
        let deviceType = detectDeviceType(obj)

        let batteryPercent: UInt8? = {
            for key in ["device_batteryLevelMain", "device_batteryLevel", "device_batteryPercent"] {
                if let val = obj[key] {
                    if let num = val as? Int { return UInt8(num) }
                    if let str = val as? String {
                        return UInt8(str.replacingOccurrences(of: "%", with: ""))
                    }
                }
            }
            return nil
        }()

        let isPaired: Bool = {
            if let paired = obj["device_paired"] as? String {
                return paired == "attrib_Yes" || paired == "Yes"
            }
            return true
        }()

        let vendor = (obj["device_manufacturer"] as? String) ?? (obj["device_vendorID"] as? String)

        return BluetoothDevice(name: name, address: address, deviceType: deviceType, batteryPercent: batteryPercent, isConnected: connected, isPaired: isPaired, vendor: vendor)
    }

    private static func detectDeviceType(_ obj: [String: Any]) -> String {
        // Check minor type
        if let minorType = (obj["device_minorType"] as? String) ?? (obj["device_minorClassOfDevice"] as? String) {
            return normalizeDeviceType(minorType)
        }

        // Check services
        if let services = obj["device_services"] as? [String] {
            for service in services {
                let lower = service.lowercased()
                if lower.contains("audio") || lower.contains("headset") || lower.contains("handsfree") { return "Headphones" }
                if lower.contains("keyboard") { return "Keyboard" }
                if lower.contains("mouse") || lower.contains("pointing") { return "Mouse" }
            }
        }

        // Check name
        if let name = (obj["device_name"] as? String) ?? (obj["_name"] as? String) {
            let lower = name.lowercased()
            if lower.contains("airpods") || lower.contains("headphone") || lower.contains("earbuds") || lower.contains("beats") { return "Headphones" }
            if lower.contains("keyboard") { return "Keyboard" }
            if lower.contains("mouse") || lower.contains("trackpad") { return "Mouse" }
            if lower.contains("watch") { return "Watch" }
            if lower.contains("iphone") || lower.contains("ipad") { return "iOS Device" }
            if lower.contains("speaker") || lower.contains("homepod") { return "Speaker" }
        }

        return "Other"
    }

    private static func normalizeDeviceType(_ type: String) -> String {
        let lower = type.lowercased()
        if lower.contains("headphone") || lower.contains("headset") || lower.contains("audio") { return "Headphones" }
        if lower.contains("keyboard") { return "Keyboard" }
        if lower.contains("mouse") || lower.contains("pointing") { return "Mouse" }
        if lower.contains("gamepad") || lower.contains("joystick") { return "Controller" }
        if lower.contains("phone") { return "Phone" }
        if lower.contains("computer") { return "Computer" }
        return type
    }
}
