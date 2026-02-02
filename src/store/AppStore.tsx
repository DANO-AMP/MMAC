import React, { createContext, useContext, useReducer, useCallback, useRef, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

// Refresh interval constants
const REFRESH_INTERVAL_REALTIME = 2000;   // 2s for system stats
const REFRESH_INTERVAL_FAST = 3000;       // 3s for processes
const REFRESH_INTERVAL_MEDIUM = 5000;     // 5s for connections/ports
const REFRESH_INTERVAL_SLOW = 10000;      // 10s for battery/bluetooth

// Types for all cacheable data
export interface SystemStats {
  cpu_usage: number;
  memory_used: number;
  memory_total: number;
  disk_used: number;
  disk_total: number;
  network_rx: number;
  network_tx: number;
  cpu_temp: number;
  fan_speed: number | null;
  disk_read_speed: number;
  disk_write_speed: number;
  gpu_name: string | null;
  gpu_vendor: string | null;
}

export interface ProcessInfo {
  pid: number;
  ppid: number;
  name: string;
  cpu_usage: number;
  memory_mb: number;
  memory_percent: number;
  user: string;
  state: string;
  threads: number;
  command: string;
}

export interface PortInfo {
  port: number;
  pid: number;
  process_name: string;
  service_type: string;
  connections: number;
  addresses: string[];
}

export interface ConnectionInfo {
  local_address: string;
  local_port: number;
  remote_address: string;
  remote_port: number;
  state: string;
  pid: number;
  process_name: string;
}

export interface BatteryInfo {
  percentage: number;
  is_charging: boolean;
  time_remaining: string;
  cycle_count: number;
  health: string;
  temperature: number;
  voltage: number;
  amperage: number;
  wattage: number;
  design_capacity: number;
  max_capacity: number;
  current_capacity: number;
  manufacturer: string;
  serial_number: string;
}

export interface BluetoothDevice {
  name: string;
  address: string;
  device_type: string;
  battery_percent: number | null;
  is_connected: boolean;
  is_paired: boolean;
}

export interface BluetoothInfo {
  is_powered: boolean;
  is_discoverable: boolean;
  devices: BluetoothDevice[];
}

// Cache entry with timestamp
interface CacheEntry<T> {
  data: T;
  timestamp: number;
  isLoading: boolean;
  error: string | null;
}

// App state
interface AppState {
  // Real-time data (refresh every 2s)
  systemStats: CacheEntry<SystemStats | null>;
  processes: CacheEntry<ProcessInfo[]>;

  // Semi-real-time data (refresh every 5s)
  connections: CacheEntry<ConnectionInfo[]>;
  ports: CacheEntry<PortInfo[]>;
  battery: CacheEntry<BatteryInfo | null>;
  bluetooth: CacheEntry<BluetoothInfo | null>;

  // On-demand data (manual refresh)
  // These are kept in component state since they require user action

  // Background refresh settings
  isBackgroundRefreshEnabled: boolean;
  lastGlobalUpdate: number;
}

type Action =
  | { type: "SET_SYSTEM_STATS"; payload: SystemStats }
  | { type: "SET_SYSTEM_STATS_LOADING"; payload: boolean }
  | { type: "SET_SYSTEM_STATS_ERROR"; payload: string | null }
  | { type: "SET_PROCESSES"; payload: ProcessInfo[] }
  | { type: "SET_PROCESSES_LOADING"; payload: boolean }
  | { type: "SET_PROCESSES_ERROR"; payload: string | null }
  | { type: "SET_CONNECTIONS"; payload: ConnectionInfo[] }
  | { type: "SET_CONNECTIONS_LOADING"; payload: boolean }
  | { type: "SET_CONNECTIONS_ERROR"; payload: string | null }
  | { type: "SET_PORTS"; payload: PortInfo[] }
  | { type: "SET_PORTS_LOADING"; payload: boolean }
  | { type: "SET_PORTS_ERROR"; payload: string | null }
  | { type: "SET_BATTERY"; payload: BatteryInfo | null }
  | { type: "SET_BATTERY_LOADING"; payload: boolean }
  | { type: "SET_BATTERY_ERROR"; payload: string | null }
  | { type: "SET_BLUETOOTH"; payload: BluetoothInfo | null }
  | { type: "SET_BLUETOOTH_LOADING"; payload: boolean }
  | { type: "SET_BLUETOOTH_ERROR"; payload: string | null }
  | { type: "SET_BACKGROUND_REFRESH"; payload: boolean }
  | { type: "UPDATE_TIMESTAMP" };

const initialCacheEntry = <T,>(data: T): CacheEntry<T> => ({
  data,
  timestamp: 0,
  isLoading: false,
  error: null,
});

const initialState: AppState = {
  systemStats: initialCacheEntry(null),
  processes: initialCacheEntry([]),
  connections: initialCacheEntry([]),
  ports: initialCacheEntry([]),
  battery: initialCacheEntry(null),
  bluetooth: initialCacheEntry(null),
  isBackgroundRefreshEnabled: true,
  lastGlobalUpdate: 0,
};

function reducer(state: AppState, action: Action): AppState {
  const now = Date.now();

  switch (action.type) {
    case "SET_SYSTEM_STATS":
      return {
        ...state,
        systemStats: { ...state.systemStats, data: action.payload, timestamp: now, isLoading: false, error: null },
        lastGlobalUpdate: now,
      };
    case "SET_SYSTEM_STATS_LOADING":
      return { ...state, systemStats: { ...state.systemStats, isLoading: action.payload } };
    case "SET_SYSTEM_STATS_ERROR":
      return { ...state, systemStats: { ...state.systemStats, error: action.payload, isLoading: false } };

    case "SET_PROCESSES":
      return {
        ...state,
        processes: { ...state.processes, data: action.payload, timestamp: now, isLoading: false, error: null },
      };
    case "SET_PROCESSES_LOADING":
      return { ...state, processes: { ...state.processes, isLoading: action.payload } };
    case "SET_PROCESSES_ERROR":
      return { ...state, processes: { ...state.processes, error: action.payload, isLoading: false } };

    case "SET_CONNECTIONS":
      return {
        ...state,
        connections: { ...state.connections, data: action.payload, timestamp: now, isLoading: false, error: null },
      };
    case "SET_CONNECTIONS_LOADING":
      return { ...state, connections: { ...state.connections, isLoading: action.payload } };
    case "SET_CONNECTIONS_ERROR":
      return { ...state, connections: { ...state.connections, error: action.payload, isLoading: false } };

    case "SET_PORTS":
      return {
        ...state,
        ports: { ...state.ports, data: action.payload, timestamp: now, isLoading: false, error: null },
      };
    case "SET_PORTS_LOADING":
      return { ...state, ports: { ...state.ports, isLoading: action.payload } };
    case "SET_PORTS_ERROR":
      return { ...state, ports: { ...state.ports, error: action.payload, isLoading: false } };

    case "SET_BATTERY":
      return {
        ...state,
        battery: { ...state.battery, data: action.payload, timestamp: now, isLoading: false, error: null },
      };
    case "SET_BATTERY_LOADING":
      return { ...state, battery: { ...state.battery, isLoading: action.payload } };
    case "SET_BATTERY_ERROR":
      return { ...state, battery: { ...state.battery, error: action.payload, isLoading: false } };

    case "SET_BLUETOOTH":
      return {
        ...state,
        bluetooth: { ...state.bluetooth, data: action.payload, timestamp: now, isLoading: false, error: null },
      };
    case "SET_BLUETOOTH_LOADING":
      return { ...state, bluetooth: { ...state.bluetooth, isLoading: action.payload } };
    case "SET_BLUETOOTH_ERROR":
      return { ...state, bluetooth: { ...state.bluetooth, error: action.payload, isLoading: false } };

    case "SET_BACKGROUND_REFRESH":
      return { ...state, isBackgroundRefreshEnabled: action.payload };

    case "UPDATE_TIMESTAMP":
      return { ...state, lastGlobalUpdate: now };

    default:
      return state;
  }
}

// Context
interface AppContextValue {
  state: AppState;
  dispatch: React.Dispatch<Action>;

  // Convenience methods
  refreshSystemStats: () => Promise<void>;
  refreshProcesses: () => Promise<void>;
  refreshConnections: () => Promise<void>;
  refreshPorts: () => Promise<void>;
  refreshBattery: () => Promise<void>;
  refreshBluetooth: () => Promise<void>;
  refreshAll: () => Promise<void>;

  setBackgroundRefresh: (enabled: boolean) => void;
}

const AppContext = createContext<AppContextValue | null>(null);

export function AppProvider({ children }: { children: React.ReactNode }) {
  const [state, dispatch] = useReducer(reducer, initialState);
  const intervalsRef = useRef<{ [key: string]: ReturnType<typeof setInterval> }>({});

  // Use refs for latest state to avoid stale closures
  const stateRef = useRef(state);
  useEffect(() => {
    stateRef.current = state;
  }, [state]);

  // Refresh functions - use refs to avoid stale closures
  const refreshSystemStats = useCallback(async () => {
    if (stateRef.current.systemStats.isLoading) return;
    dispatch({ type: "SET_SYSTEM_STATS_LOADING", payload: true });
    try {
      const result = await invoke<SystemStats>("get_system_stats");
      dispatch({ type: "SET_SYSTEM_STATS", payload: result });
    } catch (err) {
      dispatch({ type: "SET_SYSTEM_STATS_ERROR", payload: String(err) });
    }
  }, []); // No deps needed - uses ref

  const refreshProcesses = useCallback(async () => {
    if (stateRef.current.processes.isLoading) return;
    dispatch({ type: "SET_PROCESSES_LOADING", payload: true });
    try {
      const result = await invoke<ProcessInfo[]>("get_all_processes");
      dispatch({ type: "SET_PROCESSES", payload: result });
    } catch (err) {
      dispatch({ type: "SET_PROCESSES_ERROR", payload: String(err) });
    }
  }, []); // No deps needed - uses ref

  const refreshConnections = useCallback(async () => {
    if (stateRef.current.connections.isLoading) return;
    dispatch({ type: "SET_CONNECTIONS_LOADING", payload: true });
    try {
      const result = await invoke<ConnectionInfo[]>("get_active_connections");
      dispatch({ type: "SET_CONNECTIONS", payload: result });
    } catch (err) {
      dispatch({ type: "SET_CONNECTIONS_ERROR", payload: String(err) });
    }
  }, []); // No deps needed - uses ref

  const refreshPorts = useCallback(async () => {
    if (stateRef.current.ports.isLoading) return;
    dispatch({ type: "SET_PORTS_LOADING", payload: true });
    try {
      const result = await invoke<PortInfo[]>("scan_ports");
      dispatch({ type: "SET_PORTS", payload: result });
    } catch (err) {
      dispatch({ type: "SET_PORTS_ERROR", payload: String(err) });
    }
  }, []); // No deps needed - uses ref

  const refreshBattery = useCallback(async () => {
    if (stateRef.current.battery.isLoading) return;
    dispatch({ type: "SET_BATTERY_LOADING", payload: true });
    try {
      const result = await invoke<BatteryInfo>("get_battery_info");
      dispatch({ type: "SET_BATTERY", payload: result });
    } catch (err) {
      dispatch({ type: "SET_BATTERY_ERROR", payload: String(err) });
    }
  }, []); // No deps needed - uses ref

  const refreshBluetooth = useCallback(async () => {
    if (stateRef.current.bluetooth.isLoading) return;
    dispatch({ type: "SET_BLUETOOTH_LOADING", payload: true });
    try {
      const result = await invoke<BluetoothInfo>("get_bluetooth_info");
      dispatch({ type: "SET_BLUETOOTH", payload: result });
    } catch (err) {
      dispatch({ type: "SET_BLUETOOTH_ERROR", payload: String(err) });
    }
  }, []); // No deps needed - uses ref

  const refreshAll = useCallback(async () => {
    await Promise.all([
      refreshSystemStats(),
      refreshProcesses(),
      refreshConnections(),
      refreshPorts(),
      refreshBattery(),
      refreshBluetooth(),
    ]);
  }, [refreshSystemStats, refreshProcesses, refreshConnections, refreshPorts, refreshBattery, refreshBluetooth]);

  const setBackgroundRefresh = useCallback((enabled: boolean) => {
    dispatch({ type: "SET_BACKGROUND_REFRESH", payload: enabled });
  }, []);

  // Keep refs to refresh functions for intervals
  const refreshFnsRef = useRef({
    refreshSystemStats,
    refreshProcesses,
    refreshConnections,
    refreshPorts,
    refreshBattery,
    refreshBluetooth
  });

  useEffect(() => {
    refreshFnsRef.current = {
      refreshSystemStats,
      refreshProcesses,
      refreshConnections,
      refreshPorts,
      refreshBattery,
      refreshBluetooth
    };
  });

  // Background refresh effect
  useEffect(() => {
    if (!state.isBackgroundRefreshEnabled) {
      // Clear all intervals
      Object.values(intervalsRef.current).forEach(clearInterval);
      intervalsRef.current = {};
      return;
    }

    // Initial fetch
    refreshFnsRef.current.refreshSystemStats();
    refreshFnsRef.current.refreshProcesses();
    refreshFnsRef.current.refreshBattery();
    refreshFnsRef.current.refreshBluetooth();

    // Slower initial fetch for network-heavy operations
    setTimeout(() => {
      refreshFnsRef.current.refreshConnections();
      refreshFnsRef.current.refreshPorts();
    }, 500);

    // Set up intervals - real-time data
    intervalsRef.current.systemStats = setInterval(() => refreshFnsRef.current.refreshSystemStats(), REFRESH_INTERVAL_REALTIME);
    intervalsRef.current.processes = setInterval(() => refreshFnsRef.current.refreshProcesses(), REFRESH_INTERVAL_FAST);

    // Semi-real-time data
    intervalsRef.current.connections = setInterval(() => refreshFnsRef.current.refreshConnections(), REFRESH_INTERVAL_MEDIUM);
    intervalsRef.current.ports = setInterval(() => refreshFnsRef.current.refreshPorts(), REFRESH_INTERVAL_MEDIUM);
    intervalsRef.current.battery = setInterval(() => refreshFnsRef.current.refreshBattery(), REFRESH_INTERVAL_SLOW);
    intervalsRef.current.bluetooth = setInterval(() => refreshFnsRef.current.refreshBluetooth(), REFRESH_INTERVAL_SLOW);

    return () => {
      Object.values(intervalsRef.current).forEach(clearInterval);
      intervalsRef.current = {};
    };
  }, [state.isBackgroundRefreshEnabled]); // Now correct - only depends on the toggle

  const value: AppContextValue = {
    state,
    dispatch,
    refreshSystemStats,
    refreshProcesses,
    refreshConnections,
    refreshPorts,
    refreshBattery,
    refreshBluetooth,
    refreshAll,
    setBackgroundRefresh,
  };

  return <AppContext.Provider value={value}>{children}</AppContext.Provider>;
}

export function useAppStore() {
  const context = useContext(AppContext);
  if (!context) {
    throw new Error("useAppStore must be used within AppProvider");
  }
  return context;
}

// Selector hooks for specific data
export function useSystemStats() {
  const { state, refreshSystemStats } = useAppStore();
  return {
    stats: state.systemStats.data,
    isLoading: state.systemStats.isLoading,
    error: state.systemStats.error,
    lastUpdated: state.systemStats.timestamp,
    refresh: refreshSystemStats,
  };
}

export function useProcesses() {
  const { state, refreshProcesses } = useAppStore();
  return {
    processes: state.processes.data,
    isLoading: state.processes.isLoading,
    error: state.processes.error,
    lastUpdated: state.processes.timestamp,
    refresh: refreshProcesses,
  };
}

export function useConnections() {
  const { state, refreshConnections } = useAppStore();
  return {
    connections: state.connections.data,
    isLoading: state.connections.isLoading,
    error: state.connections.error,
    lastUpdated: state.connections.timestamp,
    refresh: refreshConnections,
  };
}

export function usePorts() {
  const { state, refreshPorts } = useAppStore();
  return {
    ports: state.ports.data,
    isLoading: state.ports.isLoading,
    error: state.ports.error,
    lastUpdated: state.ports.timestamp,
    refresh: refreshPorts,
  };
}

export function useBattery() {
  const { state, refreshBattery } = useAppStore();
  return {
    battery: state.battery.data,
    isLoading: state.battery.isLoading,
    error: state.battery.error,
    lastUpdated: state.battery.timestamp,
    refresh: refreshBattery,
  };
}

export function useBluetooth() {
  const { state, refreshBluetooth } = useAppStore();
  return {
    bluetooth: state.bluetooth.data,
    isLoading: state.bluetooth.isLoading,
    error: state.bluetooth.error,
    lastUpdated: state.bluetooth.timestamp,
    refresh: refreshBluetooth,
  };
}
