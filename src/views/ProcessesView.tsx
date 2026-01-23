import { useState, useMemo, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Cpu,
  RefreshCw,
  Search,
  Square,
  Zap,
  MemoryStick,
  ArrowUpDown,
  ArrowUp,
  ArrowDown,
  List,
  GitBranch,
  Pause,
  Play,
} from "lucide-react";
import { ErrorBanner } from "../components/ErrorBanner";
import { ConfirmDialog } from "../components/ConfirmDialog";
import { useConfirmation } from "../hooks/useConfirmation";
import { useProcesses, ProcessInfo } from "../store/AppStore";

interface TreeNode extends ProcessInfo {
  children: TreeNode[];
  depth: number;
}

type SortField = "cpu_usage" | "memory_mb" | "name" | "pid";
type SortDirection = "asc" | "desc";

function ProcessesView() {
  const { processes, isLoading, error, refresh } = useProcesses();
  const [searchQuery, setSearchQuery] = useState("");
  const [sortField, setSortField] = useState<SortField>("cpu_usage");
  const [sortDirection, setSortDirection] = useState<SortDirection>("desc");
  const [selectedProcess, setSelectedProcess] = useState<ProcessInfo | null>(null);
  const [viewMode, setViewMode] = useState<"list" | "tree">("list");
  const [expandedPids, setExpandedPids] = useState<Set<number>>(new Set());
  const [actionError, setActionError] = useState<string | null>(null);

  const { confirm, dialogProps } = useConfirmation();

  const sendSignal = async (pid: number, signal: string) => {
    try {
      setActionError(null);
      await invoke("send_process_signal", { pid, signal });
      if (signal === "SIGKILL" || signal === "SIGTERM") {
        setSelectedProcess(null);
      }
      // Refresh will happen automatically via background refresh
    } catch (err) {
      setActionError(err instanceof Error ? err.message : String(err));
    }
  };

  const buildProcessTree = useCallback((procs: ProcessInfo[]): TreeNode[] => {
    const nodeMap = new Map<number, TreeNode>();
    const roots: TreeNode[] = [];

    // Create nodes
    for (const proc of procs) {
      nodeMap.set(proc.pid, { ...proc, children: [], depth: 0 });
    }

    // Build tree
    for (const proc of procs) {
      const node = nodeMap.get(proc.pid)!;
      const parent = nodeMap.get(proc.ppid);
      if (parent) {
        node.depth = parent.depth + 1;
        parent.children.push(node);
      } else {
        roots.push(node);
      }
    }

    return roots;
  }, []);

  const flattenTree = useCallback((nodes: TreeNode[]): TreeNode[] => {
    const result: TreeNode[] = [];
    const traverse = (node: TreeNode) => {
      result.push(node);
      if (expandedPids.has(node.pid)) {
        for (const child of node.children) {
          traverse(child);
        }
      }
    };
    for (const root of nodes) {
      traverse(root);
    }
    return result;
  }, [expandedPids]);

  const toggleExpand = (pid: number) => {
    const newExpanded = new Set(expandedPids);
    if (newExpanded.has(pid)) {
      newExpanded.delete(pid);
    } else {
      newExpanded.add(pid);
    }
    setExpandedPids(newExpanded);
  };

  const handleSort = (field: SortField) => {
    if (sortField === field) {
      setSortDirection(sortDirection === "asc" ? "desc" : "asc");
    } else {
      setSortField(field);
      setSortDirection(field === "name" ? "asc" : "desc");
    }
  };

  const getSortIcon = (field: SortField) => {
    if (sortField !== field) return <ArrowUpDown size={14} className="opacity-30" />;
    return sortDirection === "asc" ? <ArrowUp size={14} /> : <ArrowDown size={14} />;
  };

  const filteredAndSorted = useMemo(() => {
    return processes
      .filter(
        (p) =>
          p.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
          p.command.toLowerCase().includes(searchQuery.toLowerCase()) ||
          p.user.toLowerCase().includes(searchQuery.toLowerCase()) ||
          p.pid.toString().includes(searchQuery)
      )
      .sort((a, b) => {
        const multiplier = sortDirection === "asc" ? 1 : -1;
        if (sortField === "name") {
          return multiplier * a.name.localeCompare(b.name);
        }
        return multiplier * (a[sortField] - b[sortField]);
      });
  }, [processes, searchQuery, sortField, sortDirection]);

  const { totalCpu, totalMemory } = useMemo(() => ({
    totalCpu: processes.reduce((sum, p) => sum + p.cpu_usage, 0),
    totalMemory: processes.reduce((sum, p) => sum + p.memory_mb, 0),
  }), [processes]);

  return (
    <div className="p-6">
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <h2 className="text-2xl font-bold" id="processes-title">Procesos</h2>
          <p className="text-gray-400 mt-1">
            Monitor de actividad del sistema
          </p>
        </div>
        <div className="flex items-center gap-2">
          {isLoading && (
            <div className="flex items-center gap-2 text-sm text-gray-400" role="status" aria-live="polite">
              <div className="w-2 h-2 rounded-full bg-blue-500 animate-pulse" aria-hidden="true" />
              <span>Actualizando...</span>
            </div>
          )}
          <div className="flex bg-dark-card border border-dark-border rounded-lg overflow-hidden" role="group" aria-label="Modo de vista">
            <button
              onClick={() => setViewMode("list")}
              className={`flex items-center gap-1 px-3 py-2 transition-colors ${
                viewMode === "list"
                  ? "bg-primary-500/20 text-primary-400"
                  : "text-gray-400 hover:text-white"
              }`}
              aria-label="Vista de lista"
              aria-pressed={viewMode === "list"}
            >
              <List size={16} aria-hidden="true" />
            </button>
            <button
              onClick={() => setViewMode("tree")}
              className={`flex items-center gap-1 px-3 py-2 transition-colors ${
                viewMode === "tree"
                  ? "bg-primary-500/20 text-primary-400"
                  : "text-gray-400 hover:text-white"
              }`}
              aria-label="Vista de arbol"
              aria-pressed={viewMode === "tree"}
            >
              <GitBranch size={16} aria-hidden="true" />
            </button>
          </div>
          <button
            onClick={refresh}
            disabled={isLoading}
            className="flex items-center gap-2 px-4 py-2 bg-primary-600 hover:bg-primary-700 text-white rounded-lg transition-colors disabled:opacity-50"
            aria-label="Actualizar lista de procesos"
          >
            <RefreshCw size={18} className={isLoading ? "animate-spin" : ""} aria-hidden="true" />
            <span>Actualizar</span>
          </button>
        </div>
      </div>

      {(error || actionError) && (
        <ErrorBanner
          error={error || actionError || ""}
          onRetry={() => {
            setActionError(null);
            refresh();
          }}
          className="mb-6"
        />
      )}

      {/* Summary cards */}
      <div className="grid grid-cols-4 gap-4 mb-6">
        <div className="bg-dark-card border border-dark-border rounded-xl p-4">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-lg bg-blue-500/20 text-blue-400">
              <Cpu size={20} />
            </div>
            <div>
              <p className="text-sm text-gray-400">Procesos</p>
              <p className="text-2xl font-bold">{processes.length}</p>
            </div>
          </div>
        </div>
        <div className="bg-dark-card border border-dark-border rounded-xl p-4">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-lg bg-orange-500/20 text-orange-400">
              <Cpu size={20} />
            </div>
            <div>
              <p className="text-sm text-gray-400">CPU Total</p>
              <p className={`text-2xl font-bold ${totalCpu > 100 ? 'text-red-400' : totalCpu > 50 ? 'text-yellow-400' : 'text-green-400'}`}>
                {totalCpu.toFixed(1)}%
              </p>
            </div>
          </div>
        </div>
        <div className="bg-dark-card border border-dark-border rounded-xl p-4">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-lg bg-purple-500/20 text-purple-400">
              <MemoryStick size={20} />
            </div>
            <div>
              <p className="text-sm text-gray-400">RAM Usada</p>
              <p className="text-2xl font-bold">
                {(totalMemory / 1024).toFixed(1)} GB
              </p>
            </div>
          </div>
        </div>
        <div className="bg-dark-card border border-dark-border rounded-xl p-4">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-lg bg-green-500/20 text-green-400">
              <Zap size={20} />
            </div>
            <div>
              <p className="text-sm text-gray-400">Top CPU</p>
              <p className="text-lg font-bold truncate">
                {processes[0]?.name || "-"}
              </p>
            </div>
          </div>
        </div>
      </div>

      {/* Search */}
      <div className="mb-4">
        <div className="relative">
          <Search
            size={18}
            className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400"
            aria-hidden="true"
          />
          <input
            type="text"
            placeholder="Buscar proceso por nombre, PID, usuario..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full bg-dark-card border border-dark-border rounded-lg pl-10 pr-4 py-2 text-white placeholder-gray-500 focus:outline-none focus:border-primary-500"
            aria-label="Buscar proceso por nombre, PID, usuario"
          />
        </div>
      </div>

      <div className="flex gap-6">
        {/* Processes table */}
        <div className="flex-1">
          <div className="bg-dark-card border border-dark-border rounded-xl overflow-hidden">
            <div className="max-h-[500px] overflow-auto" role="region" aria-label="Lista de procesos" aria-live="polite">
              <table className="w-full" role="table" aria-labelledby="processes-title">
                <thead className="sticky top-0 bg-dark-card z-10">
                  <tr className="border-b border-dark-border">
                    <th
                      scope="col"
                      className="text-left px-4 py-3 text-sm font-medium text-gray-400 cursor-pointer hover:text-white"
                      onClick={() => handleSort("pid")}
                      onKeyDown={(e) => e.key === "Enter" && handleSort("pid")}
                      tabIndex={0}
                      role="columnheader"
                      aria-sort={sortField === "pid" ? (sortDirection === "asc" ? "ascending" : "descending") : "none"}
                    >
                      <div className="flex items-center gap-1">
                        PID {getSortIcon("pid")}
                      </div>
                    </th>
                    <th
                      scope="col"
                      className="text-left px-4 py-3 text-sm font-medium text-gray-400 cursor-pointer hover:text-white"
                      onClick={() => handleSort("name")}
                      onKeyDown={(e) => e.key === "Enter" && handleSort("name")}
                      tabIndex={0}
                      role="columnheader"
                      aria-sort={sortField === "name" ? (sortDirection === "asc" ? "ascending" : "descending") : "none"}
                    >
                      <div className="flex items-center gap-1">
                        Proceso {getSortIcon("name")}
                      </div>
                    </th>
                    <th scope="col" className="text-left px-4 py-3 text-sm font-medium text-gray-400">
                      Usuario
                    </th>
                    <th
                      scope="col"
                      className="text-right px-4 py-3 text-sm font-medium text-gray-400 cursor-pointer hover:text-white"
                      onClick={() => handleSort("cpu_usage")}
                      onKeyDown={(e) => e.key === "Enter" && handleSort("cpu_usage")}
                      tabIndex={0}
                      role="columnheader"
                      aria-sort={sortField === "cpu_usage" ? (sortDirection === "asc" ? "ascending" : "descending") : "none"}
                    >
                      <div className="flex items-center justify-end gap-1">
                        CPU {getSortIcon("cpu_usage")}
                      </div>
                    </th>
                    <th
                      scope="col"
                      className="text-right px-4 py-3 text-sm font-medium text-gray-400 cursor-pointer hover:text-white"
                      onClick={() => handleSort("memory_mb")}
                      onKeyDown={(e) => e.key === "Enter" && handleSort("memory_mb")}
                      tabIndex={0}
                      role="columnheader"
                      aria-sort={sortField === "memory_mb" ? (sortDirection === "asc" ? "ascending" : "descending") : "none"}
                    >
                      <div className="flex items-center justify-end gap-1">
                        RAM {getSortIcon("memory_mb")}
                      </div>
                    </th>
                    <th scope="col" className="text-left px-4 py-3 text-sm font-medium text-gray-400">
                      Estado
                    </th>
                  </tr>
                </thead>
                <tbody>
                  {(viewMode === "tree"
                    ? flattenTree(buildProcessTree(filteredAndSorted))
                    : filteredAndSorted
                  ).map((proc) => {
                    const treeProc = proc as TreeNode;
                    const hasChildren = viewMode === "tree" && treeProc.children?.length > 0;
                    const depth = viewMode === "tree" ? (treeProc.depth || 0) : 0;

                    return (
                      <tr
                        key={proc.pid}
                        onClick={() => setSelectedProcess(proc)}
                        className={`border-b border-dark-border/50 cursor-pointer transition-colors ${
                          selectedProcess?.pid === proc.pid
                            ? "bg-primary-500/10"
                            : "hover:bg-dark-border/30"
                        }`}
                      >
                        <td className="px-4 py-2 text-gray-400 font-mono text-sm">
                          {proc.pid}
                        </td>
                        <td className="px-4 py-2">
                          <div className="flex items-center">
                            {viewMode === "tree" && (
                              <span style={{ width: `${depth * 16}px` }} className="flex-shrink-0" />
                            )}
                            {viewMode === "tree" && hasChildren && (
                              <button
                                onClick={(e) => {
                                  e.stopPropagation();
                                  toggleExpand(proc.pid);
                                }}
                                className="w-4 h-4 flex items-center justify-center text-gray-500 hover:text-white mr-1"
                                aria-label={expandedPids.has(proc.pid) ? `Contraer procesos hijos de ${proc.name}` : `Expandir procesos hijos de ${proc.name}`}
                                aria-expanded={expandedPids.has(proc.pid)}
                              >
                                {expandedPids.has(proc.pid) ? "−" : "+"}
                              </button>
                            )}
                            {viewMode === "tree" && !hasChildren && depth > 0 && (
                              <span className="w-4 mr-1" />
                            )}
                            <span className="font-medium">{proc.name}</span>
                          </div>
                        </td>
                        <td className="px-4 py-2 text-gray-400 text-sm">
                          {proc.user}
                        </td>
                        <td className="px-4 py-2 text-right">
                          <span
                            className={`font-mono text-sm ${
                              proc.cpu_usage > 50
                                ? "text-red-400"
                                : proc.cpu_usage > 20
                                ? "text-yellow-400"
                                : "text-gray-400"
                            }`}
                          >
                            {proc.cpu_usage.toFixed(1)}%
                          </span>
                        </td>
                        <td className="px-4 py-2 text-right">
                          <span className="font-mono text-sm text-gray-400">
                            {proc.memory_mb >= 1024
                              ? `${(proc.memory_mb / 1024).toFixed(1)} GB`
                              : `${proc.memory_mb.toFixed(0)} MB`}
                          </span>
                        </td>
                        <td className="px-4 py-2">
                          <span
                            className={`text-xs px-2 py-0.5 rounded-full ${
                              proc.state === "Ejecutando"
                                ? "bg-green-500/20 text-green-400"
                                : proc.state === "Suspendido"
                                ? "bg-yellow-500/20 text-yellow-400"
                                : "bg-gray-500/20 text-gray-400"
                            }`}
                          >
                            {proc.state}
                          </span>
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>

            {filteredAndSorted.length === 0 && (
              <div className="p-8 text-center text-gray-400">
                <Cpu size={48} className="mx-auto mb-4 opacity-50" />
                <p>No se encontraron procesos</p>
              </div>
            )}
          </div>
        </div>

        {/* Details panel */}
        {selectedProcess && (
          <div className="w-80">
            <div className="bg-dark-card border border-dark-border rounded-xl p-4 sticky top-6">
              <h3 className="font-semibold mb-4">Detalles del Proceso</h3>

              <div className="space-y-4">
                <div>
                  <p className="text-sm text-gray-400">Proceso</p>
                  <p className="font-medium text-lg">{selectedProcess.name}</p>
                  <p className="text-sm text-gray-500">PID: {selectedProcess.pid}</p>
                </div>

                <div className="grid grid-cols-2 gap-4">
                  <div className="bg-dark-bg p-3 rounded-lg">
                    <p className="text-xs text-gray-400">CPU</p>
                    <p
                      className={`text-xl font-bold ${
                        selectedProcess.cpu_usage > 50
                          ? "text-red-400"
                          : selectedProcess.cpu_usage > 20
                          ? "text-yellow-400"
                          : "text-green-400"
                      }`}
                    >
                      {selectedProcess.cpu_usage.toFixed(1)}%
                    </p>
                  </div>
                  <div className="bg-dark-bg p-3 rounded-lg">
                    <p className="text-xs text-gray-400">RAM</p>
                    <p className="text-xl font-bold text-blue-400">
                      {selectedProcess.memory_mb >= 1024
                        ? `${(selectedProcess.memory_mb / 1024).toFixed(1)} GB`
                        : `${selectedProcess.memory_mb.toFixed(0)} MB`}
                    </p>
                  </div>
                </div>

                <div>
                  <p className="text-sm text-gray-400">Usuario</p>
                  <p className="font-mono text-sm">{selectedProcess.user}</p>
                </div>

                <div>
                  <p className="text-sm text-gray-400">Estado</p>
                  <p className="text-sm">{selectedProcess.state}</p>
                </div>

                <div>
                  <p className="text-sm text-gray-400">Hilos</p>
                  <p className="text-sm">{selectedProcess.threads}</p>
                </div>

                {selectedProcess.command && (
                  <div>
                    <p className="text-sm text-gray-400">Comando</p>
                    <p className="font-mono text-xs text-gray-300 break-all bg-dark-bg p-2 rounded max-h-24 overflow-auto">
                      {selectedProcess.command}
                    </p>
                  </div>
                )}

                <div className="pt-4 space-y-2">
                  <p className="text-xs text-gray-500 mb-2" id="signal-label">Enviar senal:</p>
                  <div className="grid grid-cols-2 gap-2" role="group" aria-labelledby="signal-label">
                    <button
                      onClick={async () => {
                        const confirmed = await confirm({
                          title: "Terminar proceso",
                          message: `¿Terminar "${selectedProcess.name}" (PID: ${selectedProcess.pid})?`,
                          confirmLabel: "Terminar",
                          cancelLabel: "Cancelar",
                          variant: "warning",
                        });
                        if (confirmed) sendSignal(selectedProcess.pid, "SIGTERM");
                      }}
                      className="flex items-center justify-center gap-1 px-3 py-2 bg-yellow-500/20 hover:bg-yellow-500/30 text-yellow-400 border border-yellow-500/30 rounded-lg transition-colors text-sm"
                      aria-label={`Enviar senal TERM a ${selectedProcess.name}`}
                    >
                      <Square size={14} aria-hidden="true" />
                      <span>TERM</span>
                    </button>
                    <button
                      onClick={async () => {
                        const confirmed = await confirm({
                          title: "Forzar cierre",
                          message: `¿FORZAR cierre de "${selectedProcess.name}"? Puede causar perdida de datos.`,
                          confirmLabel: "Forzar",
                          cancelLabel: "Cancelar",
                          variant: "danger",
                        });
                        if (confirmed) sendSignal(selectedProcess.pid, "SIGKILL");
                      }}
                      className="flex items-center justify-center gap-1 px-3 py-2 bg-red-500/20 hover:bg-red-500/30 text-red-400 border border-red-500/30 rounded-lg transition-colors text-sm"
                      aria-label={`Enviar senal KILL a ${selectedProcess.name}`}
                    >
                      <Zap size={14} aria-hidden="true" />
                      <span>KILL</span>
                    </button>
                    <button
                      onClick={() => sendSignal(selectedProcess.pid, "SIGSTOP")}
                      className="flex items-center justify-center gap-1 px-3 py-2 bg-blue-500/20 hover:bg-blue-500/30 text-blue-400 border border-blue-500/30 rounded-lg transition-colors text-sm"
                      aria-label={`Enviar senal STOP a ${selectedProcess.name}`}
                    >
                      <Pause size={14} aria-hidden="true" />
                      <span>STOP</span>
                    </button>
                    <button
                      onClick={() => sendSignal(selectedProcess.pid, "SIGCONT")}
                      className="flex items-center justify-center gap-1 px-3 py-2 bg-green-500/20 hover:bg-green-500/30 text-green-400 border border-green-500/30 rounded-lg transition-colors text-sm"
                      aria-label={`Enviar senal CONT a ${selectedProcess.name}`}
                    >
                      <Play size={14} aria-hidden="true" />
                      <span>CONT</span>
                    </button>
                  </div>
                </div>
              </div>
            </div>
          </div>
        )}
      </div>

      <ConfirmDialog {...dialogProps} />
    </div>
  );
}

export default ProcessesView;
