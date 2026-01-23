import { useState } from "react";
import {
  Settings,
  Shield,
  Bell,
  Palette,
  FolderOpen,
  Info,
  ExternalLink,
  CheckCircle2,
  RefreshCw,
} from "lucide-react";
import { APP_VERSION, APP_NAME } from "../constants/app";
import { useSettings, AppSettings } from "../contexts/SettingsContext";

interface SettingSection {
  id: string;
  title: string;
  icon: React.ReactNode;
}

function SettingsView() {
  const { settings, updateSettings, isLoading } = useSettings();
  const [activeSection, setActiveSection] = useState("general");

  const sections: SettingSection[] = [
    { id: "general", title: "General", icon: <Settings size={18} /> },
    { id: "permissions", title: "Permisos", icon: <Shield size={18} /> },
    { id: "notifications", title: "Notificaciones", icon: <Bell size={18} /> },
    { id: "appearance", title: "Apariencia", icon: <Palette size={18} /> },
    { id: "whitelist", title: "Lista Blanca", icon: <FolderOpen size={18} /> },
    { id: "about", title: "Acerca de", icon: <Info size={18} /> },
  ];

  const [permissions] = useState({
    fullDiskAccess: true,
    notifications: true,
  });

  const handleToggle = (key: keyof AppSettings, currentValue: boolean) => {
    updateSettings({ [key]: !currentValue });
  };

  const handleNumberChange = (key: keyof AppSettings, value: number) => {
    updateSettings({ [key]: value });
  };

  if (isLoading) {
    return (
      <div className="p-6 h-full flex items-center justify-center">
        <RefreshCw size={32} className="animate-spin text-primary-400" />
      </div>
    );
  }

  return (
    <div className="p-6 h-full flex">
      {/* Sidebar */}
      <div className="w-56 pr-6 border-r border-dark-border">
        <h2 className="text-xl font-bold mb-6">Ajustes</h2>
        <nav className="space-y-1" role="tablist" aria-label="Secciones de configuracion">
          {sections.map((section) => (
            <button
              key={section.id}
              onClick={() => setActiveSection(section.id)}
              className={`w-full flex items-center gap-3 px-3 py-2.5 rounded-lg transition-colors ${
                activeSection === section.id
                  ? "bg-primary-500/10 text-primary-400"
                  : "text-gray-400 hover:text-white hover:bg-white/5"
              }`}
              role="tab"
              aria-selected={activeSection === section.id}
              aria-controls={`${section.id}-panel`}
            >
              <span aria-hidden="true">{section.icon}</span>
              <span>{section.title}</span>
            </button>
          ))}
        </nav>
      </div>

      {/* Content */}
      <div className="flex-1 pl-6 overflow-auto">
        {activeSection === "general" && (
          <div role="tabpanel" id="general-panel" aria-labelledby="general-tab">
            <h3 className="text-lg font-semibold mb-6">Configuracion General</h3>

            <div className="space-y-6">
              <div className="bg-dark-card border border-dark-border rounded-xl p-4">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="font-medium" id="auto-scan-label">Escaneo automatico</p>
                    <p className="text-sm text-gray-400">
                      Escanear al iniciar la aplicacion
                    </p>
                  </div>
                  <button
                    onClick={() => handleToggle("autoScan", settings.autoScan)}
                    className={`w-12 h-6 rounded-full transition-colors ${
                      settings.autoScan ? "bg-primary-500" : "bg-dark-border"
                    }`}
                    role="switch"
                    aria-checked={settings.autoScan}
                    aria-labelledby="auto-scan-label"
                  >
                    <div
                      className={`w-5 h-5 bg-white rounded-full transition-transform ${
                        settings.autoScan ? "translate-x-6" : "translate-x-0.5"
                      }`}
                    />
                  </button>
                </div>
              </div>

              <div className="bg-dark-card border border-dark-border rounded-xl p-4">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="font-medium" id="move-trash-label">Mover a Papelera</p>
                    <p className="text-sm text-gray-400">
                      Mover archivos a la papelera en lugar de eliminar permanentemente
                    </p>
                  </div>
                  <button
                    onClick={() => handleToggle("moveToTrash", settings.moveToTrash)}
                    className={`w-12 h-6 rounded-full transition-colors ${
                      settings.moveToTrash ? "bg-primary-500" : "bg-dark-border"
                    }`}
                    role="switch"
                    aria-checked={settings.moveToTrash}
                    aria-labelledby="move-trash-label"
                  >
                    <div
                      className={`w-5 h-5 bg-white rounded-full transition-transform ${
                        settings.moveToTrash ? "translate-x-6" : "translate-x-0.5"
                      }`}
                    />
                  </button>
                </div>
              </div>

              <div className="bg-dark-card border border-dark-border rounded-xl p-4">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="font-medium" id="confirm-delete-label">Confirmar eliminacion</p>
                    <p className="text-sm text-gray-400">
                      Pedir confirmacion antes de eliminar archivos
                    </p>
                  </div>
                  <button
                    onClick={() => handleToggle("confirmDelete", settings.confirmDelete)}
                    className={`w-12 h-6 rounded-full transition-colors ${
                      settings.confirmDelete ? "bg-primary-500" : "bg-dark-border"
                    }`}
                    role="switch"
                    aria-checked={settings.confirmDelete}
                    aria-labelledby="confirm-delete-label"
                  >
                    <div
                      className={`w-5 h-5 bg-white rounded-full transition-transform ${
                        settings.confirmDelete
                          ? "translate-x-6"
                          : "translate-x-0.5"
                      }`}
                    />
                  </button>
                </div>
              </div>

              <div className="bg-dark-card border border-dark-border rounded-xl p-4">
                <div className="flex items-center justify-between mb-4">
                  <div>
                    <p className="font-medium" id="protect-recent-label">Proteger proyectos recientes</p>
                    <p className="text-sm text-gray-400">
                      Ocultar proyectos modificados recientemente
                    </p>
                  </div>
                  <button
                    onClick={() => handleToggle("protectRecent", settings.protectRecent)}
                    className={`w-12 h-6 rounded-full transition-colors ${
                      settings.protectRecent ? "bg-primary-500" : "bg-dark-border"
                    }`}
                    role="switch"
                    aria-checked={settings.protectRecent}
                    aria-labelledby="protect-recent-label"
                  >
                    <div
                      className={`w-5 h-5 bg-white rounded-full transition-transform ${
                        settings.protectRecent
                          ? "translate-x-6"
                          : "translate-x-0.5"
                      }`}
                    />
                  </button>
                </div>
                {settings.protectRecent && (
                  <div>
                    <label htmlFor="recent-days" className="text-sm text-gray-400">
                      Dias de proteccion
                    </label>
                    <input
                      id="recent-days"
                      type="number"
                      value={settings.recentDays}
                      onChange={(e) =>
                        handleNumberChange("recentDays", parseInt(e.target.value) || 7)
                      }
                      className="w-20 mt-2 px-3 py-2 bg-dark-bg border border-dark-border rounded-lg focus:outline-none focus:border-primary-500"
                      min={1}
                      max={30}
                    />
                  </div>
                )}
              </div>
            </div>
          </div>
        )}

        {activeSection === "permissions" && (
          <div role="tabpanel" id="permissions-panel" aria-labelledby="permissions-tab">
            <h3 className="text-lg font-semibold mb-6">Permisos del Sistema</h3>

            <div className="space-y-4">
              <div className="bg-dark-card border border-dark-border rounded-xl p-4">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <div
                      className={`p-2 rounded-lg ${
                        permissions.fullDiskAccess
                          ? "bg-green-500/20"
                          : "bg-red-500/20"
                      }`}
                    >
                      <Shield
                        size={20}
                        className={
                          permissions.fullDiskAccess
                            ? "text-green-400"
                            : "text-red-400"
                        }
                      />
                    </div>
                    <div>
                      <p className="font-medium">Acceso Completo al Disco</p>
                      <p className="text-sm text-gray-400">
                        Requerido para limpiar caches del sistema
                      </p>
                    </div>
                  </div>
                  {permissions.fullDiskAccess ? (
                    <div className="flex items-center gap-2 text-green-400">
                      <CheckCircle2 size={18} />
                      <span className="text-sm">Concedido</span>
                    </div>
                  ) : (
                    <button className="flex items-center gap-2 px-4 py-2 bg-primary-600 hover:bg-primary-700 text-white rounded-lg transition-colors">
                      <ExternalLink size={16} />
                      <span>Configurar</span>
                    </button>
                  )}
                </div>
              </div>

              <div className="bg-dark-card border border-dark-border rounded-xl p-4">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <div
                      className={`p-2 rounded-lg ${
                        permissions.notifications
                          ? "bg-green-500/20"
                          : "bg-red-500/20"
                      }`}
                    >
                      <Bell
                        size={20}
                        className={
                          permissions.notifications
                            ? "text-green-400"
                            : "text-red-400"
                        }
                      />
                    </div>
                    <div>
                      <p className="font-medium">Notificaciones</p>
                      <p className="text-sm text-gray-400">
                        Para alertas de limpieza completada
                      </p>
                    </div>
                  </div>
                  {permissions.notifications ? (
                    <div className="flex items-center gap-2 text-green-400">
                      <CheckCircle2 size={18} />
                      <span className="text-sm">Concedido</span>
                    </div>
                  ) : (
                    <button className="flex items-center gap-2 px-4 py-2 bg-primary-600 hover:bg-primary-700 text-white rounded-lg transition-colors">
                      <ExternalLink size={16} />
                      <span>Configurar</span>
                    </button>
                  )}
                </div>
              </div>
            </div>

            <div className="mt-6 p-4 bg-blue-500/10 border border-blue-500/30 rounded-xl">
              <p className="text-sm text-blue-300">
                Los permisos se configuran en{" "}
                <strong>Preferencias del Sistema &gt; Privacidad y Seguridad</strong>
              </p>
            </div>
          </div>
        )}

        {activeSection === "about" && (
          <div role="tabpanel" id="about-panel" aria-labelledby="about-tab">
            <h3 className="text-lg font-semibold mb-6">Acerca de SysMac</h3>

            <div className="bg-dark-card border border-dark-border rounded-xl p-6 text-center">
              <div className="w-20 h-20 mx-auto mb-4 rounded-2xl bg-gradient-to-br from-primary-500 to-primary-700 flex items-center justify-center">
                <Settings size={40} className="text-white" aria-hidden="true" />
              </div>
              <h4 className="text-xl font-bold">{APP_NAME}</h4>
              <p className="text-gray-400 mt-1">Version {APP_VERSION}</p>

              <div className="mt-6 pt-6 border-t border-dark-border">
                <p className="text-sm text-gray-400">
                  Sistema de optimizacion y monitoreo para macOS
                </p>
                <p className="text-sm text-gray-500 mt-2">
                  Desarrollado con Rust + Tauri + React
                </p>
              </div>

              <div className="mt-6 flex justify-center gap-4">
                <button className="flex items-center gap-2 px-4 py-2 bg-dark-border hover:bg-dark-border/80 rounded-lg transition-colors" aria-label="Abrir repositorio de GitHub">
                  <ExternalLink size={16} aria-hidden="true" />
                  <span>GitHub</span>
                </button>
              </div>
            </div>
          </div>
        )}

        {activeSection === "appearance" && (
          <div role="tabpanel" id="appearance-panel" aria-labelledby="appearance-tab">
            <h3 className="text-lg font-semibold mb-6">Apariencia</h3>

            <div className="bg-dark-card border border-dark-border rounded-xl p-4">
              <p className="font-medium mb-4" id="theme-label">Tema</p>
              <div className="grid grid-cols-2 gap-4" role="radiogroup" aria-labelledby="theme-label">
                <button
                  onClick={() => updateSettings({ theme: "dark" })}
                  className={`p-4 rounded-xl border-2 transition-colors ${
                    settings.theme === "dark"
                      ? "border-primary-500 bg-primary-500/10"
                      : "border-dark-border hover:border-dark-border/80"
                  }`}
                  role="radio"
                  aria-checked={settings.theme === "dark"}
                  aria-label="Tema oscuro"
                >
                  <div className="w-full h-20 bg-dark-bg rounded-lg mb-3" aria-hidden="true" />
                  <p className="font-medium">Oscuro</p>
                </button>
                <button
                  onClick={() => updateSettings({ theme: "light" })}
                  className={`p-4 rounded-xl border-2 transition-colors ${
                    settings.theme === "light"
                      ? "border-primary-500 bg-primary-500/10"
                      : "border-dark-border hover:border-dark-border/80"
                  }`}
                  role="radio"
                  aria-checked={settings.theme === "light"}
                  aria-label="Tema claro"
                >
                  <div className="w-full h-20 bg-gray-200 rounded-lg mb-3" aria-hidden="true" />
                  <p className="font-medium">Claro</p>
                </button>
              </div>
            </div>
          </div>
        )}

        {activeSection === "notifications" && (
          <div role="tabpanel" id="notifications-panel" aria-labelledby="notifications-tab">
            <h3 className="text-lg font-semibold mb-6">Notificaciones</h3>

            <div className="bg-dark-card border border-dark-border rounded-xl p-4">
              <div className="flex items-center justify-between">
                <div>
                  <p className="font-medium" id="notifications-label">Activar notificaciones</p>
                  <p className="text-sm text-gray-400">
                    Recibir alertas al completar tareas
                  </p>
                </div>
                <button
                  onClick={() => handleToggle("notifications", settings.notifications)}
                  className={`w-12 h-6 rounded-full transition-colors ${
                    settings.notifications ? "bg-primary-500" : "bg-dark-border"
                  }`}
                  role="switch"
                  aria-checked={settings.notifications}
                  aria-labelledby="notifications-label"
                >
                  <div
                    className={`w-5 h-5 bg-white rounded-full transition-transform ${
                      settings.notifications
                        ? "translate-x-6"
                        : "translate-x-0.5"
                    }`}
                  />
                </button>
              </div>
            </div>
          </div>
        )}

        {activeSection === "whitelist" && (
          <div role="tabpanel" id="whitelist-panel" aria-labelledby="whitelist-tab">
            <h3 className="text-lg font-semibold mb-6">Lista Blanca</h3>

            <div className="bg-dark-card border border-dark-border rounded-xl p-4">
              <p className="text-gray-400 mb-4">
                Las siguientes rutas estan protegidas y no se eliminaran:
              </p>
              <div className="space-y-2 font-mono text-sm" role="list" aria-label="Rutas protegidas">
                <div className="p-2 bg-dark-bg rounded" role="listitem">/System</div>
                <div className="p-2 bg-dark-bg rounded" role="listitem">/usr</div>
                <div className="p-2 bg-dark-bg rounded" role="listitem">/bin</div>
                <div className="p-2 bg-dark-bg rounded" role="listitem">/sbin</div>
                <div className="p-2 bg-dark-bg rounded" role="listitem">~/Library/Keychains</div>
              </div>

              <button className="mt-4 text-primary-400 text-sm hover:underline" aria-label="Agregar nueva ruta a la lista blanca">
                + Agregar ruta personalizada
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

export default SettingsView;
