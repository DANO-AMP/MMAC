import { vi } from "vitest";

type InvokeHandler = (cmd: string, args?: Record<string, unknown>) => unknown;

let invokeHandler: InvokeHandler = () => null;

export function mockInvoke(handler: InvokeHandler) {
  invokeHandler = handler;
}

export function resetInvokeMock() {
  invokeHandler = () => null;
}

// Create a mock invoke function that can be customized per test
export const createInvokeMock = () => {
  return vi.fn().mockImplementation((cmd: string, args?: Record<string, unknown>) => {
    return Promise.resolve(invokeHandler(cmd, args));
  });
};

// Common mock responses for Tauri commands
export const mockResponses = {
  get_memory_info: {
    total: 17179869184,
    used: 8589934592,
    free: 8589934592,
    active: 4294967296,
    inactive: 2147483648,
    wired: 2147483648,
    compressed: 1073741824,
    app_memory: 6442450944,
    cached: 2147483648,
  },
  get_battery_info: {
    is_present: true,
    is_charging: false,
    is_fully_charged: false,
    charge_percent: 75.0,
    cycle_count: 150,
    max_capacity: 4800,
    design_capacity: 5103,
    health_percent: 94.0,
    temperature: 28.5,
    voltage: 12.5,
    amperage: -1500,
    time_remaining: "3:45 restante",
    power_source: "Bateria",
    condition: "Normal",
  },
  scan_ports: [
    {
      port: 3000,
      pid: 12345,
      process_name: "node",
      service_type: "Next.js / React",
      protocol: "TCP",
      local_address: "127.0.0.1",
      working_dir: "/Users/dev/project",
      command: "node server.js",
    },
  ],
  get_system_stats: {
    cpu_usage: 25.5,
    memory_used: 8589934592,
    memory_total: 17179869184,
    disk_read_speed: 1048576,
    disk_write_speed: 524288,
    network_rx_speed: 102400,
    network_tx_speed: 51200,
  },
};
