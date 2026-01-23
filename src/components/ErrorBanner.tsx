import { AlertCircle, RefreshCw } from "lucide-react";

interface ErrorBannerProps {
  error: Error | string;
  onRetry?: () => void;
  className?: string;
}

/**
 * Banner component for displaying errors with optional retry button.
 */
export function ErrorBanner({ error, onRetry, className = "" }: ErrorBannerProps) {
  const errorMessage = error instanceof Error ? error.message : error;

  return (
    <div
      className={`bg-red-500/10 border border-red-500/30 rounded-xl p-4 ${className}`}
    >
      <div className="flex items-start gap-3">
        <AlertCircle size={20} className="text-red-400 flex-shrink-0 mt-0.5" />
        <div className="flex-1">
          <p className="font-medium text-red-400">Error</p>
          <p className="text-sm text-red-300 mt-1">{errorMessage}</p>
        </div>
        {onRetry && (
          <button
            onClick={onRetry}
            className="flex items-center gap-2 px-3 py-1.5 bg-red-500/20 hover:bg-red-500/30 text-red-300 rounded-lg transition-colors text-sm"
          >
            <RefreshCw size={14} />
            <span>Reintentar</span>
          </button>
        )}
      </div>
    </div>
  );
}

export default ErrorBanner;
