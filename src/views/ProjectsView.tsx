import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  FolderOpen,
  RefreshCw,
  Trash2,
  CheckCircle2,
  AlertCircle,
  Code,
  Package,
  Box,
} from "lucide-react";

interface ProjectArtifact {
  project_path: string;
  project_name: string;
  artifact_type: string;
  artifact_path: string;
  size: number;
  last_modified: string;
  is_recent: boolean;
}

interface ArtifactSummary {
  type: string;
  count: number;
  total_size: number;
  icon: React.ReactNode;
  color: string;
}

function formatSize(bytes: number): string {
  if (!bytes || bytes <= 0 || !isFinite(bytes)) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.min(Math.floor(Math.log(bytes) / Math.log(k)), sizes.length - 1);
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
}

function formatDate(dateStr: string): string {
  const date = new Date(dateStr);
  const now = new Date();
  const diffDays = Math.floor((now.getTime() - date.getTime()) / (1000 * 60 * 60 * 24));

  if (diffDays === 0) return "Hoy";
  if (diffDays === 1) return "Ayer";
  if (diffDays < 7) return `Hace ${diffDays} dias`;
  if (diffDays < 30) return `Hace ${Math.floor(diffDays / 7)} semanas`;
  return date.toLocaleDateString("es-ES");
}

function ProjectsView() {
  const [artifacts, setArtifacts] = useState<ProjectArtifact[]>([]);
  const [selectedArtifacts, setSelectedArtifacts] = useState<Set<string>>(new Set());
  const [isScanning, setIsScanning] = useState(false);
  const [isCleaning, setIsCleaning] = useState(false);
  const [showRecent, setShowRecent] = useState(false);

  useEffect(() => {
    scanProjects();
  }, []);

  const scanProjects = async () => {
    setIsScanning(true);
    try {
      const result: ProjectArtifact[] = await invoke("scan_project_artifacts");
      setArtifacts(result);
    } catch (error) {
      console.error("Scan error:", error);
      // Demo data
      const demoData: ProjectArtifact[] = [
        {
          project_path: "~/projects/my-react-app",
          project_name: "my-react-app",
          artifact_type: "node_modules",
          artifact_path: "~/projects/my-react-app/node_modules",
          size: 450 * 1024 * 1024,
          last_modified: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000).toISOString(),
          is_recent: false,
        },
        {
          project_path: "~/projects/next-blog",
          project_name: "next-blog",
          artifact_type: "node_modules",
          artifact_path: "~/projects/next-blog/node_modules",
          size: 380 * 1024 * 1024,
          last_modified: new Date(Date.now() - 15 * 24 * 60 * 60 * 1000).toISOString(),
          is_recent: false,
        },
        {
          project_path: "~/projects/rust-cli",
          project_name: "rust-cli",
          artifact_type: "target",
          artifact_path: "~/projects/rust-cli/target",
          size: 2.5 * 1024 * 1024 * 1024,
          last_modified: new Date(Date.now() - 45 * 24 * 60 * 60 * 1000).toISOString(),
          is_recent: false,
        },
        {
          project_path: "~/projects/python-api",
          project_name: "python-api",
          artifact_type: "venv",
          artifact_path: "~/projects/python-api/venv",
          size: 250 * 1024 * 1024,
          last_modified: new Date(Date.now() - 60 * 24 * 60 * 60 * 1000).toISOString(),
          is_recent: false,
        },
        {
          project_path: "~/projects/vue-dashboard",
          project_name: "vue-dashboard",
          artifact_type: "node_modules",
          artifact_path: "~/projects/vue-dashboard/node_modules",
          size: 320 * 1024 * 1024,
          last_modified: new Date(Date.now() - 2 * 24 * 60 * 60 * 1000).toISOString(),
          is_recent: true,
        },
        {
          project_path: "~/projects/go-service",
          project_name: "go-service",
          artifact_type: "build",
          artifact_path: "~/projects/go-service/build",
          size: 85 * 1024 * 1024,
          last_modified: new Date(Date.now() - 90 * 24 * 60 * 60 * 1000).toISOString(),
          is_recent: false,
        },
        {
          project_path: "~/projects/tauri-app",
          project_name: "tauri-app",
          artifact_type: "target",
          artifact_path: "~/projects/tauri-app/src-tauri/target",
          size: 3.2 * 1024 * 1024 * 1024,
          last_modified: new Date(Date.now() - 5 * 24 * 60 * 60 * 1000).toISOString(),
          is_recent: true,
        },
      ];
      setArtifacts(demoData);
    }
    setIsScanning(false);
  };

  const cleanSelected = async () => {
    if (selectedArtifacts.size === 0) return;

    setIsCleaning(true);
    try {
      for (const path of selectedArtifacts) {
        await invoke("delete_artifact", { path });
      }
      setArtifacts((prev) =>
        prev.filter((a) => !selectedArtifacts.has(a.artifact_path))
      );
      setSelectedArtifacts(new Set());
    } catch (error) {
      console.error("Clean error:", error);
    }
    setIsCleaning(false);
  };

  const toggleSelect = (path: string) => {
    const newSelected = new Set(selectedArtifacts);
    if (newSelected.has(path)) {
      newSelected.delete(path);
    } else {
      newSelected.add(path);
    }
    setSelectedArtifacts(newSelected);
  };

  const selectAll = () => {
    const filteredArtifacts = artifacts.filter(
      (a) => showRecent || !a.is_recent
    );
    if (selectedArtifacts.size === filteredArtifacts.length) {
      setSelectedArtifacts(new Set());
    } else {
      setSelectedArtifacts(new Set(filteredArtifacts.map((a) => a.artifact_path)));
    }
  };

  const filteredArtifacts = artifacts.filter(
    (a) => showRecent || !a.is_recent
  );

  const summaries: ArtifactSummary[] = [
    {
      type: "node_modules",
      count: artifacts.filter((a) => a.artifact_type === "node_modules").length,
      total_size: artifacts
        .filter((a) => a.artifact_type === "node_modules")
        .reduce((acc, a) => acc + a.size, 0),
      icon: <Package size={20} />,
      color: "text-green-400 bg-green-500/20",
    },
    {
      type: "target",
      count: artifacts.filter((a) => a.artifact_type === "target").length,
      total_size: artifacts
        .filter((a) => a.artifact_type === "target")
        .reduce((acc, a) => acc + a.size, 0),
      icon: <Box size={20} />,
      color: "text-orange-400 bg-orange-500/20",
    },
    {
      type: "venv",
      count: artifacts.filter((a) => a.artifact_type === "venv").length,
      total_size: artifacts
        .filter((a) => a.artifact_type === "venv")
        .reduce((acc, a) => acc + a.size, 0),
      icon: <Code size={20} />,
      color: "text-blue-400 bg-blue-500/20",
    },
    {
      type: "build",
      count: artifacts.filter((a) => a.artifact_type === "build").length,
      total_size: artifacts
        .filter((a) => a.artifact_type === "build")
        .reduce((acc, a) => acc + a.size, 0),
      icon: <FolderOpen size={20} />,
      color: "text-purple-400 bg-purple-500/20",
    },
  ];

  const totalSize = filteredArtifacts.reduce((acc, a) => acc + a.size, 0);
  const selectedSize = artifacts
    .filter((a) => selectedArtifacts.has(a.artifact_path))
    .reduce((acc, a) => acc + a.size, 0);

  return (
    <div className="p-6">
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <h2 className="text-2xl font-bold">Limpieza de Proyectos</h2>
          <p className="text-gray-400 mt-1">
            Elimina artefactos de desarrollo para liberar espacio
          </p>
        </div>
        <button
          onClick={scanProjects}
          disabled={isScanning}
          className="flex items-center gap-2 px-4 py-2 bg-dark-card border border-dark-border rounded-lg hover:bg-dark-border transition-colors disabled:opacity-50"
        >
          <RefreshCw size={18} className={isScanning ? "animate-spin" : ""} />
          <span>Escanear</span>
        </button>
      </div>

      {/* Summary cards */}
      <div className="grid grid-cols-4 gap-4 mb-6">
        {summaries.map((summary) => (
          <div
            key={summary.type}
            className="bg-dark-card border border-dark-border rounded-xl p-4"
          >
            <div className="flex items-center gap-3 mb-2">
              <div className={`p-2 rounded-lg ${summary.color}`}>
                {summary.icon}
              </div>
              <span className="text-gray-400 text-sm capitalize">
                {summary.type}
              </span>
            </div>
            <p className="text-2xl font-bold">{formatSize(summary.total_size)}</p>
            <p className="text-sm text-gray-400">{summary.count} proyectos</p>
          </div>
        ))}
      </div>

      {/* Action bar */}
      <div className="bg-dark-card border border-dark-border rounded-xl p-4 mb-6">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4">
            <button
              onClick={selectAll}
              className="flex items-center gap-2 text-sm text-gray-400 hover:text-white transition-colors"
            >
              <div
                className={`w-5 h-5 rounded border-2 flex items-center justify-center ${
                  selectedArtifacts.size === filteredArtifacts.length &&
                  filteredArtifacts.length > 0
                    ? "border-primary-500 bg-primary-500"
                    : "border-gray-500"
                }`}
              >
                {selectedArtifacts.size === filteredArtifacts.length &&
                  filteredArtifacts.length > 0 && (
                    <CheckCircle2 size={14} className="text-white" />
                  )}
              </div>
              <span>Seleccionar todo</span>
            </button>

            <div className="h-6 w-px bg-dark-border" />

            <label className="flex items-center gap-2 text-sm text-gray-400 cursor-pointer">
              <input
                type="checkbox"
                checked={showRecent}
                onChange={(e) => setShowRecent(e.target.checked)}
                className="rounded border-gray-500"
              />
              <span>Incluir proyectos recientes (&lt;7 dias)</span>
            </label>
          </div>

          <div className="flex items-center gap-4">
            {selectedArtifacts.size > 0 && (
              <span className="text-sm text-gray-400">
                {selectedArtifacts.size} seleccionados ({formatSize(selectedSize)})
              </span>
            )}
            <button
              onClick={cleanSelected}
              disabled={selectedArtifacts.size === 0 || isCleaning}
              className="flex items-center gap-2 px-4 py-2 bg-red-500/20 hover:bg-red-500/30 text-red-400 border border-red-500/30 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {isCleaning ? (
                <RefreshCw size={16} className="animate-spin" />
              ) : (
                <Trash2 size={16} />
              )}
              <span>Eliminar Seleccionados</span>
            </button>
          </div>
        </div>
      </div>

      {/* Recent warning */}
      {!showRecent && artifacts.some((a) => a.is_recent) && (
        <div className="bg-yellow-500/10 border border-yellow-500/30 rounded-xl p-4 mb-6 flex items-start gap-3">
          <AlertCircle size={20} className="text-yellow-400 flex-shrink-0 mt-0.5" />
          <div>
            <p className="text-yellow-300 font-medium">
              Proyectos recientes ocultos
            </p>
            <p className="text-sm text-yellow-400/80">
              Se han ocultado {artifacts.filter((a) => a.is_recent).length} proyectos
              modificados en los ultimos 7 dias para evitar eliminar trabajo activo.
            </p>
          </div>
        </div>
      )}

      {/* Projects list */}
      <div className="bg-dark-card border border-dark-border rounded-xl overflow-hidden">
        <div className="px-4 py-3 border-b border-dark-border text-sm text-gray-400 flex items-center justify-between">
          <span>
            {filteredArtifacts.length} artefactos encontrados
          </span>
          <span>Total: {formatSize(totalSize)}</span>
        </div>

        <div className="divide-y divide-dark-border/50">
          {filteredArtifacts.map((artifact) => (
            <div
              key={artifact.artifact_path}
              className={`p-4 transition-colors ${
                selectedArtifacts.has(artifact.artifact_path)
                  ? "bg-primary-500/10"
                  : "hover:bg-dark-border/30"
              }`}
            >
              <div className="flex items-center gap-4">
                <button
                  onClick={() => toggleSelect(artifact.artifact_path)}
                  className={`w-5 h-5 rounded border-2 flex items-center justify-center flex-shrink-0 ${
                    selectedArtifacts.has(artifact.artifact_path)
                      ? "border-primary-500 bg-primary-500"
                      : "border-gray-500"
                  }`}
                >
                  {selectedArtifacts.has(artifact.artifact_path) && (
                    <CheckCircle2 size={14} className="text-white" />
                  )}
                </button>

                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <p className="font-medium truncate">{artifact.project_name}</p>
                    <span
                      className={`px-2 py-0.5 rounded text-xs ${
                        artifact.artifact_type === "node_modules"
                          ? "bg-green-500/20 text-green-400"
                          : artifact.artifact_type === "target"
                          ? "bg-orange-500/20 text-orange-400"
                          : artifact.artifact_type === "venv"
                          ? "bg-blue-500/20 text-blue-400"
                          : "bg-purple-500/20 text-purple-400"
                      }`}
                    >
                      {artifact.artifact_type}
                    </span>
                    {artifact.is_recent && (
                      <span className="px-2 py-0.5 rounded text-xs bg-yellow-500/20 text-yellow-400">
                        Reciente
                      </span>
                    )}
                  </div>
                  <p className="text-xs text-gray-400 truncate font-mono mt-1">
                    {artifact.artifact_path}
                  </p>
                </div>

                <div className="text-right">
                  <p className="font-bold text-primary-400">
                    {formatSize(artifact.size)}
                  </p>
                  <p className="text-xs text-gray-400">
                    {formatDate(artifact.last_modified)}
                  </p>
                </div>
              </div>
            </div>
          ))}
        </div>

        {filteredArtifacts.length === 0 && (
          <div className="p-8 text-center text-gray-400">
            <FolderOpen size={48} className="mx-auto mb-4 opacity-50" />
            <p>No se encontraron artefactos de desarrollo</p>
          </div>
        )}
      </div>
    </div>
  );
}

export default ProjectsView;
