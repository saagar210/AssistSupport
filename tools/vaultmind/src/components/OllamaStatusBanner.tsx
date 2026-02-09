import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AlertTriangle, RefreshCw, Loader2 } from "lucide-react";
import { useOllamaStatus } from "../hooks/useOllamaStatus";

export function OllamaStatusBanner() {
  const { connected, loading } = useOllamaStatus();
  const [retrying, setRetrying] = useState(false);
  const retryTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const retryingRef = useRef(false);

  const handleRetry = useCallback(async () => {
    if (retryingRef.current) return;
    retryingRef.current = true;
    setRetrying(true);
    try {
      await invoke("check_ollama_connection");
    } catch {
      // Status will update via the hook
    } finally {
      setRetrying(false);
      retryingRef.current = false;
    }
  }, []);

  // Auto-retry every 10 seconds when disconnected, using setTimeout chaining
  useEffect(() => {
    if (connected || loading) return;

    function scheduleRetry() {
      retryTimerRef.current = setTimeout(async () => {
        await handleRetry();
        // Schedule next retry only if still mounted (cleanup hasn't run)
        if (retryTimerRef.current !== null) {
          scheduleRetry();
        }
      }, 10_000);
    }

    scheduleRetry();

    return () => {
      if (retryTimerRef.current) {
        clearTimeout(retryTimerRef.current);
        retryTimerRef.current = null;
      }
    };
  }, [connected, loading, handleRetry]);

  if (connected || loading) {
    return null;
  }

  return (
    <div
      data-testid="ollama-banner"
      className="flex items-center gap-2 border-b border-yellow-500/30 bg-yellow-500/10 px-4 py-2 text-sm text-yellow-600 dark:text-yellow-400"
    >
      <AlertTriangle size={14} className="shrink-0" />
      <span className="flex-1">
        Ollama disconnected. Search and chat unavailable.
      </span>
      <button
        onClick={handleRetry}
        disabled={retrying}
        className="inline-flex items-center gap-1 rounded px-2 py-0.5 text-xs font-medium transition-colors hover:bg-yellow-500/20 disabled:opacity-50"
      >
        {retrying ? (
          <Loader2 size={12} className="animate-spin" />
        ) : (
          <RefreshCw size={12} />
        )}
        Retry
      </button>
    </div>
  );
}
