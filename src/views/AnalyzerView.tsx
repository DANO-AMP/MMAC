import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  HardDrive,
  ChevronRight,
  FolderOpen,
  File,
  Trash2,
  Eye,
  RefreshCw,
  Home,
} from "lucide-react";
import { formatSize } from "../utils";
import { ErrorBanner } from "../components/ErrorBanner";

interface DiskItem {
  name: string;
  path: string;
  size: number;
  is_dir: boolean;
  children?: DiskItem[];
}

interface DiskInfo {
  total: number;
  used: number;
  available: number;
}

function getColorForSize(size: number, maxSize: number): string {
  const ratio = size / maxSize;
  if (ratio > 0.3) return "bg-red-500";
  if (ratio > 0.15) return "bg-orange-500";
  if (ratio > 0.05) return "bg-yellow-500";
  return "bg-blue-500";
}

function AnalyzerView() {
  const [diskInfo] = useState<DiskInfo>({
    total: 500 * 1024 * 1024 * 1024,
    used: 234 * 1024 * 1024 * 1024,
    available: 266 * 1024 * 1024 * 1024,
  });
  const [items, setItems] = useState<DiskItem[]>([]);
  const [currentPath, setCurrentPath] = useState<string[]>(["~"]);
  const [isLoading, setIsLoading] = useState(false);
  const [selectedItem, setSelectedItem] = useState<DiskItem | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);

  useEffect(() => {
    let isMounted = true;

    const loadInitialPath = async () => {
      if (!isMounted) return;
      const path = currentPath.join("/");
      setIsLoading(true);
      setLoadError(null);
      try {
        const result: DiskItem[] = await invoke("analyze_path", { path });
        if (isMounted) {
          setItems(result.sort((a, b) => b.size - a.size));
        }
      } catch (error) {
        console.error("Analyze error:", error);
        if (isMounted) {
          setLoadError(error instanceof Error ? error.message : String(error));
        }
      }
      if (isMounted) {
        setIsLoading(false);
      }
    };

    loadInitialPath();
    return () => { isMounted = false; };
  }, []);

  const loadPath = async (path: string) => {
    setIsLoading(true);
    setLoadError(null);
    try {
      const result: DiskItem[] = await invoke("analyze_path", { path });
      setItems(result.sort((a, b) => b.size - a.size));
    } catch (error) {
      console.error("Analyze error:", error);
      setLoadError(error instanceof Error ? error.message : String(error));
    }
    setIsLoading(false);
  };

  const navigateTo = (item: DiskItem) => {
    if (!item.is_dir) return;
    const newPath = [...currentPath, item.name];
    setCurrentPath(newPath);
    loadPath(newPath.join("/"));
    setSelectedItem(null);
  };

  const goToPath = (index: number) => {
    const newPath = currentPath.slice(0, index + 1);
    setCurrentPath(newPath);
    loadPath(newPath.join("/"));
    setSelectedItem(null);
  };

  const maxSize = items.length > 0 ? items[0].size : 1;
  const totalAnalyzed = items.reduce((acc, item) => acc + item.size, 0);

  const revealInFinder = async (path: string) => {
    setActionError(null);
    try {
      await invoke("reveal_in_finder", { path });
    } catch (error) {
      console.error("Reveal error:", error);
      setActionError(error instanceof Error ? error.message : String(error));
    }
  };

  const moveToTrash = async (path: string) => {
    setActionError(null);
    try {
      await invoke("move_to_trash", { path });
      setItems((prev) => prev.filter((i) => i.path !== path));
      setSelectedItem(null);
    } catch (error) {
      console.error("Delete error:", error);
      setActionError(error instanceof Error ? error.message : String(error));
    }
  };

  return (
    <div className="p-6 h-full flex flex-col">
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <h2 className="text-2xl font-bold">Analizador de Disco</h2>
          <p className="text-gray-400 mt-1">
            Visualiza que ocupa espacio en tu Mac
          </p>
        </div>
      </div>

      {/* Disk overview */}
      <div className="bg-dark-card border border-dark-border rounded-xl p-4 mb-6">
        <div className="flex items-center gap-4 mb-4">
          <HardDrive size={24} className="text-primary-400" />
          <div className="flex-1">
            <div className="flex items-center justify-between mb-2">
              <span className="font-medium">Macintosh HD</span>
              <span className="text-sm text-gray-400">
                {formatSize(diskInfo.used)} de {formatSize(diskInfo.total)} usado
              </span>
            </div>
            <div className="h-3 bg-dark-bg rounded-full overflow-hidden">
              <div
                className="h-full bg-gradient-to-r from-primary-500 to-primary-700 transition-all"
                style={{ width: `${(diskInfo.used / diskInfo.total) * 100}%` }}
              />
            </div>
          </div>
        </div>
        <div className="flex gap-6 text-sm">
          <div>
            <span className="text-gray-400">Disponible: </span>
            <span className="font-medium text-green-400">
              {formatSize(diskInfo.available)}
            </span>
          </div>
          <div>
            <span className="text-gray-400">Analizado: </span>
            <span className="font-medium">{formatSize(totalAnalyzed)}</span>
          </div>
        </div>
      </div>

      {/* Error banners */}
      {loadError && (
        <ErrorBanner
          error={loadError}
          onRetry={() => loadPath(currentPath.join("/"))}
          className="mb-4"
        />
      )}
      {actionError && (
        <ErrorBanner
          error={actionError}
          onRetry={() => setActionError(null)}
          className="mb-4"
        />
      )}

      {/* Breadcrumb */}
      <nav className="flex items-center gap-1 mb-4 text-sm overflow-x-auto pb-2" aria-label="Ruta de navegacion">
        {currentPath.map((segment, idx) => (
          <div key={idx} className="flex items-center">
            {idx > 0 && <ChevronRight size={14} className="text-gray-500 mx-1" aria-hidden="true" />}
            <button
              onClick={() => goToPath(idx)}
              className={`px-2 py-1 rounded hover:bg-dark-card transition-colors ${
                idx === currentPath.length - 1
                  ? "text-primary-400 font-medium"
                  : "text-gray-400"
              }`}
              aria-label={segment === "~" ? "Carpeta de inicio" : `Ir a ${segment}`}
              aria-current={idx === currentPath.length - 1 ? "page" : undefined}
            >
              {segment === "~" ? <Home size={14} aria-hidden="true" /> : segment}
            </button>
          </div>
        ))}
        {isLoading && (
          <RefreshCw size={14} className="animate-spin text-primary-400 ml-2" role="status" aria-label="Cargando" />
        )}
      </nav>

      <div className="flex-1 flex gap-6 min-h-0">
        {/* Treemap / List */}
        <div className="flex-1 bg-dark-card border border-dark-border rounded-xl overflow-hidden">
          <div className="h-full overflow-auto p-4">
            {/* Size bars */}
            <div className="space-y-2" role="list" aria-label="Lista de archivos y carpetas">
              {items.map((item) => {
                const percentage = (item.size / maxSize) * 100;
                const color = getColorForSize(item.size, totalAnalyzed);

                return (
                  <div
                    key={item.path}
                    onClick={() => setSelectedItem(item)}
                    onDoubleClick={() => navigateTo(item)}
                    onKeyDown={(e) => {
                      if (e.key === "Enter") {
                        if (item.is_dir) navigateTo(item);
                        else setSelectedItem(item);
                      }
                    }}
                    tabIndex={0}
                    role="listitem"
                    aria-label={`${item.name}, ${formatSize(item.size)}, ${item.is_dir ? "carpeta" : "archivo"}`}
                    aria-selected={selectedItem?.path === item.path}
                    className={`relative cursor-pointer rounded-lg overflow-hidden transition-all ${
                      selectedItem?.path === item.path
                        ? "ring-2 ring-primary-500"
                        : "hover:ring-1 hover:ring-dark-border"
                    }`}
                  >
                    <div
                      className={`absolute inset-0 ${color} opacity-20`}
                      style={{ width: `${percentage}%` }}
                    />
                    <div className="relative flex items-center gap-3 p-3">
                      <div
                        className={`p-2 rounded-lg ${
                          item.is_dir ? "bg-blue-500/20" : "bg-gray-500/20"
                        }`}
                      >
                        {item.is_dir ? (
                          <FolderOpen
                            size={18}
                            className="text-blue-400"
                          />
                        ) : (
                          <File size={18} className="text-gray-400" />
                        )}
                      </div>
                      <div className="flex-1 min-w-0">
                        <p className="font-medium truncate">{item.name}</p>
                        <p className="text-sm text-gray-400">
                          {formatSize(item.size)}
                        </p>
                      </div>
                      <div className="text-right">
                        <p className="text-sm font-mono text-gray-400">
                          {((item.size / totalAnalyzed) * 100).toFixed(1)}%
                        </p>
                      </div>
                      {item.is_dir && (
                        <ChevronRight size={18} className="text-gray-500" />
                      )}
                    </div>
                  </div>
                );
              })}
            </div>

            {items.length === 0 && !isLoading && (
              <div className="flex items-center justify-center h-full text-gray-400">
                <p>No hay elementos para mostrar</p>
              </div>
            )}
          </div>
        </div>

        {/* Details panel */}
        {selectedItem && (
          <div className="w-72">
            <div className="bg-dark-card border border-dark-border rounded-xl p-4 sticky top-6">
              <div className="flex items-center gap-3 mb-4">
                <div
                  className={`p-3 rounded-xl ${
                    selectedItem.is_dir ? "bg-blue-500/20" : "bg-gray-500/20"
                  }`}
                >
                  {selectedItem.is_dir ? (
                    <FolderOpen size={24} className="text-blue-400" />
                  ) : (
                    <File size={24} className="text-gray-400" />
                  )}
                </div>
                <div className="flex-1 min-w-0">
                  <p className="font-medium truncate">{selectedItem.name}</p>
                  <p className="text-xs text-gray-400">
                    {selectedItem.is_dir ? "Carpeta" : "Archivo"}
                  </p>
                </div>
              </div>

              <div className="space-y-3 mb-6">
                <div>
                  <p className="text-xs text-gray-400">Tamano</p>
                  <p className="text-2xl font-bold text-primary-400">
                    {formatSize(selectedItem.size)}
                  </p>
                </div>
                <div>
                  <p className="text-xs text-gray-400">Ruta</p>
                  <p className="text-xs font-mono text-gray-300 break-all">
                    {selectedItem.path}
                  </p>
                </div>
              </div>

              <div className="space-y-2" role="group" aria-label="Acciones">
                {selectedItem.is_dir && (
                  <button
                    onClick={() => navigateTo(selectedItem)}
                    className="w-full flex items-center justify-center gap-2 px-4 py-2 bg-primary-600 hover:bg-primary-700 text-white rounded-lg transition-colors"
                    aria-label={`Abrir carpeta ${selectedItem.name}`}
                  >
                    <FolderOpen size={16} aria-hidden="true" />
                    <span>Abrir</span>
                  </button>
                )}

                <button
                  onClick={() => revealInFinder(selectedItem.path)}
                  className="w-full flex items-center justify-center gap-2 px-4 py-2 bg-dark-border hover:bg-dark-border/80 text-white rounded-lg transition-colors"
                  aria-label={`Mostrar ${selectedItem.name} en Finder`}
                >
                  <Eye size={16} aria-hidden="true" />
                  <span>Mostrar en Finder</span>
                </button>

                <button
                  onClick={() => moveToTrash(selectedItem.path)}
                  className="w-full flex items-center justify-center gap-2 px-4 py-2 bg-red-500/20 hover:bg-red-500/30 text-red-400 border border-red-500/30 rounded-lg transition-colors"
                  aria-label={`Mover ${selectedItem.name} a la papelera`}
                >
                  <Trash2 size={16} aria-hidden="true" />
                  <span>Mover a Papelera</span>
                </button>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

export default AnalyzerView;
