import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Globe,
  RefreshCw,
  ExternalLink,
  Square,
  Copy,
  Server,
  Database,
  Code,
  Terminal,
  Zap,
} from "lucide-react";
import { ErrorBanner } from "../components/ErrorBanner";

interface PortInfo {
  port: number;
  pid: number;
  process_name: string;
  service_type: string;
  protocol: string;
  local_address: string;
  working_dir?: string;
  command?: string;
  cpu_usage: number;
  memory_mb: number;
}

const serviceIcons: Record<string, React.ReactNode> = {
  http: <Globe size={18} />,
  https: <Globe size={18} />,
  database: <Database size={18} />,
  nodejs: <Code size={18} />,
  python: <Terminal size={18} />,
  default: <Server size={18} />,
};

const serviceColors: Record<string, string> = {
  http: "text-green-400 bg-green-500/20",
  https: "text-green-400 bg-green-500/20",
  database: "text-purple-400 bg-purple-500/20",
  nodejs: "text-yellow-400 bg-yellow-500/20",
  python: "text-blue-400 bg-blue-500/20",
  default: "text-gray-400 bg-gray-500/20",
};

function getServiceCategory(port: number, processName: string): string {
  const name = processName.toLowerCase();

  if (name.includes("node") || name.includes("npm") || name.includes("vite") || name.includes("next")) {
    return "nodejs";
  }
  if (name.includes("python") || name.includes("jupyter") || name.includes("flask") || name.includes("django")) {
    return "python";
  }
  if (name.includes("postgres") || name.includes("mysql") || name.includes("mongo") || name.includes("redis")) {
    return "database";
  }
  if ([80, 443, 3000, 3001, 4000, 5000, 5173, 8000, 8080, 8888].includes(port)) {
    return "http";
  }

  return "default";
}

function PortScannerView() {
  const [ports, setPorts] = useState<PortInfo[]>([]);
  const [isScanning, setIsScanning] = useState(false);
  const [selectedPort, setSelectedPort] = useState<PortInfo | null>(null);
  const [filter, setFilter] = useState<string>("all");
  const [scanError, setScanError] = useState<string | null>(null);
  const [killError, setKillError] = useState<string | null>(null);
  const [autoRefresh, setAutoRefresh] = useState(false);

  const scanPorts = async () => {
    setIsScanning(true);
    setScanError(null);
    try {
      const result: PortInfo[] = await invoke("scan_ports");
      setPorts(result);
    } catch (error) {
      console.error("Port scan error:", error);
      setScanError(error instanceof Error ? error.message : String(error));
    }
    setIsScanning(false);
  };

  useEffect(() => {
    let isMounted = true;
    const loadInitialData = async () => {
      if (!isMounted) return;
      setIsScanning(true);
      setScanError(null);
      try {
        const result: PortInfo[] = await invoke("scan_ports");
        if (isMounted) {
          setPorts(result);
        }
      } catch (error) {
        console.error("Port scan error:", error);
        if (isMounted) {
          setScanError(error instanceof Error ? error.message : String(error));
        }
      }
      if (isMounted) {
        setIsScanning(false);
      }
    };
    loadInitialData();
    return () => { isMounted = false; };
  }, []);

  // Auto-refresh
  useEffect(() => {
    if (!autoRefresh) return;
    let isMounted = true;
    const interval = setInterval(async () => {
      if (!isMounted) return;
      try {
        const result: PortInfo[] = await invoke("scan_ports");
        if (isMounted) {
          setPorts(result);
        }
      } catch (error) {
        console.error("Port scan error:", error);
      }
    }, 5000);
    return () => {
      isMounted = false;
      clearInterval(interval);
    };
  }, [autoRefresh]);

  const openInBrowser = (port: number) => {
    window.open(`http://localhost:${port}`, "_blank");
  };

  const copyUrl = (port: number) => {
    navigator.clipboard.writeText(`http://localhost:${port}`);
  };

  const stopProcess = async (pid: number) => {
    setKillError(null);
    try {
      await invoke("kill_process", { pid });
      setPorts((prev) => prev.filter((p) => p.pid !== pid));
      setSelectedPort(null);
    } catch (error) {
      console.error("Failed to stop process:", error);
      setKillError(error instanceof Error ? error.message : String(error));
    }
  };

  const filteredPorts = ports.filter((port) => {
    if (filter === "all") return true;
    return getServiceCategory(port.port, port.process_name) === filter;
  });

  return (
    <div className="p-6">
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <h2 className="text-2xl font-bold" id="port-scanner-title">Escaner de Puertos</h2>
          <p className="text-gray-400 mt-1">
            Detecta servicios web corriendo en tu Mac
          </p>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={() => setAutoRefresh(!autoRefresh)}
            className={`flex items-center gap-2 px-3 py-2 rounded-lg transition-colors ${
              autoRefresh
                ? "bg-green-500/20 text-green-400 border border-green-500/30"
                : "bg-dark-card border border-dark-border text-gray-400 hover:text-white"
            }`}
            aria-label={autoRefresh ? "Desactivar actualizacion automatica" : "Activar actualizacion automatica"}
            aria-pressed={autoRefresh}
          >
            <Zap size={16} aria-hidden="true" />
            <span className="text-sm">Auto</span>
          </button>
          <button
            onClick={scanPorts}
            disabled={isScanning}
            className="flex items-center gap-2 px-4 py-2 bg-primary-600 hover:bg-primary-700 text-white rounded-lg transition-colors disabled:opacity-50"
            aria-label="Escanear puertos activos"
          >
            <RefreshCw size={18} className={isScanning ? "animate-spin" : ""} aria-hidden="true" />
            <span>Escanear Puertos</span>
          </button>
        </div>
      </div>

      {/* Error banners */}
      {scanError && (
        <ErrorBanner
          error={scanError}
          onRetry={scanPorts}
          className="mb-6"
        />
      )}
      {killError && (
        <ErrorBanner
          error={killError}
          onRetry={() => setKillError(null)}
          className="mb-6"
        />
      )}

      {/* Filter tabs */}
      <div className="flex gap-2 mb-6" role="tablist" aria-label="Filtrar por tipo de servicio">
        {[
          { id: "all", label: "Todos" },
          { id: "http", label: "Web" },
          { id: "database", label: "Bases de Datos" },
          { id: "nodejs", label: "Node.js" },
          { id: "python", label: "Python" },
        ].map((tab) => (
          <button
            key={tab.id}
            onClick={() => setFilter(tab.id)}
            className={`px-4 py-2 rounded-lg transition-colors ${
              filter === tab.id
                ? "bg-primary-500/20 text-primary-400 border border-primary-500/30"
                : "bg-dark-card border border-dark-border text-gray-400 hover:text-white"
            }`}
            role="tab"
            aria-selected={filter === tab.id}
            aria-controls="ports-panel"
          >
            {tab.label}
          </button>
        ))}
      </div>

      {/* Summary */}
      <div className="bg-dark-card border border-dark-border rounded-xl p-4 mb-6">
        <div className="flex items-center gap-6">
          <div>
            <p className="text-gray-400 text-sm">Servicios activos</p>
            <p className="text-2xl font-bold text-primary-400">{ports.length}</p>
          </div>
          <div className="h-10 w-px bg-dark-border" />
          <div>
            <p className="text-gray-400 text-sm">Servidores Web</p>
            <p className="text-2xl font-bold">
              {
                ports.filter((p) =>
                  getServiceCategory(p.port, p.process_name) === "http" ||
                  getServiceCategory(p.port, p.process_name) === "nodejs" ||
                  getServiceCategory(p.port, p.process_name) === "python"
                ).length
              }
            </p>
          </div>
          <div className="h-10 w-px bg-dark-border" />
          <div>
            <p className="text-gray-400 text-sm">Bases de Datos</p>
            <p className="text-2xl font-bold">
              {
                ports.filter(
                  (p) => getServiceCategory(p.port, p.process_name) === "database"
                ).length
              }
            </p>
          </div>
        </div>
      </div>

      <div className="flex gap-6">
        {/* Ports list */}
        <div className="flex-1" id="ports-panel" role="tabpanel" aria-live="polite">
          <div className="bg-dark-card border border-dark-border rounded-xl overflow-hidden">
            <table className="w-full" role="table" aria-labelledby="port-scanner-title">
              <thead>
                <tr className="border-b border-dark-border">
                  <th scope="col" className="text-left px-4 py-3 text-sm font-medium text-gray-400">
                    Puerto
                  </th>
                  <th scope="col" className="text-left px-4 py-3 text-sm font-medium text-gray-400">
                    Servicio
                  </th>
                  <th scope="col" className="text-left px-4 py-3 text-sm font-medium text-gray-400">
                    Proceso
                  </th>
                  <th scope="col" className="text-left px-4 py-3 text-sm font-medium text-gray-400">
                    PID
                  </th>
                  <th scope="col" className="text-right px-4 py-3 text-sm font-medium text-gray-400">
                    CPU
                  </th>
                  <th scope="col" className="text-right px-4 py-3 text-sm font-medium text-gray-400">
                    RAM
                  </th>
                  <th scope="col" className="text-left px-4 py-3 text-sm font-medium text-gray-400">
                    Estado
                  </th>
                </tr>
              </thead>
              <tbody>
                {filteredPorts.map((port) => {
                  const category = getServiceCategory(port.port, port.process_name);
                  const colors = serviceColors[category] || serviceColors.default;
                  const icon = serviceIcons[category] || serviceIcons.default;

                  return (
                    <tr
                      key={`${port.port}-${port.pid}`}
                      onClick={() => setSelectedPort(port)}
                      className={`border-b border-dark-border/50 cursor-pointer transition-colors ${
                        selectedPort?.port === port.port
                          ? "bg-primary-500/10"
                          : "hover:bg-dark-border/30"
                      }`}
                    >
                      <td className="px-4 py-3">
                        <span className="font-mono text-primary-400 font-bold">
                          {port.port}
                        </span>
                      </td>
                      <td className="px-4 py-3">
                        <div className="flex items-center gap-2">
                          <div className={`p-1.5 rounded-lg ${colors}`}>
                            {icon}
                          </div>
                          <span>{port.service_type}</span>
                        </div>
                      </td>
                      <td className="px-4 py-3 text-gray-400">
                        {port.process_name}
                      </td>
                      <td className="px-4 py-3 text-gray-400 font-mono">
                        {port.pid}
                      </td>
                      <td className="px-4 py-3 text-right">
                        <span className={`font-mono text-sm ${port.cpu_usage > 50 ? 'text-red-400' : port.cpu_usage > 20 ? 'text-yellow-400' : 'text-gray-400'}`}>
                          {port.cpu_usage.toFixed(1)}%
                        </span>
                      </td>
                      <td className="px-4 py-3 text-right">
                        <span className="font-mono text-sm text-gray-400">
                          {port.memory_mb >= 1024
                            ? `${(port.memory_mb / 1024).toFixed(1)} GB`
                            : `${port.memory_mb.toFixed(0)} MB`}
                        </span>
                      </td>
                      <td className="px-4 py-3">
                        <div className="flex items-center gap-2">
                          <div className="status-dot status-active" />
                          <span className="text-green-400 text-sm">Activo</span>
                        </div>
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>

            {filteredPorts.length === 0 && (
              <div className="p-8 text-center text-gray-400">
                <Globe size={48} className="mx-auto mb-4 opacity-50" />
                <p>No se encontraron servicios</p>
              </div>
            )}
          </div>
        </div>

        {/* Details panel */}
        {selectedPort && (
          <div className="w-80">
            <div className="bg-dark-card border border-dark-border rounded-xl p-4 sticky top-6">
              <h3 className="font-semibold mb-4">Detalles del Puerto</h3>

              <div className="space-y-4">
                <div>
                  <p className="text-sm text-gray-400">Puerto</p>
                  <p className="font-mono text-2xl text-primary-400">
                    {selectedPort.port}
                  </p>
                </div>

                <div>
                  <p className="text-sm text-gray-400">Proceso</p>
                  <p className="font-medium">{selectedPort.process_name}</p>
                  <p className="text-sm text-gray-500">PID: {selectedPort.pid}</p>
                </div>

                <div className="grid grid-cols-2 gap-4">
                  <div className="bg-dark-bg p-3 rounded-lg">
                    <p className="text-xs text-gray-400">CPU</p>
                    <p className={`text-lg font-bold ${selectedPort.cpu_usage > 50 ? 'text-red-400' : selectedPort.cpu_usage > 20 ? 'text-yellow-400' : 'text-green-400'}`}>
                      {selectedPort.cpu_usage.toFixed(1)}%
                    </p>
                  </div>
                  <div className="bg-dark-bg p-3 rounded-lg">
                    <p className="text-xs text-gray-400">RAM</p>
                    <p className="text-lg font-bold text-blue-400">
                      {selectedPort.memory_mb >= 1024
                        ? `${(selectedPort.memory_mb / 1024).toFixed(1)} GB`
                        : `${selectedPort.memory_mb.toFixed(0)} MB`}
                    </p>
                  </div>
                </div>

                <div>
                  <p className="text-sm text-gray-400">Direccion</p>
                  <p className="font-mono text-sm">
                    {selectedPort.local_address}:{selectedPort.port}
                  </p>
                </div>

                {selectedPort.working_dir && (
                  <div>
                    <p className="text-sm text-gray-400">Directorio</p>
                    <p className="font-mono text-xs text-gray-300 break-all">
                      {selectedPort.working_dir}
                    </p>
                  </div>
                )}

                {selectedPort.command && (
                  <div>
                    <p className="text-sm text-gray-400">Comando</p>
                    <p className="font-mono text-xs text-gray-300 break-all bg-dark-bg p-2 rounded">
                      {selectedPort.command}
                    </p>
                  </div>
                )}

                <div className="pt-4 space-y-2" role="group" aria-label="Acciones del puerto">
                  <button
                    onClick={() => openInBrowser(selectedPort.port)}
                    className="w-full flex items-center justify-center gap-2 px-4 py-2 bg-primary-600 hover:bg-primary-700 text-white rounded-lg transition-colors"
                    aria-label={`Abrir puerto ${selectedPort.port} en navegador`}
                  >
                    <ExternalLink size={16} aria-hidden="true" />
                    <span>Abrir en Navegador</span>
                  </button>

                  <button
                    onClick={() => copyUrl(selectedPort.port)}
                    className="w-full flex items-center justify-center gap-2 px-4 py-2 bg-dark-border hover:bg-dark-border/80 text-white rounded-lg transition-colors"
                    aria-label={`Copiar URL del puerto ${selectedPort.port}`}
                  >
                    <Copy size={16} aria-hidden="true" />
                    <span>Copiar URL</span>
                  </button>

                  <button
                    onClick={() => stopProcess(selectedPort.pid)}
                    className="w-full flex items-center justify-center gap-2 px-4 py-2 bg-red-500/20 hover:bg-red-500/30 text-red-400 border border-red-500/30 rounded-lg transition-colors"
                    aria-label={`Detener proceso ${selectedPort.process_name}`}
                  >
                    <Square size={16} aria-hidden="true" />
                    <span>Detener Proceso</span>
                  </button>
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

export default PortScannerView;
