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

function formatSize(bytes: number): string {
  if (!bytes || bytes <= 0 || !isFinite(bytes)) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.min(Math.floor(Math.log(bytes) / Math.log(k)), sizes.length - 1);
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
}

function getColorForSize(size: number, maxSize: number): string {
  const ratio = size / maxSize;
  if (ratio > 0.3) return "bg-red-500";
  if (ratio > 0.15) return "bg-orange-500";
  if (ratio > 0.05) return "bg-yellow-500";
  return "bg-blue-500";
}

function AnalyzerView() {
  const [diskInfo, setDiskInfo] = useState<DiskInfo>({
    total: 500 * 1024 * 1024 * 1024,
    used: 234 * 1024 * 1024 * 1024,
    available: 266 * 1024 * 1024 * 1024,
  });
  const [items, setItems] = useState<DiskItem[]>([]);
  const [currentPath, setCurrentPath] = useState<string[]>(["~"]);
  const [isLoading, setIsLoading] = useState(false);
  const [selectedItem, setSelectedItem] = useState<DiskItem | null>(null);

  useEffect(() => {
    loadPath(currentPath.join("/"));
  }, []);

  const loadPath = async (path: string) => {
    setIsLoading(true);
    try {
      const result: DiskItem[] = await invoke("analyze_path", { path });
      setItems(result.sort((a, b) => b.size - a.size));
    } catch (error) {
      console.error("Analyze error:", error);
      // Demo data
      setItems([
        {
          name: "Library",
          path: "~/Library",
          size: 45 * 1024 * 1024 * 1024,
          is_dir: true,
        },
        {
          name: "Documents",
          path: "~/Documents",
          size: 32 * 1024 * 1024 * 1024,
          is_dir: true,
        },
        {
          name: "Downloads",
          path: "~/Downloads",
          size: 18 * 1024 * 1024 * 1024,
          is_dir: true,
        },
        {
          name: "Movies",
          path: "~/Movies",
          size: 15 * 1024 * 1024 * 1024,
          is_dir: true,
        },
        {
          name: "Desktop",
          path: "~/Desktop",
          size: 8 * 1024 * 1024 * 1024,
          is_dir: true,
        },
        {
          name: "Pictures",
          path: "~/Pictures",
          size: 6 * 1024 * 1024 * 1024,
          is_dir: true,
        },
        {
          name: "Music",
          path: "~/Music",
          size: 4 * 1024 * 1024 * 1024,
          is_dir: true,
        },
        {
          name: ".npm",
          path: "~/.npm",
          size: 3 * 1024 * 1024 * 1024,
          is_dir: true,
        },
        {
          name: ".cargo",
          path: "~/.cargo",
          size: 2.5 * 1024 * 1024 * 1024,
          is_dir: true,
        },
        {
          name: "Applications",
          path: "~/Applications",
          size: 1.2 * 1024 * 1024 * 1024,
          is_dir: true,
        },
      ].sort((a, b) => b.size - a.size));
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
    try {
      await invoke("reveal_in_finder", { path });
    } catch (error) {
      console.error("Reveal error:", error);
    }
  };

  const moveToTrash = async (path: string) => {
    try {
      await invoke("move_to_trash", { path });
      setItems((prev) => prev.filter((i) => i.path !== path));
      setSelectedItem(null);
    } catch (error) {
      console.error("Delete error:", error);
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

      {/* Breadcrumb */}
      <div className="flex items-center gap-1 mb-4 text-sm overflow-x-auto pb-2">
        {currentPath.map((segment, idx) => (
          <div key={idx} className="flex items-center">
            {idx > 0 && <ChevronRight size={14} className="text-gray-500 mx-1" />}
            <button
              onClick={() => goToPath(idx)}
              className={`px-2 py-1 rounded hover:bg-dark-card transition-colors ${
                idx === currentPath.length - 1
                  ? "text-primary-400 font-medium"
                  : "text-gray-400"
              }`}
            >
              {segment === "~" ? <Home size={14} /> : segment}
            </button>
          </div>
        ))}
        {isLoading && (
          <RefreshCw size={14} className="animate-spin text-primary-400 ml-2" />
        )}
      </div>

      <div className="flex-1 flex gap-6 min-h-0">
        {/* Treemap / List */}
        <div className="flex-1 bg-dark-card border border-dark-border rounded-xl overflow-hidden">
          <div className="h-full overflow-auto p-4">
            {/* Size bars */}
            <div className="space-y-2">
              {items.map((item) => {
                const percentage = (item.size / maxSize) * 100;
                const color = getColorForSize(item.size, totalAnalyzed);

                return (
                  <div
                    key={item.path}
                    onClick={() => setSelectedItem(item)}
                    onDoubleClick={() => navigateTo(item)}
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

              <div className="space-y-2">
                {selectedItem.is_dir && (
                  <button
                    onClick={() => navigateTo(selectedItem)}
                    className="w-full flex items-center justify-center gap-2 px-4 py-2 bg-primary-600 hover:bg-primary-700 text-white rounded-lg transition-colors"
                  >
                    <FolderOpen size={16} />
                    <span>Abrir</span>
                  </button>
                )}

                <button
                  onClick={() => revealInFinder(selectedItem.path)}
                  className="w-full flex items-center justify-center gap-2 px-4 py-2 bg-dark-border hover:bg-dark-border/80 text-white rounded-lg transition-colors"
                >
                  <Eye size={16} />
                  <span>Mostrar en Finder</span>
                </button>

                <button
                  onClick={() => moveToTrash(selectedItem.path)}
                  className="w-full flex items-center justify-center gap-2 px-4 py-2 bg-red-500/20 hover:bg-red-500/30 text-red-400 border border-red-500/30 rounded-lg transition-colors"
                >
                  <Trash2 size={16} />
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
