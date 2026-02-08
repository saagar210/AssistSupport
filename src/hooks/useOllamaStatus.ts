import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

interface OllamaStatus {
  connected: boolean;
  version: string;
  loading: boolean;
}

interface OllamaCheckResult {
  connected: boolean;
  version: string;
}

export function useOllamaStatus(): OllamaStatus {
  const [connected, setConnected] = useState(false);
  const [version, setVersion] = useState("");
  const [loading, setLoading] = useState(true);

  const check = useCallback(async () => {
    try {
      const result = await invoke<OllamaCheckResult>(
        "check_ollama_connection",
      );
      setConnected(result.connected);
      setVersion(result.version);
    } catch {
      setConnected(false);
      setVersion("");
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    check();
    const interval = setInterval(check, 30_000);
    return () => clearInterval(interval);
  }, [check]);

  return { connected, version, loading };
}
