import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Package,
  RefreshCw,
  ArrowUpCircle,
  Trash2,
  AlertCircle,
  CheckCircle2,
  Search,
  Beer,
  Box,
  Sparkles,
} from "lucide-react";
import { ErrorBanner } from "../components/ErrorBanner";

interface BrewPackage {
  name: string;
  version: string;
  is_outdated: boolean;
  newer_version: string | null;
  is_cask: boolean;
}

interface HomebrewInfo {
  installed: boolean;
  version: string | null;
  formulae_count: number;
  casks_count: number;
}

function HomebrewView() {
  const [homebrewInfo, setHomebrewInfo] = useState<HomebrewInfo | null>(null);
  const [packages, setPackages] = useState<BrewPackage[]>([]);
  const [outdatedPackages, setOutdatedPackages] = useState<BrewPackage[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [filter, setFilter] = useState<"all" | "formulae" | "casks" | "outdated">("all");
  const [operationStatus, setOperationStatus] = useState<{ type: "success" | "error"; message: string } | null>(null);
  const [operatingOn, setOperatingOn] = useState<string | null>(null);

  const loadData = async (refresh = false) => {
    if (refresh) {
      setIsRefreshing(true);
    } else {
      setIsLoading(true);
    }
    setError(null);

    try {
      const info: HomebrewInfo = await invoke("get_homebrew_info");
      setHomebrewInfo(info);

      if (info.installed) {
        const [pkgs, outdated] = await Promise.all([
          invoke<BrewPackage[]>("list_brew_packages"),
          invoke<BrewPackage[]>("get_outdated_packages"),
        ]);
        setPackages(pkgs);
        setOutdatedPackages(outdated);

        // Mark packages as outdated in main list
        const outdatedNames = new Set(outdated.map(p => p.name));
        setPackages(pkgs.map(p => ({
          ...p,
          is_outdated: outdatedNames.has(p.name),
          newer_version: outdated.find(o => o.name === p.name)?.newer_version || null,
        })));
      }
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

  const handleUpgrade = async (pkg: BrewPackage) => {
    setOperatingOn(pkg.name);
    setOperationStatus(null);
    try {
      await invoke("upgrade_brew_package", { name: pkg.name, isCask: pkg.is_cask });
      setOperationStatus({ type: "success", message: `Successfully upgraded ${pkg.name}` });
      loadData(true);
    } catch (err) {
      setOperationStatus({ type: "error", message: err instanceof Error ? err.message : String(err) });
    } finally {
      setOperatingOn(null);
    }
  };

  const handleUpgradeAll = async () => {
    setOperatingOn("all");
    setOperationStatus(null);
    try {
      await invoke("upgrade_all_packages");
      setOperationStatus({ type: "success", message: "Successfully upgraded all packages" });
      loadData(true);
    } catch (err) {
      setOperationStatus({ type: "error", message: err instanceof Error ? err.message : String(err) });
    } finally {
      setOperatingOn(null);
    }
  };

  const handleUninstall = async (pkg: BrewPackage) => {
    if (!confirm(`Are you sure you want to uninstall ${pkg.name}?`)) return;

    setOperatingOn(pkg.name);
    setOperationStatus(null);
    try {
      await invoke("uninstall_brew_package", { name: pkg.name, isCask: pkg.is_cask });
      setOperationStatus({ type: "success", message: `Successfully uninstalled ${pkg.name}` });
      loadData(true);
    } catch (err) {
      setOperationStatus({ type: "error", message: err instanceof Error ? err.message : String(err) });
    } finally {
      setOperatingOn(null);
    }
  };

  const handleCleanup = async () => {
    setOperatingOn("cleanup");
    setOperationStatus(null);
    try {
      const result = await invoke<string>("brew_cleanup");
      setOperationStatus({ type: "success", message: result });
      loadData(true);
    } catch (err) {
      setOperationStatus({ type: "error", message: err instanceof Error ? err.message : String(err) });
    } finally {
      setOperatingOn(null);
    }
  };

  const filteredPackages = packages.filter(pkg => {
    const matchesSearch = pkg.name.toLowerCase().includes(searchQuery.toLowerCase());
    const matchesFilter =
      filter === "all" ||
      (filter === "formulae" && !pkg.is_cask) ||
      (filter === "casks" && pkg.is_cask) ||
      (filter === "outdated" && pkg.is_outdated);
    return matchesSearch && matchesFilter;
  });

  if (isLoading) {
    return (
      <div className="p-6 flex items-center justify-center h-full">
        <div className="text-center">
          <RefreshCw size={32} className="animate-spin mx-auto text-primary-400 mb-4" />
          <p className="text-gray-400">Cargando informacion de Homebrew...</p>
        </div>
      </div>
    );
  }

  if (!homebrewInfo?.installed) {
    return (
      <div className="p-6">
        <div className="bg-dark-card border border-dark-border rounded-xl p-8 text-center">
          <Beer size={48} className="mx-auto text-gray-500 mb-4" />
          <h2 className="text-xl font-bold mb-2">Homebrew no instalado</h2>
          <p className="text-gray-400 mb-4">
            Homebrew es un gestor de paquetes para macOS. Instalalo para usar esta funcion.
          </p>
          <a
            href="https://brew.sh"
            target="_blank"
            rel="noopener noreferrer"
            className="inline-flex items-center gap-2 px-4 py-2 bg-primary-600 hover:bg-primary-700 rounded-lg transition-colors"
          >
            Visitar brew.sh
          </a>
        </div>
      </div>
    );
  }

  return (
    <div className="p-6">
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <h2 className="text-2xl font-bold">Homebrew</h2>
          <p className="text-gray-400 mt-1">
            Gestiona tus paquetes de Homebrew
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

      {/* Operation status */}
      {operationStatus && (
        <div className={`mb-6 p-4 rounded-lg flex items-center gap-3 ${
          operationStatus.type === "success"
            ? "bg-green-500/10 border border-green-500/30"
            : "bg-red-500/10 border border-red-500/30"
        }`}>
          {operationStatus.type === "success" ? (
            <CheckCircle2 size={20} className="text-green-400" />
          ) : (
            <AlertCircle size={20} className="text-red-400" />
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
              <Beer size={20} className="text-primary-400" />
            </div>
            <div>
              <p className="text-2xl font-bold">{homebrewInfo.version || "?"}</p>
              <p className="text-sm text-gray-400">Version</p>
            </div>
          </div>
        </div>
        <div className="bg-dark-card border border-dark-border rounded-xl p-4">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-blue-500/20 rounded-lg">
              <Package size={20} className="text-blue-400" />
            </div>
            <div>
              <p className="text-2xl font-bold">{homebrewInfo.formulae_count}</p>
              <p className="text-sm text-gray-400">Formulae</p>
            </div>
          </div>
        </div>
        <div className="bg-dark-card border border-dark-border rounded-xl p-4">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-purple-500/20 rounded-lg">
              <Box size={20} className="text-purple-400" />
            </div>
            <div>
              <p className="text-2xl font-bold">{homebrewInfo.casks_count}</p>
              <p className="text-sm text-gray-400">Casks</p>
            </div>
          </div>
        </div>
        <div className="bg-dark-card border border-dark-border rounded-xl p-4">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-yellow-500/20 rounded-lg">
              <ArrowUpCircle size={20} className="text-yellow-400" />
            </div>
            <div>
              <p className="text-2xl font-bold">{outdatedPackages.length}</p>
              <p className="text-sm text-gray-400">Desactualizados</p>
            </div>
          </div>
        </div>
      </div>

      {/* Action buttons */}
      {outdatedPackages.length > 0 && (
        <div className="bg-gradient-to-r from-yellow-600/20 to-yellow-800/20 border border-yellow-500/30 rounded-xl p-4 mb-6">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <ArrowUpCircle size={24} className="text-yellow-400" />
              <div>
                <p className="font-medium">Hay {outdatedPackages.length} paquetes desactualizados</p>
                <p className="text-sm text-gray-400">Actualiza todos o selecciona individualmente</p>
              </div>
            </div>
            <div className="flex gap-2">
              <button
                onClick={handleCleanup}
                disabled={operatingOn !== null}
                className="flex items-center gap-2 px-4 py-2 bg-dark-card border border-dark-border rounded-lg hover:bg-dark-border transition-colors disabled:opacity-50"
              >
                {operatingOn === "cleanup" ? (
                  <RefreshCw size={18} className="animate-spin" />
                ) : (
                  <Sparkles size={18} />
                )}
                <span>Limpiar</span>
              </button>
              <button
                onClick={handleUpgradeAll}
                disabled={operatingOn !== null}
                className="flex items-center gap-2 px-4 py-2 bg-yellow-600 hover:bg-yellow-700 rounded-lg transition-colors disabled:opacity-50"
              >
                {operatingOn === "all" ? (
                  <RefreshCw size={18} className="animate-spin" />
                ) : (
                  <ArrowUpCircle size={18} />
                )}
                <span>Actualizar Todo</span>
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Search and filter */}
      <div className="flex gap-4 mb-4">
        <div className="flex-1 relative">
          <Search size={18} className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-500" />
          <input
            type="text"
            placeholder="Buscar paquetes..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full pl-10 pr-4 py-2 bg-dark-card border border-dark-border rounded-lg focus:outline-none focus:border-primary-500"
          />
        </div>
        <div className="flex bg-dark-card border border-dark-border rounded-lg overflow-hidden">
          {[
            { id: "all", label: "Todos" },
            { id: "formulae", label: "Formulae" },
            { id: "casks", label: "Casks" },
            { id: "outdated", label: "Desactualizados" },
          ].map((f) => (
            <button
              key={f.id}
              onClick={() => setFilter(f.id as typeof filter)}
              className={`px-4 py-2 text-sm transition-colors ${
                filter === f.id
                  ? "bg-primary-500/20 text-primary-400"
                  : "text-gray-400 hover:text-white hover:bg-dark-border"
              }`}
            >
              {f.label}
            </button>
          ))}
        </div>
      </div>

      {/* Packages list */}
      <div className="bg-dark-card border border-dark-border rounded-xl overflow-hidden">
        <div className="grid grid-cols-12 gap-4 px-4 py-3 bg-dark-border/50 text-sm font-medium text-gray-400">
          <div className="col-span-4">Paquete</div>
          <div className="col-span-2">Version</div>
          <div className="col-span-2">Nueva Version</div>
          <div className="col-span-2">Tipo</div>
          <div className="col-span-2 text-right">Acciones</div>
        </div>
        <div className="divide-y divide-dark-border max-h-96 overflow-y-auto">
          {filteredPackages.length === 0 ? (
            <div className="p-8 text-center text-gray-500">
              No se encontraron paquetes
            </div>
          ) : (
            filteredPackages.map((pkg) => (
              <div
                key={`${pkg.name}-${pkg.is_cask}`}
                className={`grid grid-cols-12 gap-4 px-4 py-3 items-center hover:bg-dark-border/30 transition-colors ${
                  pkg.is_outdated ? "bg-yellow-500/5" : ""
                }`}
              >
                <div className="col-span-4 flex items-center gap-2">
                  {pkg.is_cask ? (
                    <Box size={16} className="text-purple-400" />
                  ) : (
                    <Package size={16} className="text-blue-400" />
                  )}
                  <span className="font-medium truncate">{pkg.name}</span>
                </div>
                <div className="col-span-2 text-gray-400">{pkg.version}</div>
                <div className="col-span-2">
                  {pkg.newer_version ? (
                    <span className="text-yellow-400">{pkg.newer_version}</span>
                  ) : (
                    <span className="text-gray-600">-</span>
                  )}
                </div>
                <div className="col-span-2">
                  <span className={`px-2 py-1 rounded text-xs ${
                    pkg.is_cask
                      ? "bg-purple-500/20 text-purple-400"
                      : "bg-blue-500/20 text-blue-400"
                  }`}>
                    {pkg.is_cask ? "Cask" : "Formula"}
                  </span>
                </div>
                <div className="col-span-2 flex justify-end gap-2">
                  {pkg.is_outdated && (
                    <button
                      onClick={() => handleUpgrade(pkg)}
                      disabled={operatingOn !== null}
                      className="p-2 text-yellow-400 hover:bg-yellow-500/20 rounded-lg transition-colors disabled:opacity-50"
                      title="Actualizar"
                    >
                      {operatingOn === pkg.name ? (
                        <RefreshCw size={16} className="animate-spin" />
                      ) : (
                        <ArrowUpCircle size={16} />
                      )}
                    </button>
                  )}
                  <button
                    onClick={() => handleUninstall(pkg)}
                    disabled={operatingOn !== null}
                    className="p-2 text-red-400 hover:bg-red-500/20 rounded-lg transition-colors disabled:opacity-50"
                    title="Desinstalar"
                  >
                    <Trash2 size={16} />
                  </button>
                </div>
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}

export default HomebrewView;
