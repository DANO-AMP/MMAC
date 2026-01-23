import { useState, useCallback } from "react";

interface ConfirmationState {
  isOpen: boolean;
  title: string;
  message: string;
  confirmLabel: string;
  cancelLabel: string;
  variant: "danger" | "warning" | "default";
  onConfirm: () => void;
  onCancel: () => void;
}

interface ConfirmOptions {
  title: string;
  message: string;
  confirmLabel?: string;
  cancelLabel?: string;
  variant?: "danger" | "warning" | "default";
}

const defaultState: ConfirmationState = {
  isOpen: false,
  title: "",
  message: "",
  confirmLabel: "Confirmar",
  cancelLabel: "Cancelar",
  variant: "default",
  onConfirm: () => {},
  onCancel: () => {},
};

/**
 * Hook for managing confirmation dialogs.
 *
 * @example
 * ```tsx
 * const { confirm, dialogProps } = useConfirmation();
 *
 * const handleDelete = async () => {
 *   const confirmed = await confirm({
 *     title: "Eliminar archivo",
 *     message: "Esta accion no se puede deshacer.",
 *     variant: "danger",
 *   });
 *   if (confirmed) {
 *     await deleteFile();
 *   }
 * };
 *
 * return (
 *   <>
 *     <button onClick={handleDelete}>Eliminar</button>
 *     <ConfirmDialog {...dialogProps} />
 *   </>
 * );
 * ```
 */
export function useConfirmation() {
  const [state, setState] = useState<ConfirmationState>(defaultState);

  const confirm = useCallback(
    (options: ConfirmOptions): Promise<boolean> => {
      return new Promise((resolve) => {
        setState({
          isOpen: true,
          title: options.title,
          message: options.message,
          confirmLabel: options.confirmLabel ?? "Confirmar",
          cancelLabel: options.cancelLabel ?? "Cancelar",
          variant: options.variant ?? "default",
          onConfirm: () => {
            setState(defaultState);
            resolve(true);
          },
          onCancel: () => {
            setState(defaultState);
            resolve(false);
          },
        });
      });
    },
    []
  );

  const close = useCallback(() => {
    setState(defaultState);
  }, []);

  return {
    confirm,
    close,
    dialogProps: state,
  };
}

export default useConfirmation;
