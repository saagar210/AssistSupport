import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

interface OllamaStatus {
  connected: boolean;
  version: string;
  loading: boolean;
}

export function useOllamaStatus(): OllamaStatus {
  const [connected, setConnected] = useState(false);
  const [version, setVersion] = useState("");
  const [loading, setLoading] = useState(true);

  const check = useCallback(async () => {
    try {
      const result = await invoke<[boolean, string]>(
        "check_ollama_connection",
      );
      setConnected(result[0]);
      setVersion(result[1]);
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
