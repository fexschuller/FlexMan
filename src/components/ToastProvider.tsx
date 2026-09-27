import { createContext, useCallback, useContext, useState, type ReactNode } from "react";

interface ErrorToast {
  id: number;
  message: string;
}

type ShowError = (message: string) => void;

const TOAST_DURATION_MS = 6000;

const ToastContext = createContext<ShowError | null>(null);

let nextToastId = 0;

export function ToastProvider({ children }: { children: ReactNode }) {
  const [toasts, setToasts] = useState<ErrorToast[]>([]);

  function dismiss(id: number) {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  }

  const showError = useCallback<ShowError>((message) => {
    const id = nextToastId++;
    setToasts((prev) => [...prev, { id, message }]);
    setTimeout(() => dismiss(id), TOAST_DURATION_MS);
  }, []);

  return (
    <ToastContext.Provider value={showError}>
      {children}
      <div className="toast-stack" role="alert" aria-live="assertive">
        {toasts.map((toast) => (
          <div key={toast.id} className="toast" onClick={() => dismiss(toast.id)}>
            {toast.message}
          </div>
        ))}
      </div>
    </ToastContext.Provider>
  );
}

// Named "toast" for the UI concept, but every call site uses it to report a
// failure - there's no success/info variant.
export function useToast(): ShowError {
  const showError = useContext(ToastContext);
  if (!showError) {
    throw new Error("useToast must be used within a ToastProvider");
  }
  return showError;
}
