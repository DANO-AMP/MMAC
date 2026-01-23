import { useState } from "react";
import {
  Trash2,
  HardDrive,
  Activity,
  Globe,
  FolderOpen,
  Settings,
  Sparkles,
  AppWindow,
} from "lucide-react";
import CleaningView from "./views/CleaningView";
import UninstallerView from "./views/UninstallerView";
import AnalyzerView from "./views/AnalyzerView";
import MonitorView from "./views/MonitorView";
import PortScannerView from "./views/PortScannerView";
import ProjectsView from "./views/ProjectsView";
import SettingsView from "./views/SettingsView";

type View =
  | "cleaning"
  | "uninstaller"
  | "analyzer"
  | "monitor"
  | "ports"
  | "projects"
  | "settings";

interface NavItem {
  id: View;
  label: string;
  icon: React.ReactNode;
}

const navItems: NavItem[] = [
  { id: "cleaning", label: "Limpieza", icon: <Sparkles size={20} /> },
  { id: "uninstaller", label: "Desinstalar", icon: <AppWindow size={20} /> },
  { id: "analyzer", label: "Analizador", icon: <HardDrive size={20} /> },
  { id: "monitor", label: "Monitor", icon: <Activity size={20} /> },
  { id: "ports", label: "Puertos", icon: <Globe size={20} /> },
  { id: "projects", label: "Proyectos", icon: <FolderOpen size={20} /> },
];

function App() {
  const [activeView, setActiveView] = useState<View>("cleaning");

  const renderView = () => {
    switch (activeView) {
      case "cleaning":
        return <CleaningView />;
      case "uninstaller":
        return <UninstallerView />;
      case "analyzer":
        return <AnalyzerView />;
      case "monitor":
        return <MonitorView />;
      case "ports":
        return <PortScannerView />;
      case "projects":
        return <ProjectsView />;
      case "settings":
        return <SettingsView />;
      default:
        return <CleaningView />;
    }
  };

  return (
    <div className="flex h-screen bg-dark-bg text-dark-text">
      {/* Sidebar */}
      <aside className="w-56 bg-dark-card border-r border-dark-border flex flex-col">
        {/* Logo */}
        <div className="p-4 border-b border-dark-border">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-primary-500 to-primary-700 flex items-center justify-center">
              <Sparkles className="text-white" size={24} />
            </div>
            <div>
              <h1 className="font-bold text-lg">SysMac</h1>
              <p className="text-xs text-gray-400">v0.1.0</p>
            </div>
          </div>
        </div>

        {/* Navigation */}
        <nav className="flex-1 py-4">
          <ul className="space-y-1 px-2">
            {navItems.map((item) => (
              <li key={item.id}>
                <button
                  onClick={() => setActiveView(item.id)}
                  className={`w-full flex items-center gap-3 px-3 py-2.5 rounded-lg transition-all ${
                    activeView === item.id
                      ? "nav-item-active bg-primary-500/10 text-primary-400"
                      : "text-gray-400 hover:text-white hover:bg-white/5"
                  }`}
                >
                  {item.icon}
                  <span className="font-medium">{item.label}</span>
                </button>
              </li>
            ))}
          </ul>
        </nav>

        {/* Settings button */}
        <div className="p-2 border-t border-dark-border">
          <button
            onClick={() => setActiveView("settings")}
            className={`w-full flex items-center gap-3 px-3 py-2.5 rounded-lg transition-all ${
              activeView === "settings"
                ? "nav-item-active bg-primary-500/10 text-primary-400"
                : "text-gray-400 hover:text-white hover:bg-white/5"
            }`}
          >
            <Settings size={20} />
            <span className="font-medium">Ajustes</span>
          </button>
        </div>
      </aside>

      {/* Main content */}
      <main className="flex-1 overflow-auto">{renderView()}</main>
    </div>
  );
}

export default App;
