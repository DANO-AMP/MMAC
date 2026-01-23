import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Rocket, Power, PowerOff, Trash2, RefreshCw } from "lucide-react";

interface StartupItem {
  name: string;
  path: string;
  kind: string;
  enabled: boolean;
}

export default function StartupView() {
  const [items, setItems] = useState<StartupItem[]>([]);
  const [loading, setLoading] = useState(false);

  const loadItems = async () => {
    setLoading(true);
    try {
      const result = await invoke<StartupItem[]>("get_startup_items");
      setItems(result);
    } catch (error) {
      console.error("Error loading startup items:", error);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    let isMounted = true;

    const loadInitialItems = async () => {
      if (!isMounted) return;
      setLoading(true);
      try {
        const result = await invoke<StartupItem[]>("get_startup_items");
        if (isMounted) {
          setItems(result);
        }
      } catch (error) {
        console.error("Error loading startup items:", error);
      } finally {
        if (isMounted) {
          setLoading(false);
        }
      }
    };

    loadInitialItems();
    return () => { isMounted = false; };
  }, []);

  const toggleItem = async (item: StartupItem) => {
    try {
      await invoke("toggle_startup_item", {
        path: item.path,
        enable: !item.enabled,
      });
      loadItems();
    } catch (error) {
      console.error("Error toggling item:", error);
    }
  };

  const removeLoginItem = async (name: string) => {
    try {
      await invoke("remove_login_item", { name });
      loadItems();
    } catch (error) {
      console.error("Error removing login item:", error);
    }
  };

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold flex items-center gap-3">
            <Rocket className="text-primary-400" />
            Gestión de Inicio
          </h2>
          <p className="text-gray-400 mt-1">
            Controla qué aplicaciones se inician con tu Mac
          </p>
        </div>
        <button
          onClick={loadItems}
          disabled={loading}
          className="flex items-center gap-2 px-4 py-2 bg-dark-card border border-dark-border rounded-lg hover:bg-dark-border transition-colors"
          aria-label="Actualizar lista de elementos de inicio"
        >
          <RefreshCw size={18} className={loading ? "animate-spin" : ""} aria-hidden="true" />
          Actualizar
        </button>
      </div>

      {loading && items.length === 0 ? (
        <div className="flex items-center justify-center py-12">
          <RefreshCw className="animate-spin text-primary-400" size={32} />
        </div>
      ) : items.length === 0 ? (
        <div className="text-center py-12 text-gray-400">
          No se encontraron elementos de inicio
        </div>
      ) : (
        <div className="space-y-4">
          {/* Launch Agents */}
          <div className="bg-dark-card rounded-xl border border-dark-border overflow-hidden">
            <div className="px-4 py-3 border-b border-dark-border bg-dark-bg/50">
              <h3 className="font-semibold">Launch Agents</h3>
              <p className="text-sm text-gray-400">
                Servicios que se ejecutan en segundo plano
              </p>
            </div>
            <div className="divide-y divide-dark-border">
              {items
                .filter((i) => i.kind === "LaunchAgent")
                .map((item) => (
                  <div
                    key={item.path}
                    className="flex items-center justify-between p-4 hover:bg-dark-bg/30"
                  >
                    <div className="flex items-center gap-3">
                      <div
                        className={`w-2 h-2 rounded-full ${
                          item.enabled ? "bg-green-500" : "bg-gray-500"
                        }`}
                      />
                      <div>
                        <p className="font-medium">{item.name}</p>
                        <p className="text-sm text-gray-400 truncate max-w-md">
                          {item.path}
                        </p>
                      </div>
                    </div>
                    <button
                      onClick={() => toggleItem(item)}
                      className={`flex items-center gap-2 px-3 py-1.5 rounded-lg transition-colors ${
                        item.enabled
                          ? "bg-red-500/10 text-red-400 hover:bg-red-500/20"
                          : "bg-green-500/10 text-green-400 hover:bg-green-500/20"
                      }`}
                      aria-label={item.enabled ? `Desactivar ${item.name}` : `Activar ${item.name}`}
                      aria-pressed={item.enabled}
                    >
                      {item.enabled ? (
                        <>
                          <PowerOff size={16} aria-hidden="true" /> Desactivar
                        </>
                      ) : (
                        <>
                          <Power size={16} aria-hidden="true" /> Activar
                        </>
                      )}
                    </button>
                  </div>
                ))}
              {items.filter((i) => i.kind === "LaunchAgent").length === 0 && (
                <div className="p-4 text-gray-400 text-center">
                  No hay Launch Agents
                </div>
              )}
            </div>
          </div>

          {/* Login Items */}
          <div className="bg-dark-card rounded-xl border border-dark-border overflow-hidden">
            <div className="px-4 py-3 border-b border-dark-border bg-dark-bg/50">
              <h3 className="font-semibold">Login Items</h3>
              <p className="text-sm text-gray-400">
                Aplicaciones que se abren al iniciar sesión
              </p>
            </div>
            <div className="divide-y divide-dark-border">
              {items
                .filter((i) => i.kind === "LoginItem")
                .map((item) => (
                  <div
                    key={item.name}
                    className="flex items-center justify-between p-4 hover:bg-dark-bg/30"
                  >
                    <div className="flex items-center gap-3">
                      <div className="w-2 h-2 rounded-full bg-green-500" />
                      <p className="font-medium">{item.name}</p>
                    </div>
                    <button
                      onClick={() => removeLoginItem(item.name)}
                      className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-red-500/10 text-red-400 hover:bg-red-500/20 transition-colors"
                      aria-label={`Eliminar ${item.name} de elementos de inicio`}
                    >
                      <Trash2 size={16} aria-hidden="true" /> Eliminar
                    </button>
                  </div>
                ))}
              {items.filter((i) => i.kind === "LoginItem").length === 0 && (
                <div className="p-4 text-gray-400 text-center">
                  No hay Login Items
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
