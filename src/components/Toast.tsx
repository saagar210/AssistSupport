import { X, CheckCircle2, AlertCircle, AlertTriangle, Info } from "lucide-react";
import { useToastStore } from "../stores/toastStore";
import type { ToastType } from "../stores/toastStore";

const TOAST_STYLES: Record<ToastType, { bg: string; text: string; icon: typeof Info }> = {
  success: { bg: "bg-success/10 border-success/30", text: "text-success", icon: CheckCircle2 },
  error: { bg: "bg-destructive/10 border-destructive/30", text: "text-destructive", icon: AlertCircle },
  warning: { bg: "bg-warning/10 border-warning/30", text: "text-warning", icon: AlertTriangle },
  info: { bg: "bg-accent/10 border-accent/30", text: "text-accent", icon: Info },
};

export function ToastContainer() {
  const toasts = useToastStore((s) => s.toasts);
  const removeToast = useToastStore((s) => s.removeToast);

  if (toasts.length === 0) return null;

  return (
    <div className="fixed bottom-12 right-4 z-50 flex flex-col gap-2">
      {toasts.map((toast) => {
        const style = TOAST_STYLES[toast.type];
        const Icon = style.icon;
        return (
          <div
            key={toast.id}
            className={`flex items-center gap-2 rounded-lg border px-4 py-2.5 shadow-lg backdrop-blur animate-in slide-in-from-right ${style.bg}`}
          >
            <Icon size={16} className={style.text} />
            <p className={`text-sm ${style.text}`}>{toast.message}</p>
            <button
              onClick={() => removeToast(toast.id)}
              className={`ml-2 rounded p-0.5 transition-colors hover:bg-black/10 ${style.text}`}
            >
              <X size={12} />
            </button>
          </div>
        );
      })}
    </div>
  );
}
