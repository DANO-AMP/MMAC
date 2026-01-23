import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Battery,
  BatteryCharging,
  BatteryFull,
  BatteryLow,
  BatteryMedium,
  Thermometer,
  Zap,
  Heart,
  RefreshCw,
  Clock,
} from "lucide-react";

interface BatteryInfo {
  is_present: boolean;
  is_charging: boolean;
  is_fully_charged: boolean;
  charge_percent: number;
  cycle_count: number;
  max_capacity: number;
  design_capacity: number;
  health_percent: number;
  temperature: number;
  voltage: number;
  amperage: number;
  time_remaining: string | null;
  power_source: string;
  condition: string;
}

export default function BatteryView() {
  const [battery, setBattery] = useState<BatteryInfo | null>(null);
  const [loading, setLoading] = useState(true);

  const loadBatteryInfo = async () => {
    try {
      const info = await invoke<BatteryInfo | null>("get_battery_info");
      setBattery(info);
    } catch (error) {
      console.error("Error loading battery info:", error);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadBatteryInfo();
    const interval = setInterval(loadBatteryInfo, 5000);
    return () => clearInterval(interval);
  }, []);

  const getBatteryIcon = () => {
    if (!battery) return Battery;
    if (battery.is_charging) return BatteryCharging;
    if (battery.charge_percent >= 80) return BatteryFull;
    if (battery.charge_percent >= 40) return BatteryMedium;
    return BatteryLow;
  };

  const getBatteryColor = () => {
    if (!battery) return "text-gray-400";
    if (battery.is_charging) return "text-green-400";
    if (battery.charge_percent >= 50) return "text-green-400";
    if (battery.charge_percent >= 20) return "text-yellow-400";
    return "text-red-400";
  };

  const getHealthColor = (percent: number) => {
    if (percent >= 80) return "text-green-400";
    if (percent >= 60) return "text-yellow-400";
    return "text-red-400";
  };

  const BatteryIcon = getBatteryIcon();

  if (loading) {
    return (
      <div className="p-6 flex items-center justify-center h-full">
        <RefreshCw className="animate-spin text-primary-400" size={32} />
      </div>
    );
  }

  if (!battery) {
    return (
      <div className="p-6">
        <h2 className="text-2xl font-bold flex items-center gap-3">
          <Battery className="text-primary-400" />
          Batería
        </h2>
        <div className="mt-8 text-center py-12 text-gray-400">
          <Battery size={64} className="mx-auto mb-4 opacity-50" />
          <p>No se detectó batería</p>
          <p className="text-sm mt-2">Este Mac no tiene batería o es un Mac de escritorio</p>
        </div>
      </div>
    );
  }

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold flex items-center gap-3">
            <Battery className="text-primary-400" />
            Batería
          </h2>
          <p className="text-gray-400 mt-1">
            Estado y salud de la batería de tu Mac
          </p>
        </div>
        <button
          onClick={loadBatteryInfo}
          className="flex items-center gap-2 px-4 py-2 bg-dark-card border border-dark-border rounded-lg hover:bg-dark-border transition-colors"
        >
          <RefreshCw size={18} />
          Actualizar
        </button>
      </div>

      {/* Main battery status */}
      <div className="bg-dark-card rounded-xl border border-dark-border p-6">
        <div className="flex items-center gap-6">
          <div className={`${getBatteryColor()}`}>
            <BatteryIcon size={64} />
          </div>
          <div className="flex-1">
            <div className="flex items-baseline gap-3">
              <span className="text-5xl font-bold">
                {Math.round(battery.charge_percent)}%
              </span>
              <span className="text-gray-400">{battery.power_source}</span>
            </div>
            {battery.time_remaining && (
              <div className="flex items-center gap-2 mt-2 text-gray-400">
                <Clock size={16} />
                {battery.time_remaining}
              </div>
            )}
          </div>
        </div>

        {/* Progress bar */}
        <div className="mt-6">
          <div className="h-4 bg-dark-bg rounded-full overflow-hidden">
            <div
              className={`h-full transition-all ${
                battery.is_charging
                  ? "bg-green-500"
                  : battery.charge_percent >= 20
                  ? "bg-primary-500"
                  : "bg-red-500"
              }`}
              style={{ width: `${battery.charge_percent}%` }}
            />
          </div>
        </div>
      </div>

      {/* Stats grid */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <div className="bg-dark-card rounded-xl border border-dark-border p-4">
          <div className="flex items-center gap-2 text-gray-400 mb-2">
            <Heart size={18} />
            <span className="text-sm">Salud</span>
          </div>
          <p className={`text-2xl font-bold ${getHealthColor(battery.health_percent)}`}>
            {battery.health_percent.toFixed(1)}%
          </p>
          <p className="text-sm text-gray-400 mt-1">{battery.condition}</p>
        </div>

        <div className="bg-dark-card rounded-xl border border-dark-border p-4">
          <div className="flex items-center gap-2 text-gray-400 mb-2">
            <RefreshCw size={18} />
            <span className="text-sm">Ciclos</span>
          </div>
          <p className="text-2xl font-bold">{battery.cycle_count}</p>
          <p className="text-sm text-gray-400 mt-1">de ~1000 máximo</p>
        </div>

        <div className="bg-dark-card rounded-xl border border-dark-border p-4">
          <div className="flex items-center gap-2 text-gray-400 mb-2">
            <Thermometer size={18} />
            <span className="text-sm">Temperatura</span>
          </div>
          <p className="text-2xl font-bold">{battery.temperature.toFixed(1)}°C</p>
          <p className="text-sm text-gray-400 mt-1">
            {battery.temperature < 35 ? "Normal" : "Elevada"}
          </p>
        </div>

        <div className="bg-dark-card rounded-xl border border-dark-border p-4">
          <div className="flex items-center gap-2 text-gray-400 mb-2">
            <Zap size={18} />
            <span className="text-sm">Corriente</span>
          </div>
          <p className="text-2xl font-bold">
            {battery.amperage > 0 ? "+" : ""}
            {battery.amperage} mA
          </p>
          <p className="text-sm text-gray-400 mt-1">
            {battery.amperage > 0 ? "Cargando" : "Descargando"}
          </p>
        </div>
      </div>

      {/* Detailed info */}
      <div className="bg-dark-card rounded-xl border border-dark-border p-4">
        <h3 className="font-semibold mb-4">Información Detallada</h3>
        <div className="grid grid-cols-2 gap-4 text-sm">
          <div className="flex justify-between">
            <span className="text-gray-400">Capacidad actual</span>
            <span>{battery.max_capacity} mAh</span>
          </div>
          <div className="flex justify-between">
            <span className="text-gray-400">Capacidad de diseño</span>
            <span>{battery.design_capacity} mAh</span>
          </div>
          <div className="flex justify-between">
            <span className="text-gray-400">Voltaje</span>
            <span>{battery.voltage.toFixed(2)} V</span>
          </div>
          <div className="flex justify-between">
            <span className="text-gray-400">Completamente cargada</span>
            <span>{battery.is_fully_charged ? "Sí" : "No"}</span>
          </div>
        </div>
      </div>
    </div>
  );
}
