import { useState } from "react";
import {
  HardDrive,
  Activity,
  Globe,
  FolderOpen,
  Settings,
  Sparkles,
  AppWindow,
  Rocket,
  Copy,
  FileBox,
  Battery,
  Network,
} from "lucide-react";
import CleaningView from "./views/CleaningView";
import UninstallerView from "./views/UninstallerView";
import AnalyzerView from "./views/AnalyzerView";
import MonitorView from "./views/MonitorView";
import PortScannerView from "./views/PortScannerView";
import ProjectsView from "./views/ProjectsView";
import SettingsView from "./views/SettingsView";
import StartupView from "./views/StartupView";
import DuplicatesView from "./views/DuplicatesView";
import LargeFilesView from "./views/LargeFilesView";
import BatteryView from "./views/BatteryView";
import ConnectionsView from "./views/ConnectionsView";
import { ErrorBoundary } from "./components/ErrorBoundary";
import { APP_VERSION } from "./constants/app";

type View =
  | "cleaning"
  | "uninstaller"
  | "analyzer"
  | "monitor"
  | "ports"
  | "projects"
  | "settings"
  | "startup"
  | "duplicates"
  | "largefiles"
  | "battery"
  | "connections";

interface NavItem {
  id: View;
  label: string;
  icon: React.ReactNode;
}

interface NavSection {
  title: string;
  items: NavItem[];
}

const navSections: NavSection[] = [
  {
    title: "Sistema",
    items: [
      { id: "cleaning", label: "Limpieza", icon: <Sparkles size={18} /> },
      { id: "uninstaller", label: "Desinstalar", icon: <AppWindow size={18} /> },
      { id: "startup", label: "Inicio", icon: <Rocket size={18} /> },
      { id: "monitor", label: "Monitor", icon: <Activity size={18} /> },
      { id: "battery", label: "Batería", icon: <Battery size={18} /> },
    ],
  },
  {
    title: "Archivos",
    items: [
      { id: "analyzer", label: "Analizador", icon: <HardDrive size={18} /> },
      { id: "largefiles", label: "Grandes", icon: <FileBox size={18} /> },
      { id: "duplicates", label: "Duplicados", icon: <Copy size={18} /> },
      { id: "projects", label: "Proyectos", icon: <FolderOpen size={18} /> },
    ],
  },
  {
    title: "Red",
    items: [
      { id: "ports", label: "Puertos", icon: <Globe size={18} /> },
      { id: "connections", label: "Conexiones", icon: <Network size={18} /> },
    ],
  },
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
      case "startup":
        return <StartupView />;
      case "duplicates":
        return <DuplicatesView />;
      case "largefiles":
        return <LargeFilesView />;
      case "battery":
        return <BatteryView />;
      case "connections":
        return <ConnectionsView />;
      default:
        return <CleaningView />;
    }
  };

  return (
    <div className="flex h-screen bg-dark-bg text-dark-text">
      {/* Sidebar */}
      <aside className="w-52 bg-dark-card border-r border-dark-border flex flex-col">
        {/* Drag region for window */}
        <div className="h-8 flex-shrink-0" data-tauri-drag-region />

        {/* Logo */}
        <div className="px-3 pb-3 border-b border-dark-border">
          <div className="flex items-center gap-2">
            <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-primary-500 to-primary-700 flex items-center justify-center">
              <Sparkles className="text-white" size={18} />
            </div>
            <div>
              <h1 className="font-bold text-sm">SysMac</h1>
              <p className="text-xs text-gray-500">v{APP_VERSION}</p>
            </div>
          </div>
        </div>

        {/* Navigation */}
        <nav className="flex-1 py-2 overflow-y-auto">
          {navSections.map((section) => (
            <div key={section.title} className="mb-3">
              <h3 className="px-3 py-1 text-xs font-semibold text-gray-500 uppercase tracking-wider">
                {section.title}
              </h3>
              <ul className="space-y-0.5 px-2">
                {section.items.map((item) => (
                  <li key={item.id}>
                    <button
                      onClick={() => setActiveView(item.id)}
                      className={`w-full flex items-center gap-2 px-2 py-1.5 rounded-lg transition-all text-sm ${
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
            </div>
          ))}
        </nav>

        {/* Settings button */}
        <div className="p-2 border-t border-dark-border">
          <button
            onClick={() => setActiveView("settings")}
            className={`w-full flex items-center gap-2 px-2 py-1.5 rounded-lg transition-all text-sm ${
              activeView === "settings"
                ? "nav-item-active bg-primary-500/10 text-primary-400"
                : "text-gray-400 hover:text-white hover:bg-white/5"
            }`}
          >
            <Settings size={18} />
            <span className="font-medium">Ajustes</span>
          </button>
        </div>
      </aside>

      {/* Main content */}
      <main className="flex-1 overflow-auto flex flex-col">
        {/* Drag region for window */}
        <div className="h-8 flex-shrink-0 w-full" data-tauri-drag-region />
        <div className="flex-1 overflow-auto">
          <ErrorBoundary>{renderView()}</ErrorBoundary>
        </div>
      </main>
    </div>
  );
}

export default App;
