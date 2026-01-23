import { useState, useEffect, useRef } from "react";
import {
  Cpu,
  MemoryStick,
  HardDrive,
  Wifi,
  Thermometer,
  Activity,
  Fan,
  ArrowDown,
  ArrowUp,
  Monitor,
} from "lucide-react";
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  ResponsiveContainer,
  Tooltip,
} from "recharts";
import { formatSize, formatSpeed } from "../utils";
import { ErrorBanner } from "../components/ErrorBanner";
import { useSystemStats } from "../store/AppStore";

interface ChartData {
  time: string;
  cpu: number;
  memory: number;
  network_rx: number;
  network_tx: number;
}

function MonitorView() {
  const { stats, isLoading, error, lastUpdated, refresh } = useSystemStats();
  const [chartData, setChartData] = useState<ChartData[]>([]);
  const [isLive, setIsLive] = useState(true);
  const lastChartUpdateRef = useRef<number>(0);

  // Update chart data when stats change
  useEffect(() => {
    if (!stats || !isLive) return;

    // Prevent duplicate entries for same timestamp
    if (lastUpdated === lastChartUpdateRef.current) return;
    lastChartUpdateRef.current = lastUpdated;

    const now = new Date();
    const timeStr = now.toLocaleTimeString("es-ES", {
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    });

    setChartData((prev) => {
      const newData = [
        ...prev,
        {
          time: timeStr,
          cpu: stats.cpu_usage,
          memory: (stats.memory_used / stats.memory_total) * 100,
          network_rx: stats.network_rx,
          network_tx: stats.network_tx,
        },
      ];
      return newData.slice(-30);
    });
  }, [stats, lastUpdated, isLive]);

  // Default values for initial render
  const currentStats = stats || {
    cpu_usage: 0,
    memory_used: 0,
    memory_total: 16 * 1024 * 1024 * 1024,
    disk_used: 0,
    disk_total: 500 * 1024 * 1024 * 1024,
    network_rx: 0,
    network_tx: 0,
    cpu_temp: 0,
    fan_speed: null,
    disk_read_speed: 0,
    disk_write_speed: 0,
    gpu_name: null,
    gpu_vendor: null,
  };

  const memoryPercent = (currentStats.memory_used / currentStats.memory_total) * 100;
  const diskPercent = (currentStats.disk_used / currentStats.disk_total) * 100;

  return (
    <div className="p-6">
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <h2 className="text-2xl font-bold">Monitor del Sistema</h2>
          <p className="text-gray-400 mt-1">
            Metricas en tiempo real de tu Mac
            {lastUpdated > 0 && (
              <span className="ml-2 text-xs">
                (actualizado: {new Date(lastUpdated).toLocaleTimeString("es-ES")})
              </span>
            )}
          </p>
        </div>
        <div className="flex items-center gap-2">
          {isLoading && (
            <div className="w-2 h-2 rounded-full bg-blue-500 animate-pulse" />
          )}
          <button
            onClick={() => setIsLive(!isLive)}
            className={`flex items-center gap-2 px-4 py-2 rounded-lg transition-colors ${
              isLive
                ? "bg-green-500/20 text-green-400 border border-green-500/30"
                : "bg-dark-card border border-dark-border text-gray-400"
            }`}
          >
            <div
              className={`w-2 h-2 rounded-full ${
                isLive ? "bg-green-500 animate-pulse" : "bg-gray-500"
              }`}
            />
            <span>{isLive ? "En vivo" : "Pausado"}</span>
          </button>
        </div>
      </div>

      {/* Error banner */}
      {error && (
        <ErrorBanner
          error={error}
          onRetry={refresh}
          className="mb-6"
        />
      )}

      {/* Stats cards */}
      <div className="grid grid-cols-4 gap-4 mb-6">
        {/* CPU */}
        <div className="bg-dark-card border border-dark-border rounded-xl p-4">
          <div className="flex items-center gap-3 mb-3">
            <div className="p-2 bg-blue-500/20 rounded-lg">
              <Cpu size={20} className="text-blue-400" />
            </div>
            <span className="text-gray-400 text-sm">CPU</span>
          </div>
          <p className="text-3xl font-bold">{currentStats.cpu_usage.toFixed(1)}%</p>
          <div className="mt-2 h-1.5 bg-dark-bg rounded-full overflow-hidden">
            <div
              className="h-full bg-blue-500 transition-all duration-300"
              style={{ width: `${currentStats.cpu_usage}%` }}
            />
          </div>
        </div>

        {/* Memory */}
        <div className="bg-dark-card border border-dark-border rounded-xl p-4">
          <div className="flex items-center gap-3 mb-3">
            <div className="p-2 bg-purple-500/20 rounded-lg">
              <MemoryStick size={20} className="text-purple-400" />
            </div>
            <span className="text-gray-400 text-sm">Memoria</span>
          </div>
          <p className="text-3xl font-bold">{memoryPercent.toFixed(1)}%</p>
          <p className="text-sm text-gray-400 mt-1">
            {formatSize(currentStats.memory_used)} / {formatSize(currentStats.memory_total)}
          </p>
          <div className="mt-2 h-1.5 bg-dark-bg rounded-full overflow-hidden">
            <div
              className="h-full bg-purple-500 transition-all duration-300"
              style={{ width: `${memoryPercent}%` }}
            />
          </div>
        </div>

        {/* Disk */}
        <div className="bg-dark-card border border-dark-border rounded-xl p-4">
          <div className="flex items-center gap-3 mb-3">
            <div className="p-2 bg-orange-500/20 rounded-lg">
              <HardDrive size={20} className="text-orange-400" />
            </div>
            <span className="text-gray-400 text-sm">Disco</span>
          </div>
          <p className="text-3xl font-bold">{diskPercent.toFixed(1)}%</p>
          <p className="text-sm text-gray-400 mt-1">
            {formatSize(currentStats.disk_used)} / {formatSize(currentStats.disk_total)}
          </p>
          <div className="mt-2 h-1.5 bg-dark-bg rounded-full overflow-hidden">
            <div
              className="h-full bg-orange-500 transition-all duration-300"
              style={{ width: `${diskPercent}%` }}
            />
          </div>
        </div>

        {/* Temperature */}
        <div className="bg-dark-card border border-dark-border rounded-xl p-4">
          <div className="flex items-center gap-3 mb-3">
            <div className="p-2 bg-red-500/20 rounded-lg">
              <Thermometer size={20} className="text-red-400" />
            </div>
            <span className="text-gray-400 text-sm">Temperatura</span>
          </div>
          <p className="text-3xl font-bold">{currentStats.cpu_temp.toFixed(0)}C</p>
          <p className="text-sm text-gray-400 mt-1">
            {currentStats.cpu_temp < 50
              ? "Normal"
              : currentStats.cpu_temp < 70
              ? "Moderada"
              : "Alta"}
          </p>
        </div>
      </div>

      {/* Charts */}
      <div className="grid grid-cols-2 gap-4">
        {/* CPU Chart */}
        <div className="bg-dark-card border border-dark-border rounded-xl p-4">
          <div className="flex items-center gap-2 mb-4">
            <Activity size={18} className="text-blue-400" />
            <h3 className="font-semibold">Uso de CPU</h3>
          </div>
          <div className="h-48">
            <ResponsiveContainer width="100%" height="100%">
              <LineChart data={chartData}>
                <XAxis
                  dataKey="time"
                  stroke="#6b7280"
                  fontSize={10}
                  tickLine={false}
                />
                <YAxis
                  stroke="#6b7280"
                  fontSize={10}
                  tickLine={false}
                  domain={[0, 100]}
                />
                <Tooltip
                  contentStyle={{
                    backgroundColor: "#16213e",
                    border: "1px solid #0f3460",
                    borderRadius: "8px",
                  }}
                />
                <Line
                  type="monotone"
                  dataKey="cpu"
                  stroke="#3b82f6"
                  strokeWidth={2}
                  dot={false}
                />
              </LineChart>
            </ResponsiveContainer>
          </div>
        </div>

        {/* Memory Chart */}
        <div className="bg-dark-card border border-dark-border rounded-xl p-4">
          <div className="flex items-center gap-2 mb-4">
            <MemoryStick size={18} className="text-purple-400" />
            <h3 className="font-semibold">Uso de Memoria</h3>
          </div>
          <div className="h-48">
            <ResponsiveContainer width="100%" height="100%">
              <LineChart data={chartData}>
                <XAxis
                  dataKey="time"
                  stroke="#6b7280"
                  fontSize={10}
                  tickLine={false}
                />
                <YAxis
                  stroke="#6b7280"
                  fontSize={10}
                  tickLine={false}
                  domain={[0, 100]}
                />
                <Tooltip
                  contentStyle={{
                    backgroundColor: "#16213e",
                    border: "1px solid #0f3460",
                    borderRadius: "8px",
                  }}
                />
                <Line
                  type="monotone"
                  dataKey="memory"
                  stroke="#a855f7"
                  strokeWidth={2}
                  dot={false}
                />
              </LineChart>
            </ResponsiveContainer>
          </div>
        </div>
      </div>

      {/* Network stats */}
      <div className="mt-4 bg-dark-card border border-dark-border rounded-xl p-4">
        <div className="flex items-center gap-2 mb-4">
          <Wifi size={18} className="text-green-400" />
          <h3 className="font-semibold">Red</h3>
        </div>
        <div className="grid grid-cols-2 gap-8">
          <div className="flex items-center gap-4">
            <div className="text-green-400">Descarga</div>
            <div className="text-2xl font-bold">
              {formatSpeed(currentStats.network_rx)}
            </div>
          </div>
          <div className="flex items-center gap-4">
            <div className="text-blue-400">Subida</div>
            <div className="text-2xl font-bold">
              {formatSpeed(currentStats.network_tx)}
            </div>
          </div>
        </div>
      </div>

      {/* Fan and Disk I/O stats */}
      <div className="mt-4 grid grid-cols-2 gap-4">
        {/* Fan Speed */}
        <div className="bg-dark-card border border-dark-border rounded-xl p-4">
          <div className="flex items-center gap-2 mb-4">
            <Fan size={18} className="text-cyan-400" />
            <h3 className="font-semibold">Ventilador</h3>
          </div>
          <div className="flex items-center justify-between">
            <div>
              <p className="text-3xl font-bold">
                {currentStats.fan_speed !== null ? `${currentStats.fan_speed}` : "--"}
              </p>
              <p className="text-sm text-gray-400">RPM</p>
            </div>
            {currentStats.fan_speed !== null && (
              <div className="text-right text-sm text-gray-400">
                {currentStats.fan_speed < 2000 ? "Bajo" : currentStats.fan_speed < 4000 ? "Normal" : "Alto"}
              </div>
            )}
          </div>
        </div>

        {/* Disk I/O */}
        <div className="bg-dark-card border border-dark-border rounded-xl p-4">
          <div className="flex items-center gap-2 mb-4">
            <HardDrive size={18} className="text-orange-400" />
            <h3 className="font-semibold">Disco I/O</h3>
          </div>
          <div className="grid grid-cols-2 gap-4">
            <div className="flex items-center gap-2">
              <ArrowDown size={16} className="text-green-400" />
              <div>
                <p className="text-lg font-bold">{formatSpeed(currentStats.disk_read_speed)}</p>
                <p className="text-xs text-gray-400">Lectura</p>
              </div>
            </div>
            <div className="flex items-center gap-2">
              <ArrowUp size={16} className="text-blue-400" />
              <div>
                <p className="text-lg font-bold">{formatSpeed(currentStats.disk_write_speed)}</p>
                <p className="text-xs text-gray-400">Escritura</p>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* GPU Info */}
      {currentStats.gpu_name && (
        <div className="mt-4 bg-dark-card border border-dark-border rounded-xl p-4">
          <div className="flex items-center gap-2 mb-4">
            <Monitor size={18} className="text-pink-400" />
            <h3 className="font-semibold">GPU</h3>
          </div>
          <div className="flex items-center justify-between">
            <div>
              <p className="text-xl font-bold">{currentStats.gpu_name}</p>
              {currentStats.gpu_vendor && (
                <p className="text-sm text-gray-400">{currentStats.gpu_vendor}</p>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default MonitorView;
