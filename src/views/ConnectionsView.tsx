import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Network,
  RefreshCw,
  Globe,
  Server,
  Trash2,
  FileText,
} from "lucide-react";
import { useConnections } from "../store/AppStore";

interface HostEntry {
  ip: string;
  hostname: string;
  comment: string | null;
}

type Tab = "connections" | "hosts" | "dns";

export default function ConnectionsView() {
  const { connections, isLoading: connectionsLoading, lastUpdated, refresh } = useConnections();
  const [activeTab, setActiveTab] = useState<Tab>("connections");
  const [hosts, setHosts] = useState<HostEntry[]>([]);
  const [hostsLoading, setHostsLoading] = useState(false);
  const [dnsLoading, setDnsLoading] = useState(false);
  const [dnsMessage, setDnsMessage] = useState<string | null>(null);

  const loadHosts = async () => {
    setHostsLoading(true);
    try {
      const result = await invoke<HostEntry[]>("get_hosts");
      setHosts(result);
    } catch (error) {
      console.error("Error loading hosts:", error);
    } finally {
      setHostsLoading(false);
    }
  };

  const flushDns = async () => {
    setDnsLoading(true);
    try {
      const result = await invoke<string>("flush_dns");
      setDnsMessage(result);
      setTimeout(() => setDnsMessage(null), 3000);
    } catch (error) {
      setDnsMessage("Error al vaciar caché DNS");
    } finally {
      setDnsLoading(false);
    }
  };

  useEffect(() => {
    let isMounted = true;

    const loadHostsData = async () => {
      if (!isMounted) return;
      if (activeTab === "hosts" && hosts.length === 0) {
        setHostsLoading(true);
        try {
          const result = await invoke<HostEntry[]>("get_hosts");
          if (isMounted) {
            setHosts(result);
          }
        } catch (error) {
          console.error("Error loading hosts:", error);
        } finally {
          if (isMounted) {
            setHostsLoading(false);
          }
        }
      }
    };

    loadHostsData();
    return () => { isMounted = false; };
  }, [activeTab, hosts.length]);

  const getStateColor = (state: string) => {
    switch (state) {
      case "ESTABLISHED":
        return "text-green-400";
      case "LISTEN":
        return "text-blue-400";
      case "TIME_WAIT":
        return "text-yellow-400";
      case "CLOSE_WAIT":
        return "text-orange-400";
      default:
        return "text-gray-400";
    }
  };

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold flex items-center gap-3">
            <Network className="text-primary-400" />
            Red
          </h2>
          <p className="text-gray-400 mt-1">
            Conexiones activas, hosts y DNS
            {lastUpdated > 0 && activeTab === "connections" && (
              <span className="ml-2 text-xs">
                (actualizado: {new Date(lastUpdated).toLocaleTimeString("es-ES")})
              </span>
            )}
          </p>
        </div>
      </div>

      {/* Tabs */}
      <div className="flex gap-2 border-b border-dark-border pb-4" role="tablist" aria-label="Secciones de red">
        <button
          onClick={() => setActiveTab("connections")}
          className={`flex items-center gap-2 px-4 py-2 rounded-lg transition-colors ${
            activeTab === "connections"
              ? "bg-primary-500/20 text-primary-400"
              : "hover:bg-dark-card"
          }`}
          role="tab"
          aria-selected={activeTab === "connections"}
          aria-controls="connections-panel"
          id="connections-tab"
        >
          <Globe size={18} aria-hidden="true" />
          Conexiones
          {connectionsLoading && <div className="w-2 h-2 rounded-full bg-blue-500 animate-pulse" aria-hidden="true" />}
        </button>
        <button
          onClick={() => setActiveTab("hosts")}
          className={`flex items-center gap-2 px-4 py-2 rounded-lg transition-colors ${
            activeTab === "hosts"
              ? "bg-primary-500/20 text-primary-400"
              : "hover:bg-dark-card"
          }`}
          role="tab"
          aria-selected={activeTab === "hosts"}
          aria-controls="hosts-panel"
          id="hosts-tab"
        >
          <Server size={18} aria-hidden="true" />
          Hosts
        </button>
        <button
          onClick={() => setActiveTab("dns")}
          className={`flex items-center gap-2 px-4 py-2 rounded-lg transition-colors ${
            activeTab === "dns"
              ? "bg-primary-500/20 text-primary-400"
              : "hover:bg-dark-card"
          }`}
          role="tab"
          aria-selected={activeTab === "dns"}
          aria-controls="dns-panel"
          id="dns-tab"
        >
          <FileText size={18} aria-hidden="true" />
          DNS
        </button>
      </div>

      {/* Connections Tab */}
      {activeTab === "connections" && (
        <div className="space-y-4" role="tabpanel" id="connections-panel" aria-labelledby="connections-tab">
          <div className="flex justify-between items-center">
            <div className="text-sm text-gray-400" aria-live="polite">
              {connections.length} conexiones activas
            </div>
            <button
              onClick={refresh}
              disabled={connectionsLoading}
              className="flex items-center gap-2 px-4 py-2 bg-dark-card border border-dark-border rounded-lg hover:bg-dark-border transition-colors"
              aria-label="Actualizar conexiones"
            >
              <RefreshCw size={18} className={connectionsLoading ? "animate-spin" : ""} aria-hidden="true" />
              Actualizar
            </button>
          </div>

          <div className="bg-dark-card rounded-xl border border-dark-border overflow-hidden">
            <table className="w-full text-sm" role="table" aria-label="Conexiones de red activas">
              <thead>
                <tr className="border-b border-dark-border bg-dark-bg/50">
                  <th scope="col" className="text-left px-4 py-3 font-medium">Proceso</th>
                  <th scope="col" className="text-left px-4 py-3 font-medium">Protocolo</th>
                  <th scope="col" className="text-left px-4 py-3 font-medium">Local</th>
                  <th scope="col" className="text-left px-4 py-3 font-medium">Remoto</th>
                  <th scope="col" className="text-left px-4 py-3 font-medium">Estado</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-dark-border">
                {connections.slice(0, 100).map((conn, idx) => (
                  <tr key={`${conn.pid}-${conn.local_address}-${conn.local_port}-${conn.remote_address}-${conn.remote_port}-${idx}`} className="hover:bg-dark-bg/30">
                    <td className="px-4 py-2">
                      <p className="font-medium">
                        {conn.process_name || `PID ${conn.pid}`}
                      </p>
                    </td>
                    <td className="px-4 py-2 font-mono text-gray-400">
                      TCP
                    </td>
                    <td className="px-4 py-2 font-mono">
                      {conn.local_address}:{conn.local_port}
                    </td>
                    <td className="px-4 py-2 font-mono">
                      {conn.remote_address}:{conn.remote_port}
                    </td>
                    <td className={`px-4 py-2 ${getStateColor(conn.state)}`}>
                      {conn.state || "-"}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
            {connections.length === 0 && (
              <div className="p-8 text-center text-gray-400">
                No se encontraron conexiones activas
              </div>
            )}
          </div>
        </div>
      )}

      {/* Hosts Tab */}
      {activeTab === "hosts" && (
        <div className="space-y-4" role="tabpanel" id="hosts-panel" aria-labelledby="hosts-tab">
          <div className="flex justify-end">
            <button
              onClick={loadHosts}
              disabled={hostsLoading}
              className="flex items-center gap-2 px-4 py-2 bg-dark-card border border-dark-border rounded-lg hover:bg-dark-border transition-colors"
              aria-label="Actualizar lista de hosts"
            >
              <RefreshCw size={18} className={hostsLoading ? "animate-spin" : ""} aria-hidden="true" />
              Actualizar
            </button>
          </div>

          <div className="bg-dark-card rounded-xl border border-dark-border overflow-hidden">
            <table className="w-full" role="table" aria-label="Entradas de hosts">
              <thead>
                <tr className="border-b border-dark-border bg-dark-bg/50">
                  <th scope="col" className="text-left px-4 py-3 font-medium">IP</th>
                  <th scope="col" className="text-left px-4 py-3 font-medium">Hostname</th>
                  <th scope="col" className="text-left px-4 py-3 font-medium">Comentario</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-dark-border">
                {hosts.map((host) => (
                  <tr key={`${host.ip}-${host.hostname}`} className="hover:bg-dark-bg/30">
                    <td className="px-4 py-3 font-mono">{host.ip}</td>
                    <td className="px-4 py-3">{host.hostname}</td>
                    <td className="px-4 py-3 text-gray-400">
                      {host.comment || "-"}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
            {hosts.length === 0 && (
              <div className="p-8 text-center text-gray-400">
                No hay entradas en /etc/hosts
              </div>
            )}
          </div>
        </div>
      )}

      {/* DNS Tab */}
      {activeTab === "dns" && (
        <div className="space-y-4" role="tabpanel" id="dns-panel" aria-labelledby="dns-tab">
          <div className="bg-dark-card rounded-xl border border-dark-border p-6">
            <h3 className="font-semibold mb-2">Caché DNS</h3>
            <p className="text-gray-400 mb-4">
              Vaciar la caché DNS puede resolver problemas de conexión a sitios
              web que han cambiado de servidor o que no cargan correctamente.
            </p>
            <button
              onClick={flushDns}
              disabled={dnsLoading}
              className="flex items-center gap-2 px-6 py-2.5 bg-primary-500 hover:bg-primary-600 rounded-lg font-medium transition-colors disabled:opacity-50"
              aria-label="Vaciar cache DNS del sistema"
            >
              {dnsLoading ? (
                <RefreshCw className="animate-spin" size={18} aria-hidden="true" />
              ) : (
                <Trash2 size={18} aria-hidden="true" />
              )}
              Vaciar Caché DNS
            </button>

            {dnsMessage && (
              <div className="mt-4 p-3 bg-green-500/10 border border-green-500/20 rounded-lg text-green-400" role="status" aria-live="polite">
                {dnsMessage}
              </div>
            )}
          </div>

          <div className="bg-dark-card rounded-xl border border-dark-border p-6">
            <h3 className="font-semibold mb-2">Servidores DNS Actuales</h3>
            <p className="text-gray-400 text-sm mb-4">
              Para cambiar los servidores DNS, ve a Preferencias del Sistema → Red
            </p>
            <div className="space-y-2 font-mono text-sm">
              <p>• Configurado por DHCP o manualmente en Red</p>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
