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
import { useBattery } from "../store/AppStore";

export default function BatteryView() {
  const { battery, isLoading, refresh, lastUpdated } = useBattery();

  const getBatteryIcon = () => {
    if (!battery) return Battery;
    if (battery.is_charging) return BatteryCharging;
    if (battery.percentage >= 80) return BatteryFull;
    if (battery.percentage >= 40) return BatteryMedium;
    return BatteryLow;
  };

  const getBatteryColor = () => {
    if (!battery) return "text-gray-400";
    if (battery.is_charging) return "text-green-400";
    if (battery.percentage >= 50) return "text-green-400";
    if (battery.percentage >= 20) return "text-yellow-400";
    return "text-red-400";
  };

  const getHealthColor = (percent: number) => {
    if (percent >= 80) return "text-green-400";
    if (percent >= 60) return "text-yellow-400";
    return "text-red-400";
  };

  const BatteryIcon = getBatteryIcon();

  // No battery detected
  if (!battery && !isLoading) {
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

  // Calculate health percent
  const healthPercent = battery
    ? (battery.max_capacity / battery.design_capacity) * 100
    : 0;

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
            {lastUpdated > 0 && (
              <span className="ml-2 text-xs">
                (actualizado: {new Date(lastUpdated).toLocaleTimeString("es-ES")})
              </span>
            )}
          </p>
        </div>
        <button
          onClick={refresh}
          disabled={isLoading}
          className="flex items-center gap-2 px-4 py-2 bg-dark-card border border-dark-border rounded-lg hover:bg-dark-border transition-colors disabled:opacity-50"
        >
          <RefreshCw size={18} className={isLoading ? "animate-spin" : ""} />
          Actualizar
        </button>
      </div>

      {battery && (
        <>
          {/* Main battery status */}
          <div className="bg-dark-card rounded-xl border border-dark-border p-6">
            <div className="flex items-center gap-6">
              <div className={`${getBatteryColor()}`}>
                <BatteryIcon size={64} />
              </div>
              <div className="flex-1">
                <div className="flex items-baseline gap-3">
                  <span className="text-5xl font-bold">
                    {Math.round(battery.percentage)}%
                  </span>
                  <span className="text-gray-400">
                    {battery.is_charging ? "Cargando" : "Batería"}
                  </span>
                </div>
                {battery.time_remaining && battery.time_remaining !== "Calculando..." && (
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
                      : battery.percentage >= 20
                      ? "bg-primary-500"
                      : "bg-red-500"
                  }`}
                  style={{ width: `${battery.percentage}%` }}
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
              <p className={`text-2xl font-bold ${getHealthColor(healthPercent)}`}>
                {healthPercent.toFixed(1)}%
              </p>
              <p className="text-sm text-gray-400 mt-1">{battery.health}</p>
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
                <span className="text-sm">Potencia</span>
              </div>
              <p className="text-2xl font-bold">
                {battery.wattage.toFixed(1)} W
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
                <span>{(battery.voltage / 1000).toFixed(2)} V</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-400">Corriente</span>
                <span>{battery.amperage} mA</span>
              </div>
            </div>
          </div>
        </>
      )}
    </div>
  );
}
