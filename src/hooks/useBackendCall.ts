import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

export interface UseBackendCallResult<T> {
  data: T | null;
  error: Error | null;
  isLoading: boolean;
  execute: (args?: Record<string, unknown>) => Promise<T | null>;
  retry: () => Promise<T | null>;
  reset: () => void;
}

/**
 * Hook for making backend calls to Tauri commands.
 *
 * @param command - The Tauri command name to invoke
 * @param defaultArgs - Optional default arguments for the command
 * @returns Object with data, error, loading state, and control functions
 *
 * @example
 * ```tsx
 * const { data, error, isLoading, execute } = useBackendCall<PortInfo[]>("scan_ports");
 *
 * useEffect(() => {
 *   execute();
 * }, []);
 *
 * if (isLoading) return <Spinner />;
 * if (error) return <ErrorBanner error={error} onRetry={execute} />;
 * return <PortList ports={data} />;
 * ```
 */
export function useBackendCall<T>(
  command: string,
  defaultArgs?: Record<string, unknown>
): UseBackendCallResult<T> {
  const [data, setData] = useState<T | null>(null);
  const [error, setError] = useState<Error | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [lastArgs, setLastArgs] = useState<Record<string, unknown> | undefined>(
    defaultArgs
  );

  const execute = useCallback(
    async (args?: Record<string, unknown>): Promise<T | null> => {
      const finalArgs = args ?? lastArgs;
      setLastArgs(finalArgs);
      setIsLoading(true);
      setError(null);

      try {
        const result = await invoke<T>(command, finalArgs);
        setData(result);
        return result;
      } catch (err) {
        const error =
          err instanceof Error ? err : new Error(String(err));
        setError(error);
        return null;
      } finally {
        setIsLoading(false);
      }
    },
    [command, lastArgs]
  );

  const retry = useCallback(async (): Promise<T | null> => {
    return execute(lastArgs);
  }, [execute, lastArgs]);

  const reset = useCallback(() => {
    setData(null);
    setError(null);
    setIsLoading(false);
  }, []);

  return {
    data,
    error,
    isLoading,
    execute,
    retry,
    reset,
  };
}

export default useBackendCall;
