import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Trash2,
  FileText,
  Globe,
  FolderOpen,
  Play,
  RefreshCw,
  CheckCircle2,
  AlertCircle,
} from "lucide-react";
import { formatSize } from "../utils";
import { ErrorBanner } from "../components/ErrorBanner";

interface CleaningCategory {
  id: string;
  name: string;
  icon: React.ReactNode;
  size: number;
  items: number;
  selected: boolean;
  scanning: boolean;
}

interface ScanResult {
  category: string;
  size: number;
  items: number;
  paths: string[];
}

function CleaningView() {
  const [categories, setCategories] = useState<CleaningCategory[]>([
    {
      id: "cache",
      name: "Caches del Sistema",
      icon: <FolderOpen size={24} />,
      size: 0,
      items: 0,
      selected: true,
      scanning: false,
    },
    {
      id: "logs",
      name: "Logs y Registros",
      icon: <FileText size={24} />,
      size: 0,
      items: 0,
      selected: true,
      scanning: false,
    },
    {
      id: "browser",
      name: "Datos de Navegadores",
      icon: <Globe size={24} />,
      size: 0,
      items: 0,
      selected: true,
      scanning: false,
    },
    {
      id: "trash",
      name: "Papelera",
      icon: <Trash2 size={24} />,
      size: 0,
      items: 0,
      selected: true,
      scanning: false,
    },
  ]);

  const [isScanning, setIsScanning] = useState(false);
  const [isCleaning, setIsCleaning] = useState(false);
  const [cleaningProgress, setCleaningProgress] = useState(0);
  const [lastScanTime, setLastScanTime] = useState<Date | null>(null);
  const [scanError, setScanError] = useState<string | null>(null);

  const totalSize = categories
    .filter((c) => c.selected)
    .reduce((acc, c) => acc + c.size, 0);

  const handleScan = async () => {
    setIsScanning(true);
    setScanError(null);
    setCategories((prev) =>
      prev.map((c) => ({ ...c, scanning: true, size: 0, items: 0 }))
    );

    try {
      const results: ScanResult[] = await invoke("scan_system");
      setCategories((prev) =>
        prev.map((c) => {
          const result = results.find((r) => r.category === c.id);
          return {
            ...c,
            size: result?.size || 0,
            items: result?.items || 0,
            scanning: false,
          };
        })
      );
      setLastScanTime(new Date());
    } catch (error) {
      console.error("Scan error:", error);
      setScanError(error instanceof Error ? error.message : String(error));
      setCategories((prev) =>
        prev.map((c) => ({ ...c, scanning: false }))
      );
    }

    setIsScanning(false);
  };

  const handleClean = async () => {
    if (totalSize === 0) return;

    setIsCleaning(true);
    setCleaningProgress(0);

    const selectedCategories = categories.filter((c) => c.selected);

    for (let i = 0; i < selectedCategories.length; i++) {
      try {
        await invoke("clean_category", { category: selectedCategories[i].id });
      } catch (error) {
        console.error("Clean error:", error);
      }
      setCleaningProgress(((i + 1) / selectedCategories.length) * 100);
    }

    setCategories((prev) =>
      prev.map((c) =>
        c.selected ? { ...c, size: 0, items: 0 } : c
      )
    );
    setIsCleaning(false);
    setCleaningProgress(0);
  };

  const toggleCategory = (id: string) => {
    setCategories((prev) =>
      prev.map((c) => (c.id === id ? { ...c, selected: !c.selected } : c))
    );
  };

  useEffect(() => {
    handleScan();
  }, []);

  return (
    <div className="p-6">
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <h2 className="text-2xl font-bold">Limpieza del Sistema</h2>
          <p className="text-gray-400 mt-1">
            Libera espacio eliminando archivos innecesarios
          </p>
        </div>
        <button
          onClick={handleScan}
          disabled={isScanning}
          className="flex items-center gap-2 px-4 py-2 bg-dark-card border border-dark-border rounded-lg hover:bg-dark-border transition-colors disabled:opacity-50"
        >
          <RefreshCw size={18} className={isScanning ? "animate-spin" : ""} />
          <span>Escanear</span>
        </button>
      </div>

      {/* Error banner */}
      {scanError && (
        <ErrorBanner
          error={scanError}
          onRetry={handleScan}
          className="mb-6"
        />
      )}

      {/* Total summary card */}
      <div className="bg-gradient-to-r from-primary-600/20 to-primary-800/20 border border-primary-500/30 rounded-xl p-6 mb-6">
        <div className="flex items-center justify-between">
          <div>
            <p className="text-gray-400 text-sm">Espacio a liberar</p>
            <p className="text-4xl font-bold text-primary-400 mt-1">
              {formatSize(totalSize)}
            </p>
            {lastScanTime && (
              <p className="text-xs text-gray-500 mt-2">
                Ultimo escaneo: {lastScanTime.toLocaleTimeString()}
              </p>
            )}
          </div>
          <button
            onClick={handleClean}
            disabled={isCleaning || totalSize === 0}
            className="flex items-center gap-2 px-6 py-3 bg-primary-600 hover:bg-primary-700 text-white rounded-xl font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {isCleaning ? (
              <>
                <RefreshCw size={20} className="animate-spin" />
                <span>Limpiando...</span>
              </>
            ) : (
              <>
                <Play size={20} />
                <span>Iniciar Limpieza</span>
              </>
            )}
          </button>
        </div>

        {/* Progress bar */}
        {isCleaning && (
          <div className="mt-4">
            <div className="h-2 bg-dark-bg rounded-full overflow-hidden">
              <div
                className="h-full bg-primary-500 transition-all duration-300 progress-active"
                style={{ width: `${cleaningProgress}%` }}
              />
            </div>
            <p className="text-sm text-gray-400 mt-2">
              {Math.round(cleaningProgress)}% completado
            </p>
          </div>
        )}
      </div>

      {/* Categories grid */}
      <div className="grid grid-cols-2 gap-4">
        {categories.map((category) => (
          <div
            key={category.id}
            onClick={() => toggleCategory(category.id)}
            className={`bg-dark-card border rounded-xl p-5 cursor-pointer transition-all card-hover ${
              category.selected
                ? "border-primary-500/50 ring-1 ring-primary-500/20"
                : "border-dark-border hover:border-dark-border/80"
            }`}
          >
            <div className="flex items-start justify-between">
              <div
                className={`p-3 rounded-lg ${
                  category.selected ? "bg-primary-500/20" : "bg-dark-border/50"
                }`}
              >
                <div
                  className={
                    category.selected ? "text-primary-400" : "text-gray-400"
                  }
                >
                  {category.icon}
                </div>
              </div>
              <div
                className={`w-5 h-5 rounded-full border-2 flex items-center justify-center ${
                  category.selected
                    ? "border-primary-500 bg-primary-500"
                    : "border-gray-500"
                }`}
              >
                {category.selected && (
                  <CheckCircle2 size={14} className="text-white" />
                )}
              </div>
            </div>

            <h3 className="font-semibold mt-4">{category.name}</h3>

            {category.scanning ? (
              <div className="flex items-center gap-2 mt-2 text-gray-400">
                <RefreshCw size={14} className="animate-spin" />
                <span className="text-sm">Escaneando...</span>
              </div>
            ) : (
              <div className="mt-2">
                <p className="text-2xl font-bold text-primary-400">
                  {formatSize(category.size)}
                </p>
                <p className="text-sm text-gray-400">
                  {category.items} elementos
                </p>
              </div>
            )}
          </div>
        ))}
      </div>

      {/* Info banner */}
      <div className="mt-6 bg-dark-card border border-dark-border rounded-xl p-4 flex items-start gap-3">
        <AlertCircle size={20} className="text-primary-400 flex-shrink-0 mt-0.5" />
        <div>
          <p className="text-sm text-gray-300">
            Los archivos se moveran a la Papelera antes de eliminarlos permanentemente.
            Puedes recuperarlos si es necesario.
          </p>
        </div>
      </div>
    </div>
  );
}

export default CleaningView;
