import { useState, lazy, Suspense } from "react";
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
  Cpu,
  Beer,
  Bluetooth,
  FileQuestion,
  Settings2,
  Shield,
  Loader2,
} from "lucide-react";
import { AppProvider } from "./store/AppStore";
import { ErrorBoundary } from "./components/ErrorBoundary";
import { APP_VERSION } from "./constants/app";

// Lazy load views for better initial load
const CleaningView = lazy(() => import("./views/CleaningView"));
const UninstallerView = lazy(() => import("./views/UninstallerView"));
const AnalyzerView = lazy(() => import("./views/AnalyzerView"));
const MonitorView = lazy(() => import("./views/MonitorView"));
const PortScannerView = lazy(() => import("./views/PortScannerView"));
const ProjectsView = lazy(() => import("./views/ProjectsView"));
const SettingsView = lazy(() => import("./views/SettingsView"));
const StartupView = lazy(() => import("./views/StartupView"));
const DuplicatesView = lazy(() => import("./views/DuplicatesView"));
const LargeFilesView = lazy(() => import("./views/LargeFilesView"));
const BatteryView = lazy(() => import("./views/BatteryView"));
const ConnectionsView = lazy(() => import("./views/ConnectionsView"));
const ProcessesView = lazy(() => import("./views/ProcessesView"));
const HomebrewView = lazy(() => import("./views/HomebrewView"));
const BluetoothView = lazy(() => import("./views/BluetoothView"));
const OrphanedView = lazy(() => import("./views/OrphanedView"));
const ServicesView = lazy(() => import("./views/ServicesView"));
const FirewallView = lazy(() => import("./views/FirewallView"));

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
  | "connections"
  | "processes"
  | "homebrew"
  | "bluetooth"
  | "orphaned"
  | "services"
  | "firewall";

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
      { id: "processes", label: "Procesos", icon: <Cpu size={18} /> },
      { id: "battery", label: "Batería", icon: <Battery size={18} /> },
      { id: "bluetooth", label: "Bluetooth", icon: <Bluetooth size={18} /> },
      { id: "services", label: "Servicios", icon: <Settings2 size={18} /> },
    ],
  },
  {
    title: "Archivos",
    items: [
      { id: "analyzer", label: "Analizador", icon: <HardDrive size={18} /> },
      { id: "largefiles", label: "Grandes", icon: <FileBox size={18} /> },
      { id: "duplicates", label: "Duplicados", icon: <Copy size={18} /> },
      { id: "projects", label: "Proyectos", icon: <FolderOpen size={18} /> },
      { id: "orphaned", label: "Huerfanos", icon: <FileQuestion size={18} /> },
    ],
  },
  {
    title: "Paquetes",
    items: [
      { id: "homebrew", label: "Homebrew", icon: <Beer size={18} /> },
    ],
  },
  {
    title: "Red",
    items: [
      { id: "ports", label: "Puertos", icon: <Globe size={18} /> },
      { id: "connections", label: "Conexiones", icon: <Network size={18} /> },
      { id: "firewall", label: "Firewall", icon: <Shield size={18} /> },
    ],
  },
];

// Loading fallback component
function ViewLoader() {
  return (
    <div className="flex items-center justify-center h-full">
      <Loader2 className="w-8 h-8 animate-spin text-primary-500" />
    </div>
  );
}

// Persistent view wrapper - keeps view mounted but hidden
function PersistentView({ isActive, children }: { isActive: boolean; children: React.ReactNode }) {
  return (
    <div
      className={`h-full ${isActive ? "block" : "hidden"}`}
      style={{ contain: "content" }}
    >
      {children}
    </div>
  );
}

// Views that should stay mounted for instant switching
const PERSISTENT_VIEWS: View[] = ["monitor", "processes", "connections", "ports", "battery", "bluetooth"];

function App() {
  const [activeView, setActiveView] = useState<View>("monitor");
  const [mountedViews, setMountedViews] = useState<Set<View>>(new Set(["monitor"]));

  // Track which views have been visited to mount them
  const handleViewChange = (view: View) => {
    setActiveView(view);
    if (!mountedViews.has(view)) {
      setMountedViews(prev => new Set([...prev, view]));
    }
  };

  // Render a view only if it's been visited or is persistent
  const shouldRenderView = (view: View) => {
    return mountedViews.has(view) || PERSISTENT_VIEWS.includes(view);
  };

  const renderView = (view: View) => {
    switch (view) {
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
      case "processes":
        return <ProcessesView />;
      case "homebrew":
        return <HomebrewView />;
      case "bluetooth":
        return <BluetoothView />;
      case "orphaned":
        return <OrphanedView />;
      case "services":
        return <ServicesView />;
      case "firewall":
        return <FirewallView />;
      default:
        return null;
    }
  };

  // All view IDs for persistent rendering
  const allViews: View[] = [
    "cleaning", "uninstaller", "analyzer", "monitor", "ports",
    "projects", "settings", "startup", "duplicates", "largefiles",
    "battery", "connections", "processes", "homebrew", "bluetooth",
    "orphaned", "services", "firewall"
  ];

  return (
    <AppProvider>
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
                        onClick={() => handleViewChange(item.id)}
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
              onClick={() => handleViewChange("settings")}
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
        <main className="flex-1 overflow-hidden flex flex-col">
          {/* Drag region for window */}
          <div className="h-8 flex-shrink-0 w-full" data-tauri-drag-region />
          <div className="flex-1 overflow-auto">
            <ErrorBoundary>
              <Suspense fallback={<ViewLoader />}>
                {/* Render persistent views (always mounted once visited) */}
                {allViews.map((view) => (
                  shouldRenderView(view) && (
                    <PersistentView key={view} isActive={activeView === view}>
                      {renderView(view)}
                    </PersistentView>
                  )
                ))}
              </Suspense>
            </ErrorBoundary>
          </div>
        </main>
      </div>
    </AppProvider>
  );
}

export default App;
