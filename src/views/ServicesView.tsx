import { useState, useEffect, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Settings2,
  RefreshCw,
  Play,
  Square,
  Search,
  CheckCircle2,
  XCircle,
  AlertCircle,
  Info,
  User,
  Server,
} from "lucide-react";
import { ErrorBanner } from "../components/ErrorBanner";

interface LaunchService {
  label: string;
  pid: number | null;
  status: string;
  kind: string;
  last_exit_status: number | null;
}

interface ServicesResult {
  user_agents: LaunchService[];
  user_daemons: LaunchService[];
  system_agents: LaunchService[];
}

function ServicesView() {
  const [services, setServices] = useState<ServicesResult | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [filter, setFilter] = useState<"all" | "running" | "stopped" | "error">("all");
  const [selectedService, setSelectedService] = useState<string | null>(null);
  const [serviceInfo, setServiceInfo] = useState<string | null>(null);
  const [operatingOn, setOperatingOn] = useState<string | null>(null);
  const [operationStatus, setOperationStatus] = useState<{ type: "success" | "error"; message: string } | null>(null);

  const loadData = async (refresh = false) => {
    if (refresh) {
      setIsRefreshing(true);
    } else {
      setIsLoading(true);
    }
    setError(null);

    try {
      const result: ServicesResult = await invoke("list_launch_services");
      setServices(result);
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
        const result: ServicesResult = await invoke("list_launch_services");
        if (isMounted) {
          setServices(result);
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
    return () => { isMounted = false; };
  }, []);

  const handleStart = async (label: string) => {
    setOperatingOn(label);
    setOperationStatus(null);
    try {
      const result = await invoke<string>("start_launch_service", { label });
      setOperationStatus({ type: "success", message: result });
      loadData(true);
    } catch (err) {
      setOperationStatus({ type: "error", message: err instanceof Error ? err.message : String(err) });
    } finally {
      setOperatingOn(null);
    }
  };

  const handleStop = async (label: string) => {
    setOperatingOn(label);
    setOperationStatus(null);
    try {
      const result = await invoke<string>("stop_launch_service", { label });
      setOperationStatus({ type: "success", message: result });
      loadData(true);
    } catch (err) {
      setOperationStatus({ type: "error", message: err instanceof Error ? err.message : String(err) });
    } finally {
      setOperatingOn(null);
    }
  };

  const handleShowInfo = async (label: string) => {
    setSelectedService(label);
    try {
      const info = await invoke<string>("get_launch_service_info", { label });
      setServiceInfo(info);
    } catch {
      setServiceInfo("No se pudo obtener informacion del servicio");
    }
  };

  // Focus trap and keyboard handling for modal
  useEffect(() => {
    if (!serviceInfo) return;

    const dialog = document.querySelector('[role="dialog"]') as HTMLElement;
    if (!dialog) return;

    const focusableElements = dialog.querySelectorAll('button, [tabindex]:not([tabindex="-1"])');
    const firstEl = focusableElements[0] as HTMLElement;
    const lastEl = focusableElements[focusableElements.length - 1] as HTMLElement;

    // Focus first element when modal opens
    firstEl?.focus();

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        setServiceInfo(null);
        setSelectedService(null);
        return;
      }
      if (e.key === 'Tab') {
        if (e.shiftKey && document.activeElement === firstEl) {
          e.preventDefault();
          lastEl?.focus();
        } else if (!e.shiftKey && document.activeElement === lastEl) {
          e.preventDefault();
          firstEl?.focus();
        }
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [serviceInfo]);

  const getStatusIcon = (status: string) => {
    switch (status) {
      case "running":
        return <CheckCircle2 size={16} className="text-green-400" />;
      case "stopped":
        return <XCircle size={16} className="text-gray-400" />;
      case "error":
        return <AlertCircle size={16} className="text-red-400" />;
      default:
        return <AlertCircle size={16} className="text-yellow-400" />;
    }
  };

  const getStatusLabel = (status: string) => {
    switch (status) {
      case "running":
        return "Ejecutando";
      case "stopped":
        return "Detenido";
      case "error":
        return "Error";
      default:
        return "Desconocido";
    }
  };

  const allServices = useMemo(() => {
    return services
      ? [...services.user_agents, ...services.user_daemons, ...services.system_agents]
      : [];
  }, [services]);

  const filteredServices = useMemo(() => {
    return allServices.filter(service => {
      const matchesSearch = service.label.toLowerCase().includes(searchQuery.toLowerCase());
      const matchesFilter =
        filter === "all" ||
        service.status === filter;
      return matchesSearch && matchesFilter;
    });
  }, [allServices, searchQuery, filter]);

  const runningCount = allServices.filter(s => s.status === "running").length;
  const stoppedCount = allServices.filter(s => s.status === "stopped").length;
  const errorCount = allServices.filter(s => s.status === "error").length;

  if (isLoading) {
    return (
      <div className="p-6 flex items-center justify-center h-full">
        <div className="text-center">
          <RefreshCw size={32} className="animate-spin mx-auto text-primary-400 mb-4" />
          <p className="text-gray-400">Cargando servicios...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="p-6">
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <h2 className="text-2xl font-bold">Servicios</h2>
          <p className="text-gray-400 mt-1">
            Gestiona servicios y daemons de launchctl
          </p>
        </div>
        <button
          onClick={() => loadData(true)}
          disabled={isRefreshing}
          className="flex items-center gap-2 px-4 py-2 bg-dark-card border border-dark-border rounded-lg hover:bg-dark-border transition-colors disabled:opacity-50"
          aria-label="Actualizar lista de servicios"
        >
          <RefreshCw size={18} className={isRefreshing ? "animate-spin" : ""} aria-hidden="true" />
          <span>Actualizar</span>
        </button>
      </div>

      {/* Error banner */}
      {error && (
        <ErrorBanner error={error} onRetry={() => loadData()} className="mb-6" />
      )}

      {/* Operation status */}
      {operationStatus && (
        <div className={`mb-6 p-4 rounded-lg flex items-center gap-3 ${
          operationStatus.type === "success"
            ? "bg-green-500/10 border border-green-500/30"
            : "bg-red-500/10 border border-red-500/30"
        }`} role="status" aria-live="polite">
          {operationStatus.type === "success" ? (
            <CheckCircle2 size={20} className="text-green-400" aria-hidden="true" />
          ) : (
            <AlertCircle size={20} className="text-red-400" aria-hidden="true" />
          )}
          <span className={operationStatus.type === "success" ? "text-green-400" : "text-red-400"}>
            {operationStatus.message}
          </span>
        </div>
      )}

      {/* Stats cards */}
      <div className="grid grid-cols-4 gap-4 mb-6">
        <div className="bg-dark-card border border-dark-border rounded-xl p-4">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-primary-500/20 rounded-lg">
              <Settings2 size={20} className="text-primary-400" />
            </div>
            <div>
              <p className="text-2xl font-bold">{allServices.length}</p>
              <p className="text-sm text-gray-400">Total</p>
            </div>
          </div>
        </div>
        <div className="bg-dark-card border border-dark-border rounded-xl p-4">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-green-500/20 rounded-lg">
              <CheckCircle2 size={20} className="text-green-400" />
            </div>
            <div>
              <p className="text-2xl font-bold">{runningCount}</p>
              <p className="text-sm text-gray-400">Ejecutando</p>
            </div>
          </div>
        </div>
        <div className="bg-dark-card border border-dark-border rounded-xl p-4">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-gray-500/20 rounded-lg">
              <XCircle size={20} className="text-gray-400" />
            </div>
            <div>
              <p className="text-2xl font-bold">{stoppedCount}</p>
              <p className="text-sm text-gray-400">Detenidos</p>
            </div>
          </div>
        </div>
        <div className="bg-dark-card border border-dark-border rounded-xl p-4">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-red-500/20 rounded-lg">
              <AlertCircle size={20} className="text-red-400" />
            </div>
            <div>
              <p className="text-2xl font-bold">{errorCount}</p>
              <p className="text-sm text-gray-400">Con Error</p>
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
            placeholder="Buscar servicios..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full pl-10 pr-4 py-2 bg-dark-card border border-dark-border rounded-lg focus:outline-none focus:border-primary-500"
            aria-label="Buscar servicios por nombre"
          />
        </div>
        <div className="flex bg-dark-card border border-dark-border rounded-lg overflow-hidden" role="group" aria-label="Filtrar servicios por estado">
          {[
            { id: "all", label: "Todos" },
            { id: "running", label: "Ejecutando" },
            { id: "stopped", label: "Detenidos" },
            { id: "error", label: "Errores" },
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

      {/* Services list */}
      <div className="bg-dark-card border border-dark-border rounded-xl overflow-hidden">
        <div className="grid grid-cols-12 gap-4 px-4 py-3 bg-dark-border/50 text-sm font-medium text-gray-400">
          <div className="col-span-5">Servicio</div>
          <div className="col-span-2">Tipo</div>
          <div className="col-span-2">Estado</div>
          <div className="col-span-1">PID</div>
          <div className="col-span-2 text-right">Acciones</div>
        </div>
        <div className="divide-y divide-dark-border max-h-96 overflow-y-auto">
          {filteredServices.length === 0 ? (
            <div className="p-8 text-center text-gray-500">
              No se encontraron servicios
            </div>
          ) : (
            filteredServices.map((service) => (
              <div
                key={service.label}
                className={`grid grid-cols-12 gap-4 px-4 py-3 items-center hover:bg-dark-border/30 transition-colors ${
                  selectedService === service.label ? "bg-primary-500/5" : ""
                }`}
              >
                <div className="col-span-5">
                  <div className="flex items-center gap-2">
                    {service.kind.includes("User") ? (
                      <User size={16} className="text-blue-400" />
                    ) : (
                      <Server size={16} className="text-purple-400" />
                    )}
                    <span className="font-medium truncate" title={service.label}>
                      {service.label}
                    </span>
                  </div>
                </div>
                <div className="col-span-2">
                  <span className="text-sm text-gray-400">{service.kind}</span>
                </div>
                <div className="col-span-2">
                  <div className="flex items-center gap-2">
                    {getStatusIcon(service.status)}
                    <span className="text-sm">{getStatusLabel(service.status)}</span>
                  </div>
                </div>
                <div className="col-span-1 text-sm text-gray-400">
                  {service.pid || "-"}
                </div>
                <div className="col-span-2 flex justify-end gap-1">
                  {service.status === "running" ? (
                    <button
                      onClick={() => handleStop(service.label)}
                      disabled={operatingOn !== null}
                      className="p-2 text-red-400 hover:bg-red-500/20 rounded-lg transition-colors disabled:opacity-50"
                      aria-label={`Detener servicio ${service.label}`}
                    >
                      {operatingOn === service.label ? (
                        <RefreshCw size={16} className="animate-spin" aria-hidden="true" />
                      ) : (
                        <Square size={16} aria-hidden="true" />
                      )}
                    </button>
                  ) : (
                    <button
                      onClick={() => handleStart(service.label)}
                      disabled={operatingOn !== null}
                      className="p-2 text-green-400 hover:bg-green-500/20 rounded-lg transition-colors disabled:opacity-50"
                      aria-label={`Iniciar servicio ${service.label}`}
                    >
                      {operatingOn === service.label ? (
                        <RefreshCw size={16} className="animate-spin" aria-hidden="true" />
                      ) : (
                        <Play size={16} aria-hidden="true" />
                      )}
                    </button>
                  )}
                  <button
                    onClick={() => handleShowInfo(service.label)}
                    className="p-2 text-gray-400 hover:bg-dark-border rounded-lg transition-colors"
                    aria-label={`Ver informacion de ${service.label}`}
                  >
                    <Info size={16} aria-hidden="true" />
                  </button>
                </div>
              </div>
            ))
          )}
        </div>
      </div>

      {/* Service info modal */}
      {selectedService && serviceInfo && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50" role="dialog" aria-modal="true" aria-labelledby="service-modal-title">
          <div className="bg-dark-card border border-dark-border rounded-xl p-6 max-w-2xl w-full mx-4 max-h-[80vh] overflow-auto">
            <div className="flex items-center justify-between mb-4">
              <h3 className="font-semibold" id="service-modal-title">{selectedService}</h3>
              <button
                onClick={() => {
                  setSelectedService(null);
                  setServiceInfo(null);
                }}
                className="text-gray-400 hover:text-white"
                aria-label="Cerrar dialogo"
              >
                &times;
              </button>
            </div>
            <pre className="bg-dark-bg rounded-lg p-4 text-sm text-gray-300 overflow-auto whitespace-pre-wrap">
              {serviceInfo}
            </pre>
          </div>
        </div>
      )}
    </div>
  );
}

export default ServicesView;
