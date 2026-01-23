import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Copy, Search, Trash2, FolderOpen, RefreshCw } from "lucide-react";

interface DuplicateGroup {
  hash: string;
  size: number;
  files: string[];
}

interface DuplicateScanResult {
  groups: DuplicateGroup[];
  total_wasted: number;
  files_scanned: number;
}

function formatSize(bytes: number): string {
  if (!bytes || bytes <= 0 || !isFinite(bytes)) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.min(Math.floor(Math.log(bytes) / Math.log(k)), sizes.length - 1);
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
}

export default function DuplicatesView() {
  const [scanning, setScanning] = useState(false);
  const [result, setResult] = useState<DuplicateScanResult | null>(null);
  const [path, setPath] = useState("~");
  const [minSize, setMinSize] = useState(1); // MB

  const startScan = async () => {
    setScanning(true);
    setResult(null);
    try {
      const expandedPath = path.startsWith("~")
        ? path.replace("~", "/Users/" + (await getUsername()))
        : path;

      const scanResult = await invoke<DuplicateScanResult>("scan_duplicates", {
        path: expandedPath,
        minSize: minSize * 1024 * 1024, // Convert MB to bytes
      });
      setResult(scanResult);
    } catch (error) {
      console.error("Error scanning:", error);
    } finally {
      setScanning(false);
    }
  };

  const getUsername = async (): Promise<string> => {
    // Simple way to get username from home dir
    return "me"; // This should be dynamic
  };

  const deleteFile = async (filePath: string) => {
    try {
      await invoke("move_to_trash", { path: filePath });
      // Refresh the scan
      startScan();
    } catch (error) {
      console.error("Error deleting file:", error);
    }
  };

  const revealInFinder = async (filePath: string) => {
    try {
      await invoke("reveal_in_finder", { path: filePath });
    } catch (error) {
      console.error("Error revealing file:", error);
    }
  };

  return (
    <div className="p-6 space-y-6">
      <div>
        <h2 className="text-2xl font-bold flex items-center gap-3">
          <Copy className="text-primary-400" />
          Buscador de Duplicados
        </h2>
        <p className="text-gray-400 mt-1">
          Encuentra y elimina archivos duplicados por contenido
        </p>
      </div>

      {/* Scan options */}
      <div className="bg-dark-card rounded-xl border border-dark-border p-4 space-y-4">
        <div className="grid grid-cols-2 gap-4">
          <div>
            <label className="block text-sm text-gray-400 mb-2">
              Carpeta a analizar
            </label>
            <input
              type="text"
              value={path}
              onChange={(e) => setPath(e.target.value)}
              className="w-full px-4 py-2 bg-dark-bg border border-dark-border rounded-lg focus:outline-none focus:border-primary-500"
              placeholder="~/Documents"
            />
          </div>
          <div>
            <label className="block text-sm text-gray-400 mb-2">
              Tamaño mínimo (MB)
            </label>
            <input
              type="number"
              value={minSize}
              onChange={(e) => setMinSize(Number(e.target.value))}
              min={0}
              className="w-full px-4 py-2 bg-dark-bg border border-dark-border rounded-lg focus:outline-none focus:border-primary-500"
            />
          </div>
        </div>
        <button
          onClick={startScan}
          disabled={scanning}
          className="flex items-center gap-2 px-6 py-2.5 bg-primary-500 hover:bg-primary-600 rounded-lg font-medium transition-colors disabled:opacity-50"
        >
          {scanning ? (
            <>
              <RefreshCw className="animate-spin" size={18} />
              Escaneando...
            </>
          ) : (
            <>
              <Search size={18} />
              Buscar Duplicados
            </>
          )}
        </button>
      </div>

      {/* Results */}
      {result && (
        <div className="space-y-4">
          {/* Summary */}
          <div className="grid grid-cols-3 gap-4">
            <div className="bg-dark-card rounded-xl border border-dark-border p-4">
              <p className="text-gray-400 text-sm">Archivos escaneados</p>
              <p className="text-2xl font-bold">
                {result.files_scanned.toLocaleString()}
              </p>
            </div>
            <div className="bg-dark-card rounded-xl border border-dark-border p-4">
              <p className="text-gray-400 text-sm">Grupos de duplicados</p>
              <p className="text-2xl font-bold">{result.groups.length}</p>
            </div>
            <div className="bg-dark-card rounded-xl border border-dark-border p-4">
              <p className="text-gray-400 text-sm">Espacio recuperable</p>
              <p className="text-2xl font-bold text-red-400">
                {formatSize(result.total_wasted)}
              </p>
            </div>
          </div>

          {/* Duplicate groups */}
          {result.groups.length === 0 ? (
            <div className="text-center py-12 text-gray-400">
              No se encontraron duplicados
            </div>
          ) : (
            <div className="space-y-4">
              {result.groups.slice(0, 50).map((group, idx) => (
                <div
                  key={group.hash}
                  className="bg-dark-card rounded-xl border border-dark-border overflow-hidden"
                >
                  <div className="px-4 py-3 border-b border-dark-border bg-dark-bg/50 flex items-center justify-between">
                    <div>
                      <span className="font-medium">
                        Grupo {idx + 1}: {group.files.length} archivos
                      </span>
                      <span className="text-gray-400 ml-3">
                        {formatSize(group.size)} cada uno
                      </span>
                    </div>
                    <span className="text-red-400">
                      -{formatSize(group.size * (group.files.length - 1))}
                    </span>
                  </div>
                  <div className="divide-y divide-dark-border">
                    {group.files.map((file, fileIdx) => (
                      <div
                        key={file}
                        className="flex items-center justify-between p-3 hover:bg-dark-bg/30"
                      >
                        <div className="flex items-center gap-3 flex-1 min-w-0">
                          {fileIdx === 0 && (
                            <span className="px-2 py-0.5 bg-green-500/20 text-green-400 text-xs rounded">
                              Original
                            </span>
                          )}
                          <p className="truncate text-sm">{file}</p>
                        </div>
                        <div className="flex items-center gap-2 ml-4">
                          <button
                            onClick={() => revealInFinder(file)}
                            className="p-2 hover:bg-dark-border rounded-lg transition-colors"
                            title="Mostrar en Finder"
                          >
                            <FolderOpen size={16} />
                          </button>
                          {fileIdx > 0 && (
                            <button
                              onClick={() => deleteFile(file)}
                              className="p-2 hover:bg-red-500/20 text-red-400 rounded-lg transition-colors"
                              title="Eliminar"
                            >
                              <Trash2 size={16} />
                            </button>
                          )}
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
