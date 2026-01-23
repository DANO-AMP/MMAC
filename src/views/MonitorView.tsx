import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
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

interface SystemStats {
  cpu_usage: number;
  memory_used: number;
  memory_total: number;
  disk_used: number;
  disk_total: number;
  network_rx: number;
  network_tx: number;
  cpu_temp: number;
  fan_speed: number | null;
  disk_read_speed: number;
  disk_write_speed: number;
  gpu_name: string | null;
  gpu_vendor: string | null;
}

interface ChartData {
  time: string;
  cpu: number;
  memory: number;
  network_rx: number;
  network_tx: number;
}

function MonitorView() {
  const [stats, setStats] = useState<SystemStats>({
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
  });

  const [chartData, setChartData] = useState<ChartData[]>([]);
  const [isLive, setIsLive] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!isLive) return;
    let isMounted = true;

    const fetchStats = async () => {
      if (!isMounted) return;
      try {
        const result: SystemStats = await invoke("get_system_stats");
        if (!isMounted) return;
        setStats(result);
        setError(null);

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
              cpu: result.cpu_usage,
              memory: (result.memory_used / result.memory_total) * 100,
              network_rx: result.network_rx,
              network_tx: result.network_tx,
            },
          ];
          return newData.slice(-30);
        });
      } catch (err) {
        console.error("Failed to fetch system stats:", err);
        if (isMounted) {
          setError(err instanceof Error ? err.message : String(err));
        }
      }
    };

    fetchStats();
    const interval = setInterval(fetchStats, 2000);
    return () => {
      isMounted = false;
      clearInterval(interval);
    };
  }, [isLive]);

  const memoryPercent = (stats.memory_used / stats.memory_total) * 100;
  const diskPercent = (stats.disk_used / stats.disk_total) * 100;

  return (
    <div className="p-6">
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <h2 className="text-2xl font-bold">Monitor del Sistema</h2>
          <p className="text-gray-400 mt-1">
            Metricas en tiempo real de tu Mac
          </p>
        </div>
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

      {/* Error banner */}
      {error && (
        <ErrorBanner
          error={error}
          onRetry={() => setError(null)}
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
          <p className="text-3xl font-bold">{stats.cpu_usage.toFixed(1)}%</p>
          <div className="mt-2 h-1.5 bg-dark-bg rounded-full overflow-hidden">
            <div
              className="h-full bg-blue-500 transition-all duration-300"
              style={{ width: `${stats.cpu_usage}%` }}
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
            {formatSize(stats.memory_used)} / {formatSize(stats.memory_total)}
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
            {formatSize(stats.disk_used)} / {formatSize(stats.disk_total)}
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
          <p className="text-3xl font-bold">{stats.cpu_temp.toFixed(0)}C</p>
          <p className="text-sm text-gray-400 mt-1">
            {stats.cpu_temp < 50
              ? "Normal"
              : stats.cpu_temp < 70
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
              {formatSpeed(stats.network_rx)}
            </div>
          </div>
          <div className="flex items-center gap-4">
            <div className="text-blue-400">Subida</div>
            <div className="text-2xl font-bold">
              {formatSpeed(stats.network_tx)}
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
                {stats.fan_speed !== null ? `${stats.fan_speed}` : "--"}
              </p>
              <p className="text-sm text-gray-400">RPM</p>
            </div>
            {stats.fan_speed !== null && (
              <div className="text-right text-sm text-gray-400">
                {stats.fan_speed < 2000 ? "Bajo" : stats.fan_speed < 4000 ? "Normal" : "Alto"}
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
                <p className="text-lg font-bold">{formatSpeed(stats.disk_read_speed)}</p>
                <p className="text-xs text-gray-400">Lectura</p>
              </div>
            </div>
            <div className="flex items-center gap-2">
              <ArrowUp size={16} className="text-blue-400" />
              <div>
                <p className="text-lg font-bold">{formatSpeed(stats.disk_write_speed)}</p>
                <p className="text-xs text-gray-400">Escritura</p>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* GPU Info */}
      {stats.gpu_name && (
        <div className="mt-4 bg-dark-card border border-dark-border rounded-xl p-4">
          <div className="flex items-center gap-2 mb-4">
            <Monitor size={18} className="text-pink-400" />
            <h3 className="font-semibold">GPU</h3>
          </div>
          <div className="flex items-center justify-between">
            <div>
              <p className="text-xl font-bold">{stats.gpu_name}</p>
              {stats.gpu_vendor && (
                <p className="text-sm text-gray-400">{stats.gpu_vendor}</p>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default MonitorView;
