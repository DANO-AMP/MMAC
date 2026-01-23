import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Shield,
  ShieldCheck,
  ShieldOff,
  RefreshCw,
  Search,
  ChevronDown,
  ChevronRight,
  Globe,
  Activity,
  AlertCircle,
} from "lucide-react";
import { ErrorBanner } from "../components/ErrorBanner";

interface OutgoingConnection {
  process_name: string;
  pid: number;
  remote_host: string;
  remote_port: number;
  local_port: number;
  connection_state: string;
}

interface ProcessConnections {
  process_name: string;
  pid: number;
  connection_count: number;
  connections: OutgoingConnection[];
}

interface FirewallStatus {
  enabled: boolean;
  stealth_mode: boolean;
  block_all_incoming: boolean;
}

function FirewallView() {
  const [connections, setConnections] = useState<ProcessConnections[]>([]);
  const [firewallStatus, setFirewallStatus] = useState<FirewallStatus | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [expandedProcesses, setExpandedProcesses] = useState<Set<string>>(new Set());
  const [filter, setFilter] = useState<"all" | "established" | "listen">("all");

  const loadData = async (refresh = false) => {
    if (refresh) {
      setIsRefreshing(true);
    } else {
      setIsLoading(true);
    }
    setError(null);

    try {
      const [conns, status] = await Promise.all([
        invoke<ProcessConnections[]>("get_outgoing_connections"),
        invoke<FirewallStatus>("get_firewall_status"),
      ]);
      setConnections(conns);
      setFirewallStatus(status);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsLoading(false);
      setIsRefreshing(false);
    }
  };

  useEffect(() => {
    let isMounted = true;

    const loadInitialData = async () => {
      if (!isMounted) return;
      setIsLoading(true);
      setError(null);
      try {
        const [conns, status] = await Promise.all([
          invoke<ProcessConnections[]>("get_outgoing_connections"),
          invoke<FirewallStatus>("get_firewall_status"),
        ]);
        if (isMounted) {
          setConnections(conns);
          setFirewallStatus(status);
        }
      } catch (err) {
        if (isMounted) {
          setError(err instanceof Error ? err.message : String(err));
        }
      } finally {
        if (isMounted) {
          setIsLoading(false);
        }
      }
    };

    loadInitialData();

    // Auto-refresh every 5 seconds
    const interval = setInterval(async () => {
      if (!isMounted) return;
      try {
        const [conns, status] = await Promise.all([
          invoke<ProcessConnections[]>("get_outgoing_connections"),
          invoke<FirewallStatus>("get_firewall_status"),
        ]);
        if (isMounted) {
          setConnections(conns);
          setFirewallStatus(status);
        }
      } catch (err) {
        // Silent refresh errors
      }
    }, 5000);

    return () => {
      isMounted = false;
      clearInterval(interval);
    };
  }, []);

  const toggleExpand = (key: string) => {
    const newExpanded = new Set(expandedProcesses);
    if (newExpanded.has(key)) {
      newExpanded.delete(key);
    } else {
      newExpanded.add(key);
    }
    setExpandedProcesses(newExpanded);
  };

  const getStateColor = (state: string) => {
    switch (state) {
      case "ESTABLISHED":
        return "text-green-400";
      case "LISTEN":
        return "text-blue-400";
      case "CLOSE_WAIT":
        return "text-yellow-400";
      case "TIME_WAIT":
        return "text-gray-400";
      default:
        return "text-gray-500";
    }
  };

  const filteredConnections = connections.filter(proc => {
    const matchesSearch =
      proc.process_name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      proc.connections.some(c =>
        c.remote_host.toLowerCase().includes(searchQuery.toLowerCase())
      );

    const matchesFilter =
      filter === "all" ||
      proc.connections.some(c =>
        filter === "established"
          ? c.connection_state === "ESTABLISHED"
          : c.connection_state === "LISTEN"
      );

    return matchesSearch && matchesFilter;
  });

  const totalConnections = connections.reduce((acc, p) => acc + p.connection_count, 0);
  const establishedCount = connections.reduce(
    (acc, p) => acc + p.connections.filter(c => c.connection_state === "ESTABLISHED").length,
    0
  );
  const listeningCount = connections.reduce(
    (acc, p) => acc + p.connections.filter(c => c.connection_state === "LISTEN").length,
    0
  );

  if (isLoading) {
    return (
      <div className="p-6 flex items-center justify-center h-full">
        <div className="text-center">
          <RefreshCw size={32} className="animate-spin mx-auto text-primary-400 mb-4" />
          <p className="text-gray-400">Analizando conexiones de red...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="p-6">
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <h2 className="text-2xl font-bold">Monitor de Red</h2>
          <p className="text-gray-400 mt-1">
            Conexiones salientes y estado del firewall
          </p>
        </div>
        <button
          onClick={() => loadData(true)}
          disabled={isRefreshing}
          className="flex items-center gap-2 px-4 py-2 bg-dark-card border border-dark-border rounded-lg hover:bg-dark-border transition-colors disabled:opacity-50"
          aria-label="Actualizar conexiones de red"
        >
          <RefreshCw size={18} className={isRefreshing ? "animate-spin" : ""} aria-hidden="true" />
          <span>Actualizar</span>
        </button>
      </div>

      {/* Error banner */}
      {error && (
        <ErrorBanner error={error} onRetry={() => loadData()} className="mb-6" />
      )}

      {/* Firewall status card */}
      <div className={`bg-gradient-to-r ${
        firewallStatus?.enabled
          ? "from-green-600/20 to-green-800/20 border-green-500/30"
          : "from-red-600/20 to-red-800/20 border-red-500/30"
      } border rounded-xl p-6 mb-6`} role="region" aria-label="Estado del firewall" aria-live="polite">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4">
            <div className={`p-3 rounded-xl ${
              firewallStatus?.enabled ? "bg-green-500/20" : "bg-red-500/20"
            }`}>
              {firewallStatus?.enabled ? (
                <ShieldCheck size={32} className="text-green-400" aria-hidden="true" />
              ) : (
                <ShieldOff size={32} className="text-red-400" aria-hidden="true" />
              )}
            </div>
            <div>
              <h3 className="text-xl font-bold">
                Firewall {firewallStatus?.enabled ? "Activado" : "Desactivado"}
              </h3>
              <div className="flex gap-4 mt-1 text-sm">
                {firewallStatus?.stealth_mode && (
                  <span className="text-green-400">Modo Oculto</span>
                )}
                {firewallStatus?.block_all_incoming && (
                  <span className="text-yellow-400">Bloqueo Total</span>
                )}
              </div>
            </div>
          </div>
          <div className="text-right">
            <p className="text-3xl font-bold">{connections.length}</p>
            <p className="text-sm text-gray-400">Procesos con red</p>
          </div>
        </div>
      </div>

      {/* Stats cards */}
      <div className="grid grid-cols-4 gap-4 mb-6">
        <div className="bg-dark-card border border-dark-border rounded-xl p-4">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-primary-500/20 rounded-lg">
              <Activity size={20} className="text-primary-400" />
            </div>
            <div>
              <p className="text-2xl font-bold">{totalConnections}</p>
              <p className="text-sm text-gray-400">Total</p>
            </div>
          </div>
        </div>
        <div className="bg-dark-card border border-dark-border rounded-xl p-4">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-green-500/20 rounded-lg">
              <Globe size={20} className="text-green-400" />
            </div>
            <div>
              <p className="text-2xl font-bold">{establishedCount}</p>
              <p className="text-sm text-gray-400">Establecidas</p>
            </div>
          </div>
        </div>
        <div className="bg-dark-card border border-dark-border rounded-xl p-4">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-blue-500/20 rounded-lg">
              <Shield size={20} className="text-blue-400" />
            </div>
            <div>
              <p className="text-2xl font-bold">{listeningCount}</p>
              <p className="text-sm text-gray-400">Escuchando</p>
            </div>
          </div>
        </div>
        <div className="bg-dark-card border border-dark-border rounded-xl p-4">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-purple-500/20 rounded-lg">
              <Activity size={20} className="text-purple-400" />
            </div>
            <div>
              <p className="text-2xl font-bold">{connections.length}</p>
              <p className="text-sm text-gray-400">Procesos</p>
            </div>
          </div>
        </div>
      </div>

      {/* Search and filter */}
      <div className="flex gap-4 mb-4">
        <div className="flex-1 relative">
          <Search size={18} className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-500" aria-hidden="true" />
          <input
            type="text"
            placeholder="Buscar por proceso o host..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full pl-10 pr-4 py-2 bg-dark-card border border-dark-border rounded-lg focus:outline-none focus:border-primary-500"
            aria-label="Buscar por proceso o host remoto"
          />
        </div>
        <div className="flex bg-dark-card border border-dark-border rounded-lg overflow-hidden" role="group" aria-label="Filtrar conexiones por estado">
          {[
            { id: "all", label: "Todas" },
            { id: "established", label: "Establecidas" },
            { id: "listen", label: "Escuchando" },
          ].map((f) => (
            <button
              key={f.id}
              onClick={() => setFilter(f.id as typeof filter)}
              className={`px-4 py-2 text-sm transition-colors ${
                filter === f.id
                  ? "bg-primary-500/20 text-primary-400"
                  : "text-gray-400 hover:text-white hover:bg-dark-border"
              }`}
              aria-pressed={filter === f.id}
            >
              {f.label}
            </button>
          ))}
        </div>
      </div>

      {/* Connections list */}
      <div className="bg-dark-card border border-dark-border rounded-xl overflow-hidden">
        <div className="divide-y divide-dark-border max-h-96 overflow-y-auto">
          {filteredConnections.length === 0 ? (
            <div className="p-8 text-center text-gray-500">
              No se encontraron conexiones
            </div>
          ) : (
            filteredConnections.map((proc) => {
              const key = `${proc.process_name}-${proc.pid}`;
              const isExpanded = expandedProcesses.has(key);

              return (
                <div key={key}>
                  {/* Process header */}
                  <div
                    className="flex items-center gap-4 px-4 py-3 cursor-pointer hover:bg-dark-border/30 transition-colors"
                    onClick={() => toggleExpand(key)}
                    onKeyDown={(e) => e.key === "Enter" && toggleExpand(key)}
                    tabIndex={0}
                    role="button"
                    aria-expanded={isExpanded}
                    aria-label={`${proc.process_name}, ${proc.connection_count} conexiones. ${isExpanded ? "Contraer" : "Expandir"}`}
                  >
                    <div className="text-gray-400" aria-hidden="true">
                      {isExpanded ? <ChevronDown size={18} /> : <ChevronRight size={18} />}
                    </div>
                    <div className="flex-1">
                      <div className="flex items-center gap-2">
                        <span className="font-medium">{proc.process_name}</span>
                        <span className="text-xs text-gray-500">PID: {proc.pid}</span>
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      <span className="text-sm text-gray-400">
                        {proc.connection_count} {proc.connection_count === 1 ? "conexion" : "conexiones"}
                      </span>
                    </div>
                  </div>

                  {/* Expanded connections */}
                  {isExpanded && (
                    <div className="bg-dark-bg/50 border-t border-dark-border">
                      <div className="grid grid-cols-12 gap-4 px-4 py-2 text-xs font-medium text-gray-500 bg-dark-border/30">
                        <div className="col-span-4">Host Remoto</div>
                        <div className="col-span-2">Puerto Remoto</div>
                        <div className="col-span-2">Puerto Local</div>
                        <div className="col-span-4">Estado</div>
                      </div>
                      {proc.connections.map((conn, idx) => (
                        <div
                          key={idx}
                          className="grid grid-cols-12 gap-4 px-4 py-2 text-sm items-center hover:bg-dark-border/20"
                        >
                          <div className="col-span-4 font-mono text-xs truncate" title={conn.remote_host}>
                            {conn.remote_host === "*" ? (
                              <span className="text-gray-500">Cualquiera</span>
                            ) : (
                              conn.remote_host
                            )}
                          </div>
                          <div className="col-span-2 font-mono">
                            {conn.remote_port || "-"}
                          </div>
                          <div className="col-span-2 font-mono">
                            {conn.local_port}
                          </div>
                          <div className={`col-span-4 ${getStateColor(conn.connection_state)}`}>
                            {conn.connection_state}
                          </div>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              );
            })
          )}
        </div>
      </div>

      {/* Info banner */}
      <div className="mt-4 bg-dark-card border border-dark-border rounded-xl p-4 flex items-start gap-3">
        <AlertCircle size={20} className="text-primary-400 flex-shrink-0 mt-0.5" />
        <div>
          <p className="text-sm text-gray-300">
            Este monitor muestra las conexiones de red activas. Los datos se actualizan cada 5 segundos.
            Para controlar el firewall de macOS, usa Preferencias del Sistema.
          </p>
        </div>
      </div>
    </div>
  );
}

export default FirewallView;
