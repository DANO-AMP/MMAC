import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  FileQuestion,
  RefreshCw,
  Trash2,
  FolderOpen,
  Database,
  Settings2,
  Box,
  Archive,
  AlertTriangle,
  CheckCircle2,
  Search,
} from "lucide-react";
import { formatSize } from "../utils";
import { ErrorBanner } from "../components/ErrorBanner";

interface OrphanedFile {
  path: string;
  size: number;
  likely_app: string;
  last_accessed: number;
  file_type: string;
}

interface OrphanedScanResult {
  files: OrphanedFile[];
  total_size: number;
  total_count: number;
}

function OrphanedView() {
  const [scanResult, setScanResult] = useState<OrphanedScanResult | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isScanning, setIsScanning] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedFiles, setSelectedFiles] = useState<Set<string>>(new Set());
  const [isDeleting, setIsDeleting] = useState(false);
  const [deleteProgress, setDeleteProgress] = useState(0);
  const [filter, setFilter] = useState<"all" | "data" | "pref" | "cache" | "container">("all");

  const loadData = async (scan = false) => {
    if (scan) {
      setIsScanning(true);
    } else {
      setIsLoading(true);
    }
    setError(null);

    try {
      const result: OrphanedScanResult = await invoke("scan_orphaned_files");
      setScanResult(result);
      setSelectedFiles(new Set());
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsLoading(false);
      setIsScanning(false);
    }
  };

  useEffect(() => {
    let isMounted = true;

    const loadInitialData = async () => {
      if (!isMounted) return;
      setIsLoading(true);
      setError(null);

      try {
        const result: OrphanedScanResult = await invoke("scan_orphaned_files");
        if (isMounted) {
          setScanResult(result);
          setSelectedFiles(new Set());
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

  const getFileTypeIcon = (type: string) => {
    const iconProps = { size: 20 };
    switch (type) {
      case "data":
        return <Database {...iconProps} className="text-blue-400" />;
      case "pref":
        return <Settings2 {...iconProps} className="text-purple-400" />;
      case "cache":
        return <Archive {...iconProps} className="text-yellow-400" />;
      case "container":
      case "group_container":
        return <Box {...iconProps} className="text-green-400" />;
      case "state":
        return <FolderOpen {...iconProps} className="text-orange-400" />;
      default:
        return <FileQuestion {...iconProps} className="text-gray-400" />;
    }
  };

  const getFileTypeLabel = (type: string) => {
    switch (type) {
      case "data":
        return "App Data";
      case "pref":
        return "Preferences";
      case "cache":
        return "Cache";
      case "container":
        return "Container";
      case "group_container":
        return "Group Container";
      case "state":
        return "Saved State";
      default:
        return type;
    }
  };

  const formatDate = (timestamp: number) => {
    if (timestamp === 0) return "Desconocido";
    const date = new Date(timestamp * 1000);
    return date.toLocaleDateString("es-ES", {
      year: "numeric",
      month: "short",
      day: "numeric",
    });
  };

  const toggleFile = (path: string) => {
    const newSelected = new Set(selectedFiles);
    if (newSelected.has(path)) {
      newSelected.delete(path);
    } else {
      newSelected.add(path);
    }
    setSelectedFiles(newSelected);
  };

  const toggleAll = () => {
    if (selectedFiles.size === filteredFiles.length) {
      setSelectedFiles(new Set());
    } else {
      setSelectedFiles(new Set(filteredFiles.map(f => f.path)));
    }
  };

  const handleDelete = async () => {
    if (selectedFiles.size === 0) return;

    const filesToDelete = Array.from(selectedFiles);
    setIsDeleting(true);
    setDeleteProgress(0);

    let deletedCount = 0;
    for (const path of filesToDelete) {
      try {
        await invoke("delete_orphaned_file", { path });
        deletedCount++;
        setDeleteProgress((deletedCount / filesToDelete.length) * 100);
      } catch (err) {
        console.error(`Failed to delete ${path}:`, err);
      }
    }

    setIsDeleting(false);
    setDeleteProgress(0);
    loadData(true);
  };

  const filteredFiles = (scanResult?.files || []).filter(file => {
    const matchesSearch =
      file.likely_app.toLowerCase().includes(searchQuery.toLowerCase()) ||
      file.path.toLowerCase().includes(searchQuery.toLowerCase());
    const matchesFilter =
      filter === "all" ||
      file.file_type === filter ||
      (filter === "container" && file.file_type.includes("container"));
    return matchesSearch && matchesFilter;
  });

  const selectedSize = filteredFiles
    .filter(f => selectedFiles.has(f.path))
    .reduce((acc, f) => acc + f.size, 0);

  if (isLoading) {
    return (
      <div className="p-6 flex items-center justify-center h-full">
        <div className="text-center">
          <RefreshCw size={32} className="animate-spin mx-auto text-primary-400 mb-4" />
          <p className="text-gray-400">Escaneando archivos huerfanos...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="p-6">
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <h2 className="text-2xl font-bold">Archivos Huerfanos</h2>
          <p className="text-gray-400 mt-1">
            Archivos de aplicaciones que ya no estan instaladas
          </p>
        </div>
        <button
          onClick={() => loadData(true)}
          disabled={isScanning}
          className="flex items-center gap-2 px-4 py-2 bg-dark-card border border-dark-border rounded-lg hover:bg-dark-border transition-colors disabled:opacity-50"
          aria-label="Escanear archivos huerfanos"
        >
          <RefreshCw size={18} className={isScanning ? "animate-spin" : ""} aria-hidden="true" />
          <span>Escanear</span>
        </button>
      </div>

      {/* Error banner */}
      {error && (
        <ErrorBanner error={error} onRetry={() => loadData()} className="mb-6" />
      )}

      {/* Summary card */}
      <div className="bg-gradient-to-r from-orange-600/20 to-orange-800/20 border border-orange-500/30 rounded-xl p-6 mb-6">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4">
            <div className="p-3 bg-orange-500/20 rounded-xl">
              <FileQuestion size={32} className="text-orange-400" />
            </div>
            <div>
              <p className="text-gray-400 text-sm">Espacio recuperable</p>
              <p className="text-3xl font-bold text-orange-400">
                {formatSize(scanResult?.total_size || 0)}
              </p>
              <p className="text-sm text-gray-400 mt-1">
                {scanResult?.total_count || 0} archivos encontrados
              </p>
            </div>
          </div>
          {selectedFiles.size > 0 && (
            <div className="text-right">
              <button
                onClick={handleDelete}
                disabled={isDeleting}
                className="flex items-center gap-2 px-6 py-3 bg-red-600 hover:bg-red-700 text-white rounded-xl font-medium transition-colors disabled:opacity-50"
                aria-label={`Eliminar ${selectedFiles.size} archivos seleccionados`}
              >
                {isDeleting ? (
                  <>
                    <RefreshCw size={20} className="animate-spin" aria-hidden="true" />
                    <span>Eliminando...</span>
                  </>
                ) : (
                  <>
                    <Trash2 size={20} aria-hidden="true" />
                    <span>Eliminar ({selectedFiles.size})</span>
                  </>
                )}
              </button>
              <p className="text-sm text-gray-400 mt-2" aria-live="polite">
                {formatSize(selectedSize)} seleccionados
              </p>
            </div>
          )}
        </div>

        {isDeleting && (
          <div className="mt-4" role="status" aria-live="polite">
            <div className="h-2 bg-dark-bg rounded-full overflow-hidden" role="progressbar" aria-valuenow={Math.round(deleteProgress)} aria-valuemin={0} aria-valuemax={100} aria-label="Progreso de eliminacion">
              <div
                className="h-full bg-red-500 transition-all duration-300"
                style={{ width: `${deleteProgress}%` }}
              />
            </div>
            <p className="text-sm text-gray-400 mt-2">
              {Math.round(deleteProgress)}% completado
            </p>
          </div>
        )}
      </div>

      {/* Warning */}
      <div className="bg-yellow-500/10 border border-yellow-500/30 rounded-xl p-4 mb-6 flex items-start gap-3">
        <AlertTriangle size={20} className="text-yellow-400 flex-shrink-0 mt-0.5" />
        <div>
          <p className="text-sm text-yellow-200">
            Revisa cuidadosamente antes de eliminar. Algunos archivos pueden pertenecer a aplicaciones
            que usas ocasionalmente o cuyo nombre no fue detectado correctamente.
          </p>
        </div>
      </div>

      {/* Search and filter */}
      <div className="flex gap-4 mb-4">
        <div className="flex-1 relative">
          <Search size={18} className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-500" aria-hidden="true" />
          <input
            type="text"
            placeholder="Buscar por nombre de app o ruta..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full pl-10 pr-4 py-2 bg-dark-card border border-dark-border rounded-lg focus:outline-none focus:border-primary-500"
            aria-label="Buscar archivos huerfanos por nombre de app o ruta"
          />
        </div>
        <div className="flex bg-dark-card border border-dark-border rounded-lg overflow-hidden" role="group" aria-label="Filtrar archivos por tipo">
          {[
            { id: "all", label: "Todos" },
            { id: "data", label: "Datos" },
            { id: "cache", label: "Cache" },
            { id: "pref", label: "Prefs" },
            { id: "container", label: "Containers" },
          ].map((f) => (
            <button
              key={f.id}
              onClick={() => setFilter(f.id as typeof filter)}
              className={`px-3 py-2 text-sm transition-colors ${
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

      {/* Files list */}
      <div className="bg-dark-card border border-dark-border rounded-xl overflow-hidden">
        <div className="grid grid-cols-12 gap-4 px-4 py-3 bg-dark-border/50 text-sm font-medium text-gray-400 items-center">
          <div className="col-span-1">
            <input
              type="checkbox"
              checked={selectedFiles.size === filteredFiles.length && filteredFiles.length > 0}
              onChange={toggleAll}
              className="w-4 h-4 rounded border-dark-border text-primary-500 focus:ring-primary-500 focus:ring-offset-0 bg-dark-bg"
              aria-label="Seleccionar todos los archivos"
            />
          </div>
          <div className="col-span-4">App Probable</div>
          <div className="col-span-2">Tipo</div>
          <div className="col-span-2">Tamano</div>
          <div className="col-span-2">Ultimo Acceso</div>
          <div className="col-span-1"></div>
        </div>
        <div className="divide-y divide-dark-border max-h-96 overflow-y-auto">
          {filteredFiles.length === 0 ? (
            <div className="p-8 text-center">
              <CheckCircle2 size={48} className="mx-auto text-green-400 mb-4" />
              <p className="text-gray-400">No se encontraron archivos huerfanos</p>
            </div>
          ) : (
            filteredFiles.map((file) => (
              <div
                key={file.path}
                className={`grid grid-cols-12 gap-4 px-4 py-3 items-center hover:bg-dark-border/30 transition-colors ${
                  selectedFiles.has(file.path) ? "bg-primary-500/5" : ""
                }`}
              >
                <div className="col-span-1">
                  <input
                    type="checkbox"
                    checked={selectedFiles.has(file.path)}
                    onChange={() => toggleFile(file.path)}
                    className="w-4 h-4 rounded border-dark-border text-primary-500 focus:ring-primary-500 focus:ring-offset-0 bg-dark-bg"
                    aria-label={`Seleccionar ${file.likely_app}`}
                  />
                </div>
                <div className="col-span-4">
                  <div className="flex items-center gap-2">
                    {getFileTypeIcon(file.file_type)}
                    <div className="min-w-0">
                      <p className="font-medium truncate">{file.likely_app}</p>
                      <p className="text-xs text-gray-500 truncate">{file.path}</p>
                    </div>
                  </div>
                </div>
                <div className="col-span-2">
                  <span className="text-sm text-gray-400">
                    {getFileTypeLabel(file.file_type)}
                  </span>
                </div>
                <div className="col-span-2">
                  <span className="font-medium text-orange-400">
                    {formatSize(file.size)}
                  </span>
                </div>
                <div className="col-span-2 text-sm text-gray-400">
                  {formatDate(file.last_accessed)}
                </div>
                <div className="col-span-1 flex justify-end">
                  <button
                    onClick={() => {
                      setSelectedFiles(new Set([file.path]));
                      handleDelete();
                    }}
                    className="p-2 text-red-400 hover:bg-red-500/20 rounded-lg transition-colors"
                    aria-label={`Eliminar ${file.likely_app}`}
                  >
                    <Trash2 size={16} aria-hidden="true" />
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

export default OrphanedView;
