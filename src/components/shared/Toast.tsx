import type { ToastType } from '../../types';
import './Toast.css';

export interface ToastProps {
  message: string;
  type: ToastType;
  onClose: () => void;
}

export function Toast({ message, type, onClose }: ToastProps) {
  return (
    <div className={`toast toast-${type}`} role="alert">
      <span className="toast-message">{message}</span>
      <button
        className="toast-dismiss"
        onClick={onClose}
        aria-label="Dismiss"
      >
        &times;
      </button>
    </div>
  );
}

interface ToastContainerProps {
  children: React.ReactNode;
}

export function ToastContainer({ children }: ToastContainerProps) {
  return (
    <div className="toast-container" aria-live="polite" aria-atomic="false">
      {children}
    </div>
  );
}
