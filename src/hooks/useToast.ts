import { useState, useCallback } from 'react';
import type { Toast, ToastType } from '../types';

let toastId = 0;

export function useToast() {
  const [toasts, setToasts] = useState<Toast[]>([]);

  const addToast = useCallback((message: string, type: ToastType = 'info', duration?: number) => {
    const id = String(++toastId);
    const toast: Toast = { id, type, message, duration };

    setToasts(prev => [...prev, toast]);

    // Auto-dismiss success and info toasts
    if (type === 'success' || type === 'info') {
      const timeout = duration ?? 3000;
      setTimeout(() => {
        setToasts(prev => prev.filter(t => t.id !== id));
      }, timeout);
    }

    return id;
  }, []);

  const removeToast = useCallback((id: string) => {
    setToasts(prev => prev.filter(t => t.id !== id));
  }, []);

  const success = useCallback((message: string) => addToast(message, 'success'), [addToast]);
  const error = useCallback((message: string) => addToast(message, 'error'), [addToast]);
  const info = useCallback((message: string) => addToast(message, 'info'), [addToast]);
  const warning = useCallback((message: string) => addToast(message, 'warning'), [addToast]);

  return {
    toasts,
    addToast,
    removeToast,
    success,
    error,
    info,
    warning,
  };
}
