import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Bluetooth,
  BluetoothConnected,
  BluetoothOff,
  RefreshCw,
  Battery,
  Headphones,
  Keyboard,
  Mouse,
  Watch,
  Smartphone,
  Speaker,
  Gamepad2,
  Monitor,
  HelpCircle,
} from "lucide-react";
import { ErrorBanner } from "../components/ErrorBanner";

interface BluetoothDevice {
  name: string;
  address: string;
  device_type: string;
  battery_percent: number | null;
  is_connected: boolean;
  is_paired: boolean;
  vendor: string | null;
}

interface BluetoothInfo {
  enabled: boolean;
  discoverable: boolean;
  address: string | null;
  devices: BluetoothDevice[];
}

function BluetoothView() {
  const [bluetoothInfo, setBluetoothInfo] = useState<BluetoothInfo | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadData = async (refresh = false) => {
    if (refresh) {
      setIsRefreshing(true);
    } else {
      setIsLoading(true);
    }
    setError(null);

    try {
      const info: BluetoothInfo = await invoke("get_bluetooth_info");
      setBluetoothInfo(info);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsLoading(false);
      setIsRefreshing(false);
    }
  };

  useEffect(() => {
    loadData();
  }, []);

  const getDeviceIcon = (type: string) => {
    const iconProps = { size: 24 };
    switch (type.toLowerCase()) {
      case "headphones":
        return <Headphones {...iconProps} />;
      case "keyboard":
        return <Keyboard {...iconProps} />;
      case "mouse":
        return <Mouse {...iconProps} />;
      case "watch":
        return <Watch {...iconProps} />;
      case "ios device":
      case "phone":
        return <Smartphone {...iconProps} />;
      case "speaker":
        return <Speaker {...iconProps} />;
      case "controller":
        return <Gamepad2 {...iconProps} />;
      case "computer":
        return <Monitor {...iconProps} />;
      default:
        return <HelpCircle {...iconProps} />;
    }
  };

  const getBatteryColor = (percent: number) => {
    if (percent >= 60) return "text-green-400";
    if (percent >= 30) return "text-yellow-400";
    return "text-red-400";
  };

  if (isLoading) {
    return (
      <div className="p-6 flex items-center justify-center h-full">
        <div className="text-center">
          <RefreshCw size={32} className="animate-spin mx-auto text-primary-400 mb-4" />
          <p className="text-gray-400">Cargando dispositivos Bluetooth...</p>
        </div>
      </div>
    );
  }

  const connectedDevices = bluetoothInfo?.devices.filter(d => d.is_connected) || [];
  const pairedDevices = bluetoothInfo?.devices.filter(d => !d.is_connected && d.is_paired) || [];

  return (
    <div className="p-6">
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <h2 className="text-2xl font-bold">Bluetooth</h2>
          <p className="text-gray-400 mt-1">
            Dispositivos conectados y emparejados
          </p>
        </div>
        <button
          onClick={() => loadData(true)}
          disabled={isRefreshing}
          className="flex items-center gap-2 px-4 py-2 bg-dark-card border border-dark-border rounded-lg hover:bg-dark-border transition-colors disabled:opacity-50"
        >
          <RefreshCw size={18} className={isRefreshing ? "animate-spin" : ""} />
          <span>Actualizar</span>
        </button>
      </div>

      {/* Error banner */}
      {error && (
        <ErrorBanner error={error} onRetry={() => loadData()} className="mb-6" />
      )}

      {/* Bluetooth status card */}
      <div className={`bg-gradient-to-r ${
        bluetoothInfo?.enabled
          ? "from-blue-600/20 to-blue-800/20 border-blue-500/30"
          : "from-gray-600/20 to-gray-800/20 border-gray-500/30"
      } border rounded-xl p-6 mb-6`}>
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4">
            <div className={`p-3 rounded-xl ${
              bluetoothInfo?.enabled ? "bg-blue-500/20" : "bg-gray-500/20"
            }`}>
              {bluetoothInfo?.enabled ? (
                <BluetoothConnected size={32} className="text-blue-400" />
              ) : (
                <BluetoothOff size={32} className="text-gray-400" />
              )}
            </div>
            <div>
              <h3 className="text-xl font-bold">
                {bluetoothInfo?.enabled ? "Bluetooth Activado" : "Bluetooth Desactivado"}
              </h3>
              {bluetoothInfo?.address && (
                <p className="text-sm text-gray-400">
                  Direccion: {bluetoothInfo.address}
                </p>
              )}
              {bluetoothInfo?.discoverable && (
                <p className="text-sm text-blue-400">Visible para otros dispositivos</p>
              )}
            </div>
          </div>
          <div className="text-right">
            <p className="text-3xl font-bold text-blue-400">{connectedDevices.length}</p>
            <p className="text-sm text-gray-400">Conectados</p>
          </div>
        </div>
      </div>

      {/* Connected Devices */}
      {connectedDevices.length > 0 && (
        <div className="mb-6">
          <h3 className="text-lg font-semibold mb-3 flex items-center gap-2">
            <BluetoothConnected size={20} className="text-blue-400" />
            Dispositivos Conectados
          </h3>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {connectedDevices.map((device, index) => (
              <div
                key={`${device.address}-${index}`}
                className="bg-dark-card border border-blue-500/30 rounded-xl p-4"
              >
                <div className="flex items-start gap-4">
                  <div className="p-3 bg-blue-500/20 rounded-lg text-blue-400">
                    {getDeviceIcon(device.device_type)}
                  </div>
                  <div className="flex-1 min-w-0">
                    <h4 className="font-semibold truncate">{device.name}</h4>
                    <p className="text-sm text-gray-400">{device.device_type}</p>
                    {device.vendor && (
                      <p className="text-xs text-gray-500">{device.vendor}</p>
                    )}
                    {device.address && (
                      <p className="text-xs text-gray-600 font-mono mt-1">{device.address}</p>
                    )}
                  </div>
                  {device.battery_percent !== null && (
                    <div className="flex items-center gap-1">
                      <Battery size={18} className={getBatteryColor(device.battery_percent)} />
                      <span className={`text-sm font-medium ${getBatteryColor(device.battery_percent)}`}>
                        {device.battery_percent}%
                      </span>
                    </div>
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Paired Devices (not connected) */}
      {pairedDevices.length > 0 && (
        <div>
          <h3 className="text-lg font-semibold mb-3 flex items-center gap-2">
            <Bluetooth size={20} className="text-gray-400" />
            Dispositivos Emparejados
          </h3>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {pairedDevices.map((device, index) => (
              <div
                key={`${device.address}-${index}`}
                className="bg-dark-card border border-dark-border rounded-xl p-4 opacity-70"
              >
                <div className="flex items-start gap-4">
                  <div className="p-3 bg-dark-border rounded-lg text-gray-400">
                    {getDeviceIcon(device.device_type)}
                  </div>
                  <div className="flex-1 min-w-0">
                    <h4 className="font-semibold truncate">{device.name}</h4>
                    <p className="text-sm text-gray-400">{device.device_type}</p>
                    {device.vendor && (
                      <p className="text-xs text-gray-500">{device.vendor}</p>
                    )}
                    {device.address && (
                      <p className="text-xs text-gray-600 font-mono mt-1">{device.address}</p>
                    )}
                  </div>
                  <span className="text-xs text-gray-500 px-2 py-1 bg-dark-border rounded">
                    No conectado
                  </span>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* No devices */}
      {bluetoothInfo?.devices.length === 0 && (
        <div className="bg-dark-card border border-dark-border rounded-xl p-8 text-center">
          <Bluetooth size={48} className="mx-auto text-gray-500 mb-4" />
          <h3 className="text-lg font-medium mb-2">No hay dispositivos</h3>
          <p className="text-gray-400">
            No se encontraron dispositivos Bluetooth emparejados o conectados
          </p>
        </div>
      )}
    </div>
  );
}

export default BluetoothView;
