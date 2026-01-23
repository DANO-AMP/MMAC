import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { FileBox, Search, Trash2, FolderOpen, RefreshCw } from "lucide-react";

interface LargeFile {
  path: string;
  name: string;
  size: number;
  modified: number;
}

function formatSize(bytes: number): string {
  if (!bytes || bytes <= 0 || !isFinite(bytes)) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.min(Math.floor(Math.log(bytes) / Math.log(k)), sizes.length - 1);
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
}

function formatDate(timestamp: number): string {
  if (!timestamp) return "-";
  return new Date(timestamp * 1000).toLocaleDateString("es-ES", {
    year: "numeric",
    month: "short",
    day: "numeric",
  });
}

export default function LargeFilesView() {
  const [scanning, setScanning] = useState(false);
  const [files, setFiles] = useState<LargeFile[]>([]);
  const [path, setPath] = useState("~");
  const [minSize, setMinSize] = useState(100); // MB
  const [limit, setLimit] = useState(100);

  const startScan = async () => {
    setScanning(true);
    setFiles([]);
    try {
      const expandedPath = path.startsWith("~")
        ? path.replace("~", "/Users/me")
        : path;

      const result = await invoke<LargeFile[]>("find_large_files", {
        path: expandedPath,
        minSize: minSize * 1024 * 1024,
        limit,
      });
      setFiles(result);
    } catch (error) {
      console.error("Error scanning:", error);
    } finally {
      setScanning(false);
    }
  };

  const deleteFile = async (filePath: string) => {
    try {
      await invoke("move_to_trash", { path: filePath });
      setFiles(files.filter((f) => f.path !== filePath));
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

  const totalSize = files.reduce((acc, f) => acc + f.size, 0);

  return (
    <div className="p-6 space-y-6">
      <div>
        <h2 className="text-2xl font-bold flex items-center gap-3">
          <FileBox className="text-primary-400" />
          Archivos Grandes
        </h2>
        <p className="text-gray-400 mt-1">
          Encuentra los archivos más grandes en tu disco
        </p>
      </div>

      {/* Scan options */}
      <div className="bg-dark-card rounded-xl border border-dark-border p-4 space-y-4">
        <div className="grid grid-cols-3 gap-4">
          <div>
            <label htmlFor="large-files-path" className="block text-sm text-gray-400 mb-2">
              Carpeta a analizar
            </label>
            <input
              id="large-files-path"
              type="text"
              value={path}
              onChange={(e) => setPath(e.target.value)}
              className="w-full px-4 py-2 bg-dark-bg border border-dark-border rounded-lg focus:outline-none focus:border-primary-500"
              placeholder="~"
            />
          </div>
          <div>
            <label htmlFor="large-files-min-size" className="block text-sm text-gray-400 mb-2">
              Tamano minimo (MB)
            </label>
            <input
              id="large-files-min-size"
              type="number"
              value={minSize}
              onChange={(e) => setMinSize(Number(e.target.value))}
              min={1}
              className="w-full px-4 py-2 bg-dark-bg border border-dark-border rounded-lg focus:outline-none focus:border-primary-500"
            />
          </div>
          <div>
            <label htmlFor="large-files-limit" className="block text-sm text-gray-400 mb-2">
              Limite de resultados
            </label>
            <input
              id="large-files-limit"
              type="number"
              value={limit}
              onChange={(e) => setLimit(Number(e.target.value))}
              min={10}
              max={500}
              className="w-full px-4 py-2 bg-dark-bg border border-dark-border rounded-lg focus:outline-none focus:border-primary-500"
            />
          </div>
        </div>
        <button
          onClick={startScan}
          disabled={scanning}
          className="flex items-center gap-2 px-6 py-2.5 bg-primary-500 hover:bg-primary-600 rounded-lg font-medium transition-colors disabled:opacity-50"
          aria-label="Buscar archivos grandes en el disco"
        >
          {scanning ? (
            <>
              <RefreshCw className="animate-spin" size={18} aria-hidden="true" />
              Escaneando...
            </>
          ) : (
            <>
              <Search size={18} aria-hidden="true" />
              Buscar Archivos Grandes
            </>
          )}
        </button>
      </div>

      {/* Results */}
      {files.length > 0 && (
        <div className="space-y-4">
          {/* Summary */}
          <div className="bg-dark-card rounded-xl border border-dark-border p-4 flex items-center justify-between">
            <div>
              <span className="text-gray-400">
                {files.length} archivos encontrados
              </span>
            </div>
            <div className="text-xl font-bold">{formatSize(totalSize)}</div>
          </div>

          {/* File list */}
          <div className="bg-dark-card rounded-xl border border-dark-border overflow-hidden">
            <table className="w-full" role="table" aria-label="Lista de archivos grandes">
              <thead>
                <tr className="border-b border-dark-border bg-dark-bg/50">
                  <th scope="col" className="text-left px-4 py-3 font-medium">Nombre</th>
                  <th scope="col" className="text-right px-4 py-3 font-medium w-32">
                    Tamano
                  </th>
                  <th scope="col" className="text-right px-4 py-3 font-medium w-32">
                    Modificado
                  </th>
                  <th scope="col" className="w-24"><span className="sr-only">Acciones</span></th>
                </tr>
              </thead>
              <tbody className="divide-y divide-dark-border">
                {files.map((file) => (
                  <tr key={file.path} className="hover:bg-dark-bg/30">
                    <td className="px-4 py-3">
                      <p className="font-medium">{file.name}</p>
                      <p className="text-sm text-gray-400 truncate max-w-lg">
                        {file.path}
                      </p>
                    </td>
                    <td className="px-4 py-3 text-right font-mono">
                      {formatSize(file.size)}
                    </td>
                    <td className="px-4 py-3 text-right text-gray-400">
                      {formatDate(file.modified)}
                    </td>
                    <td className="px-4 py-3">
                      <div className="flex items-center justify-end gap-1">
                        <button
                          onClick={() => revealInFinder(file.path)}
                          className="p-2 hover:bg-dark-border rounded-lg transition-colors"
                          aria-label={`Mostrar ${file.name} en Finder`}
                        >
                          <FolderOpen size={16} aria-hidden="true" />
                        </button>
                        <button
                          onClick={() => deleteFile(file.path)}
                          className="p-2 hover:bg-red-500/20 text-red-400 rounded-lg transition-colors"
                          aria-label={`Mover ${file.name} a Papelera`}
                        >
                          <Trash2 size={16} aria-hidden="true" />
                        </button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}
    </div>
  );
}
