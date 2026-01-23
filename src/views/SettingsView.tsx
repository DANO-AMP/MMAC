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
} from "lucide-react";

interface SettingSection {
  id: string;
  title: string;
  icon: React.ReactNode;
}

function SettingsView() {
  const [activeSection, setActiveSection] = useState("general");
  const [settings, setSettings] = useState({
    theme: "dark",
    notifications: true,
    autoScan: false,
    protectRecent: true,
    recentDays: 7,
    confirmDelete: true,
    moveToTrash: true,
  });

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

  return (
    <div className="p-6 h-full flex">
      {/* Sidebar */}
      <div className="w-56 pr-6 border-r border-dark-border">
        <h2 className="text-xl font-bold mb-6">Ajustes</h2>
        <nav className="space-y-1">
          {sections.map((section) => (
            <button
              key={section.id}
              onClick={() => setActiveSection(section.id)}
              className={`w-full flex items-center gap-3 px-3 py-2.5 rounded-lg transition-colors ${
                activeSection === section.id
                  ? "bg-primary-500/10 text-primary-400"
                  : "text-gray-400 hover:text-white hover:bg-white/5"
              }`}
            >
              {section.icon}
              <span>{section.title}</span>
            </button>
          ))}
        </nav>
      </div>

      {/* Content */}
      <div className="flex-1 pl-6 overflow-auto">
        {activeSection === "general" && (
          <div>
            <h3 className="text-lg font-semibold mb-6">Configuracion General</h3>

            <div className="space-y-6">
              <div className="bg-dark-card border border-dark-border rounded-xl p-4">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="font-medium">Escaneo automatico</p>
                    <p className="text-sm text-gray-400">
                      Escanear al iniciar la aplicacion
                    </p>
                  </div>
                  <button
                    onClick={() =>
                      setSettings((s) => ({ ...s, autoScan: !s.autoScan }))
                    }
                    className={`w-12 h-6 rounded-full transition-colors ${
                      settings.autoScan ? "bg-primary-500" : "bg-dark-border"
                    }`}
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
                    <p className="font-medium">Mover a Papelera</p>
                    <p className="text-sm text-gray-400">
                      Mover archivos a la papelera en lugar de eliminar permanentemente
                    </p>
                  </div>
                  <button
                    onClick={() =>
                      setSettings((s) => ({ ...s, moveToTrash: !s.moveToTrash }))
                    }
                    className={`w-12 h-6 rounded-full transition-colors ${
                      settings.moveToTrash ? "bg-primary-500" : "bg-dark-border"
                    }`}
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
                    <p className="font-medium">Confirmar eliminacion</p>
                    <p className="text-sm text-gray-400">
                      Pedir confirmacion antes de eliminar archivos
                    </p>
                  </div>
                  <button
                    onClick={() =>
                      setSettings((s) => ({
                        ...s,
                        confirmDelete: !s.confirmDelete,
                      }))
                    }
                    className={`w-12 h-6 rounded-full transition-colors ${
                      settings.confirmDelete ? "bg-primary-500" : "bg-dark-border"
                    }`}
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
                    <p className="font-medium">Proteger proyectos recientes</p>
                    <p className="text-sm text-gray-400">
                      Ocultar proyectos modificados recientemente
                    </p>
                  </div>
                  <button
                    onClick={() =>
                      setSettings((s) => ({
                        ...s,
                        protectRecent: !s.protectRecent,
                      }))
                    }
                    className={`w-12 h-6 rounded-full transition-colors ${
                      settings.protectRecent ? "bg-primary-500" : "bg-dark-border"
                    }`}
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
                    <label className="text-sm text-gray-400">
                      Dias de proteccion
                    </label>
                    <input
                      type="number"
                      value={settings.recentDays}
                      onChange={(e) =>
                        setSettings((s) => ({
                          ...s,
                          recentDays: parseInt(e.target.value) || 7,
                        }))
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
          <div>
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
          <div>
            <h3 className="text-lg font-semibold mb-6">Acerca de SysMac</h3>

            <div className="bg-dark-card border border-dark-border rounded-xl p-6 text-center">
              <div className="w-20 h-20 mx-auto mb-4 rounded-2xl bg-gradient-to-br from-primary-500 to-primary-700 flex items-center justify-center">
                <Settings size={40} className="text-white" />
              </div>
              <h4 className="text-xl font-bold">SysMac</h4>
              <p className="text-gray-400 mt-1">Version 0.1.0</p>

              <div className="mt-6 pt-6 border-t border-dark-border">
                <p className="text-sm text-gray-400">
                  Sistema de optimizacion y monitoreo para macOS
                </p>
                <p className="text-sm text-gray-500 mt-2">
                  Desarrollado con Rust + Tauri + React
                </p>
              </div>

              <div className="mt-6 flex justify-center gap-4">
                <button className="flex items-center gap-2 px-4 py-2 bg-dark-border hover:bg-dark-border/80 rounded-lg transition-colors">
                  <ExternalLink size={16} />
                  <span>GitHub</span>
                </button>
              </div>
            </div>
          </div>
        )}

        {activeSection === "appearance" && (
          <div>
            <h3 className="text-lg font-semibold mb-6">Apariencia</h3>

            <div className="bg-dark-card border border-dark-border rounded-xl p-4">
              <p className="font-medium mb-4">Tema</p>
              <div className="grid grid-cols-2 gap-4">
                <button
                  onClick={() => setSettings((s) => ({ ...s, theme: "dark" }))}
                  className={`p-4 rounded-xl border-2 transition-colors ${
                    settings.theme === "dark"
                      ? "border-primary-500 bg-primary-500/10"
                      : "border-dark-border hover:border-dark-border/80"
                  }`}
                >
                  <div className="w-full h-20 bg-dark-bg rounded-lg mb-3" />
                  <p className="font-medium">Oscuro</p>
                </button>
                <button
                  onClick={() => setSettings((s) => ({ ...s, theme: "light" }))}
                  className={`p-4 rounded-xl border-2 transition-colors ${
                    settings.theme === "light"
                      ? "border-primary-500 bg-primary-500/10"
                      : "border-dark-border hover:border-dark-border/80"
                  }`}
                >
                  <div className="w-full h-20 bg-gray-200 rounded-lg mb-3" />
                  <p className="font-medium">Claro</p>
                </button>
              </div>
            </div>
          </div>
        )}

        {activeSection === "notifications" && (
          <div>
            <h3 className="text-lg font-semibold mb-6">Notificaciones</h3>

            <div className="bg-dark-card border border-dark-border rounded-xl p-4">
              <div className="flex items-center justify-between">
                <div>
                  <p className="font-medium">Activar notificaciones</p>
                  <p className="text-sm text-gray-400">
                    Recibir alertas al completar tareas
                  </p>
                </div>
                <button
                  onClick={() =>
                    setSettings((s) => ({
                      ...s,
                      notifications: !s.notifications,
                    }))
                  }
                  className={`w-12 h-6 rounded-full transition-colors ${
                    settings.notifications ? "bg-primary-500" : "bg-dark-border"
                  }`}
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
          <div>
            <h3 className="text-lg font-semibold mb-6">Lista Blanca</h3>

            <div className="bg-dark-card border border-dark-border rounded-xl p-4">
              <p className="text-gray-400 mb-4">
                Las siguientes rutas estan protegidas y no se eliminaran:
              </p>
              <div className="space-y-2 font-mono text-sm">
                <div className="p-2 bg-dark-bg rounded">/System</div>
                <div className="p-2 bg-dark-bg rounded">/usr</div>
                <div className="p-2 bg-dark-bg rounded">/bin</div>
                <div className="p-2 bg-dark-bg rounded">/sbin</div>
                <div className="p-2 bg-dark-bg rounded">~/Library/Keychains</div>
              </div>

              <button className="mt-4 text-primary-400 text-sm hover:underline">
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
