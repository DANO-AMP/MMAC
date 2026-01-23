import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  AppWindow,
  Search,
  Trash2,
  FolderOpen,
  AlertCircle,
  CheckCircle2,
  RefreshCw,
} from "lucide-react";

interface AppInfo {
  name: string;
  bundle_id: string;
  path: string;
  size: number;
  icon?: string;
  version?: string;
  remnants: RemnantFile[];
  remnants_size: number;
}

interface RemnantFile {
  path: string;
  size: number;
  type: string;
}

function formatSize(bytes: number): string {
  if (!bytes || bytes <= 0 || !isFinite(bytes)) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.min(Math.floor(Math.log(bytes) / Math.log(k)), sizes.length - 1);
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
}

function UninstallerView() {
  const [apps, setApps] = useState<AppInfo[]>([]);
  const [selectedApp, setSelectedApp] = useState<AppInfo | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [isLoading, setIsLoading] = useState(true);
  const [isUninstalling, setIsUninstalling] = useState(false);

  useEffect(() => {
    loadApps();
  }, []);

  const loadApps = async () => {
    setIsLoading(true);
    try {
      const result: AppInfo[] = await invoke("list_installed_apps");
      setApps(result);
    } catch (error) {
      console.error("Failed to load apps:", error);
      // Demo data
      setApps([
        {
          name: "Visual Studio Code",
          bundle_id: "com.microsoft.VSCode",
          path: "/Applications/Visual Studio Code.app",
          size: 350 * 1024 * 1024,
          version: "1.85.0",
          remnants: [
            { path: "~/Library/Application Support/Code", size: 150 * 1024 * 1024, type: "data" },
            { path: "~/Library/Caches/com.microsoft.VSCode", size: 50 * 1024 * 1024, type: "cache" },
            { path: "~/Library/Preferences/com.microsoft.VSCode.plist", size: 12 * 1024, type: "pref" },
          ],
          remnants_size: 200 * 1024 * 1024,
        },
        {
          name: "Slack",
          bundle_id: "com.tinyspeck.slackmacgap",
          path: "/Applications/Slack.app",
          size: 180 * 1024 * 1024,
          version: "4.35.0",
          remnants: [
            { path: "~/Library/Application Support/Slack", size: 80 * 1024 * 1024, type: "data" },
            { path: "~/Library/Caches/com.tinyspeck.slackmacgap", size: 25 * 1024 * 1024, type: "cache" },
          ],
          remnants_size: 105 * 1024 * 1024,
        },
        {
          name: "Discord",
          bundle_id: "com.hnc.Discord",
          path: "/Applications/Discord.app",
          size: 200 * 1024 * 1024,
          version: "0.0.285",
          remnants: [
            { path: "~/Library/Application Support/discord", size: 120 * 1024 * 1024, type: "data" },
            { path: "~/Library/Caches/com.hnc.Discord", size: 45 * 1024 * 1024, type: "cache" },
          ],
          remnants_size: 165 * 1024 * 1024,
        },
        {
          name: "Spotify",
          bundle_id: "com.spotify.client",
          path: "/Applications/Spotify.app",
          size: 150 * 1024 * 1024,
          version: "1.2.25",
          remnants: [
            { path: "~/Library/Application Support/Spotify", size: 200 * 1024 * 1024, type: "data" },
            { path: "~/Library/Caches/com.spotify.client", size: 80 * 1024 * 1024, type: "cache" },
          ],
          remnants_size: 280 * 1024 * 1024,
        },
        {
          name: "Figma",
          bundle_id: "com.figma.Desktop",
          path: "/Applications/Figma.app",
          size: 250 * 1024 * 1024,
          version: "116.14.5",
          remnants: [
            { path: "~/Library/Application Support/Figma", size: 60 * 1024 * 1024, type: "data" },
            { path: "~/Library/Caches/com.figma.Desktop", size: 30 * 1024 * 1024, type: "cache" },
          ],
          remnants_size: 90 * 1024 * 1024,
        },
      ]);
    }
    setIsLoading(false);
  };

  const handleUninstall = async () => {
    if (!selectedApp) return;

    setIsUninstalling(true);
    try {
      await invoke("uninstall_app", {
        bundleId: selectedApp.bundle_id,
        includRemnants: true,
      });
      setApps((prev) => prev.filter((a) => a.bundle_id !== selectedApp.bundle_id));
      setSelectedApp(null);
    } catch (error) {
      console.error("Uninstall error:", error);
    }
    setIsUninstalling(false);
  };

  const filteredApps = apps.filter((app) =>
    app.name.toLowerCase().includes(searchQuery.toLowerCase())
  );

  const remnantTypeLabels: Record<string, string> = {
    data: "Datos de App",
    cache: "Cache",
    pref: "Preferencias",
    container: "Container",
    support: "Soporte",
  };

  return (
    <div className="p-6 h-full flex flex-col">
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <h2 className="text-2xl font-bold">Desinstalador</h2>
          <p className="text-gray-400 mt-1">
            Elimina apps y todos sus archivos residuales
          </p>
        </div>
        <button
          onClick={loadApps}
          disabled={isLoading}
          className="flex items-center gap-2 px-4 py-2 bg-dark-card border border-dark-border rounded-lg hover:bg-dark-border transition-colors disabled:opacity-50"
        >
          <RefreshCw size={18} className={isLoading ? "animate-spin" : ""} />
          <span>Actualizar</span>
        </button>
      </div>

      {/* Search */}
      <div className="relative mb-4">
        <Search
          size={18}
          className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400"
        />
        <input
          type="text"
          placeholder="Buscar aplicaciones..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="w-full pl-10 pr-4 py-2.5 bg-dark-card border border-dark-border rounded-lg focus:outline-none focus:border-primary-500 transition-colors"
        />
      </div>

      <div className="flex-1 flex gap-6 min-h-0">
        {/* Apps list */}
        <div className="flex-1 bg-dark-card border border-dark-border rounded-xl overflow-hidden flex flex-col">
          <div className="px-4 py-3 border-b border-dark-border text-sm text-gray-400">
            {filteredApps.length} aplicaciones
          </div>
          <div className="flex-1 overflow-auto">
            {isLoading ? (
              <div className="flex items-center justify-center h-full">
                <RefreshCw size={32} className="animate-spin text-primary-400" />
              </div>
            ) : (
              <div className="divide-y divide-dark-border/50">
                {filteredApps.map((app) => (
                  <div
                    key={app.bundle_id}
                    onClick={() => setSelectedApp(app)}
                    className={`p-4 cursor-pointer transition-colors ${
                      selectedApp?.bundle_id === app.bundle_id
                        ? "bg-primary-500/10"
                        : "hover:bg-dark-border/30"
                    }`}
                  >
                    <div className="flex items-center gap-3">
                      <div className="w-10 h-10 bg-dark-border rounded-xl flex items-center justify-center">
                        <AppWindow size={24} className="text-gray-400" />
                      </div>
                      <div className="flex-1 min-w-0">
                        <p className="font-medium truncate">{app.name}</p>
                        <p className="text-sm text-gray-400">
                          {formatSize(app.size + app.remnants_size)}
                        </p>
                      </div>
                      {app.remnants_size > 0 && (
                        <div className="text-xs text-yellow-400 bg-yellow-500/20 px-2 py-1 rounded">
                          +{formatSize(app.remnants_size)} residuos
                        </div>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>

        {/* Details panel */}
        {selectedApp && (
          <div className="w-96">
            <div className="bg-dark-card border border-dark-border rounded-xl p-5 sticky top-6">
              {/* App header */}
              <div className="flex items-center gap-4 mb-6">
                <div className="w-16 h-16 bg-dark-border rounded-2xl flex items-center justify-center">
                  <AppWindow size={32} className="text-gray-400" />
                </div>
                <div>
                  <h3 className="font-bold text-lg">{selectedApp.name}</h3>
                  <p className="text-sm text-gray-400">v{selectedApp.version}</p>
                </div>
              </div>

              {/* Size info */}
              <div className="grid grid-cols-2 gap-4 mb-6">
                <div className="bg-dark-bg rounded-lg p-3">
                  <p className="text-xs text-gray-400">Aplicacion</p>
                  <p className="text-lg font-bold">{formatSize(selectedApp.size)}</p>
                </div>
                <div className="bg-dark-bg rounded-lg p-3">
                  <p className="text-xs text-gray-400">Residuos</p>
                  <p className="text-lg font-bold text-yellow-400">
                    {formatSize(selectedApp.remnants_size)}
                  </p>
                </div>
              </div>

              {/* Remnants list */}
              {selectedApp.remnants.length > 0 && (
                <div className="mb-6">
                  <p className="text-sm font-medium mb-3 flex items-center gap-2">
                    <FolderOpen size={16} />
                    Archivos residuales ({selectedApp.remnants.length})
                  </p>
                  <div className="space-y-2 max-h-48 overflow-auto">
                    {selectedApp.remnants.map((remnant, idx) => (
                      <div
                        key={idx}
                        className="bg-dark-bg rounded-lg p-3 text-sm"
                      >
                        <div className="flex items-center justify-between mb-1">
                          <span className="text-xs text-primary-400">
                            {remnantTypeLabels[remnant.type] || remnant.type}
                          </span>
                          <span className="text-xs text-gray-400">
                            {formatSize(remnant.size)}
                          </span>
                        </div>
                        <p className="text-xs text-gray-400 truncate font-mono">
                          {remnant.path}
                        </p>
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {/* Info */}
              <div className="flex items-start gap-2 p-3 bg-blue-500/10 border border-blue-500/20 rounded-lg mb-4 text-sm">
                <AlertCircle size={16} className="text-blue-400 mt-0.5 flex-shrink-0" />
                <p className="text-blue-300">
                  La app y sus residuos se moveran a la Papelera
                </p>
              </div>

              {/* Actions */}
              <button
                onClick={handleUninstall}
                disabled={isUninstalling}
                className="w-full flex items-center justify-center gap-2 px-4 py-3 bg-red-500/20 hover:bg-red-500/30 text-red-400 border border-red-500/30 rounded-lg transition-colors disabled:opacity-50"
              >
                {isUninstalling ? (
                  <>
                    <RefreshCw size={18} className="animate-spin" />
                    <span>Desinstalando...</span>
                  </>
                ) : (
                  <>
                    <Trash2 size={18} />
                    <span>Desinstalar Completamente</span>
                  </>
                )}
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

export default UninstallerView;
