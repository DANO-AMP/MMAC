import { AlertTriangle, Trash2, AlertCircle } from "lucide-react";

interface ConfirmDialogProps {
  isOpen: boolean;
  title: string;
  message: string;
  confirmLabel: string;
  cancelLabel: string;
  variant: "danger" | "warning" | "default";
  onConfirm: () => void;
  onCancel: () => void;
}

const variantConfig = {
  danger: {
    icon: Trash2,
    iconBg: "bg-red-500/20",
    iconColor: "text-red-400",
    confirmBg: "bg-red-500/20 hover:bg-red-500/30 border-red-500/30",
    confirmText: "text-red-400",
  },
  warning: {
    icon: AlertTriangle,
    iconBg: "bg-yellow-500/20",
    iconColor: "text-yellow-400",
    confirmBg: "bg-yellow-500/20 hover:bg-yellow-500/30 border-yellow-500/30",
    confirmText: "text-yellow-400",
  },
  default: {
    icon: AlertCircle,
    iconBg: "bg-primary-500/20",
    iconColor: "text-primary-400",
    confirmBg: "bg-primary-600 hover:bg-primary-700 border-transparent",
    confirmText: "text-white",
  },
};

/**
 * Confirmation dialog component for destructive actions.
 */
export function ConfirmDialog({
  isOpen,
  title,
  message,
  confirmLabel,
  cancelLabel,
  variant,
  onConfirm,
  onCancel,
}: ConfirmDialogProps) {
  if (!isOpen) return null;

  const config = variantConfig[variant];
  const Icon = config.icon;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black/60 backdrop-blur-sm"
        onClick={onCancel}
      />

      {/* Dialog */}
      <div className="relative bg-dark-card border border-dark-border rounded-2xl p-6 max-w-md w-full mx-4 shadow-2xl animate-in fade-in zoom-in-95 duration-200">
        <div className="flex items-start gap-4">
          <div className={`p-3 rounded-xl ${config.iconBg}`}>
            <Icon size={24} className={config.iconColor} />
          </div>
          <div className="flex-1">
            <h3 className="text-lg font-semibold">{title}</h3>
            <p className="text-gray-400 mt-2 text-sm">{message}</p>
          </div>
        </div>

        <div className="flex gap-3 mt-6 justify-end">
          <button
            onClick={onCancel}
            className="px-4 py-2 bg-dark-border hover:bg-dark-border/80 text-white rounded-lg transition-colors"
          >
            {cancelLabel}
          </button>
          <button
            onClick={onConfirm}
            className={`px-4 py-2 border rounded-lg transition-colors ${config.confirmBg} ${config.confirmText}`}
          >
            {confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
}

export default ConfirmDialog;
