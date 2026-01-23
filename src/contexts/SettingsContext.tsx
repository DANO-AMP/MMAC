import { createContext, useContext, useEffect, useState, useRef, ReactNode } from "react";
import { Store } from "@tauri-apps/plugin-store";

export interface AppSettings {
  theme: "dark" | "light";
  notifications: boolean;
  autoScan: boolean;
  protectRecent: boolean;
  recentDays: number;
  confirmDelete: boolean;
  moveToTrash: boolean;
}

const defaultSettings: AppSettings = {
  theme: "dark",
  notifications: true,
  autoScan: false,
  protectRecent: true,
  recentDays: 7,
  confirmDelete: true,
  moveToTrash: true,
};

interface SettingsContextType {
  settings: AppSettings;
  updateSettings: (updates: Partial<AppSettings>) => Promise<void>;
  isLoading: boolean;
}

const SettingsContext = createContext<SettingsContextType | undefined>(undefined);

const STORE_FILE = "settings.json";

export function SettingsProvider({ children }: { children: ReactNode }) {
  const [settings, setSettings] = useState<AppSettings>(defaultSettings);
  const [isLoading, setIsLoading] = useState(true);
  const storeRef = useRef<Store | null>(null);

  // Load settings on mount
  useEffect(() => {
    const loadSettings = async () => {
      try {
        const store = await Store.load(STORE_FILE, {
          defaults: defaultSettings as unknown as Record<string, unknown>,
          autoSave: true,
        });
        storeRef.current = store;

        // Load each setting individually to preserve defaults for missing keys
        const loaded: Partial<AppSettings> = {};

        for (const key of Object.keys(defaultSettings) as (keyof AppSettings)[]) {
          const value = await store.get<AppSettings[typeof key]>(key);
          if (value !== undefined && value !== null) {
            (loaded as Record<string, unknown>)[key] = value;
          }
        }

        setSettings({ ...defaultSettings, ...loaded });
      } catch (error) {
        console.error("Failed to load settings:", error);
        // Use defaults on error
      } finally {
        setIsLoading(false);
      }
    };

    loadSettings();
  }, []);

  // Apply theme to document
  useEffect(() => {
    const root = document.documentElement;
    if (settings.theme === "dark") {
      root.classList.add("dark");
      root.classList.remove("light");
    } else {
      root.classList.add("light");
      root.classList.remove("dark");
    }
  }, [settings.theme]);

  const updateSettings = async (updates: Partial<AppSettings>) => {
    const newSettings = { ...settings, ...updates };
    setSettings(newSettings);

    try {
      let store = storeRef.current;
      if (!store) {
        store = await Store.load(STORE_FILE, {
          defaults: defaultSettings as unknown as Record<string, unknown>,
          autoSave: true,
        });
        storeRef.current = store;
      }

      // Save each updated key
      for (const [key, value] of Object.entries(updates)) {
        await store.set(key, value);
      }

      await store.save();
    } catch (error) {
      console.error("Failed to save settings:", error);
      // Revert on error
      setSettings(settings);
      throw error;
    }
  };

  return (
    <SettingsContext.Provider value={{ settings, updateSettings, isLoading }}>
      {children}
    </SettingsContext.Provider>
  );
}

export function useSettings() {
  const context = useContext(SettingsContext);
  if (context === undefined) {
    throw new Error("useSettings must be used within a SettingsProvider");
  }
  return context;
}

export default SettingsContext;
