import { useCallback, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { SearchApiEmbeddingModelStatus } from "../types";

interface SearchApiEmbeddingState {
  status: SearchApiEmbeddingModelStatus | null;
  loading: boolean;
  error: string | null;
}

export function useSearchApiEmbedding() {
  const [state, setState] = useState<SearchApiEmbeddingState>({
    status: null,
    loading: false,
    error: null,
  });

  const refreshStatus =
    useCallback(async (): Promise<SearchApiEmbeddingModelStatus | null> => {
      try {
        const status = await invoke<SearchApiEmbeddingModelStatus>(
          "get_search_api_embedding_model_status",
        );
        setState({ status, loading: false, error: null });
        return status;
      } catch (e) {
        setState((prev) => ({
          ...prev,
          loading: false,
          error: String(e),
        }));
        return null;
      }
    }, []);

  const installModel =
    useCallback(async (): Promise<SearchApiEmbeddingModelStatus> => {
      setState((prev) => ({ ...prev, loading: true, error: null }));
      try {
        const status = await invoke<SearchApiEmbeddingModelStatus>(
          "install_search_api_embedding_model",
        );
        setState({ status, loading: false, error: null });
        return status;
      } catch (e) {
        const message = String(e);
        setState((prev) => ({
          ...prev,
          loading: false,
          error: message,
        }));
        throw e;
      }
    }, []);

  return {
    ...state,
    refreshStatus,
    installModel,
  };
}
