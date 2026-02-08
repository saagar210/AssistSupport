import React, { createContext, useContext, useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { MemoryKernelPreflightStatus, ModelInfo } from '../types';

export interface AppStatusState {
  // LLM status
  llmLoaded: boolean;
  llmModelName: string | null;
  llmModelInfo: ModelInfo | null;
  llmLoading: boolean;

  // Embeddings status
  embeddingsLoaded: boolean;
  embeddingsModelName: string | null;

  // Vector store status
  vectorEnabled: boolean;
  vectorConsent: boolean;

  // KB status
  kbIndexed: boolean;
  kbDocumentCount: number;
  kbChunkCount: number;

  // MemoryKernel integration status
  memoryKernelFeatureEnabled: boolean;
  memoryKernelReady: boolean;
  memoryKernelStatus: string;
  memoryKernelDetail: string;
  memoryKernelReleaseTag: string | null;
  memoryKernelCommitSha: string | null;
  memoryKernelServiceContract: string | null;
  memoryKernelApiContract: string | null;
  memoryKernelIntegrationBaseline: string | null;

  // Overall
  initialized: boolean;
  lastUpdated: Date | null;
}

interface AppStatusContextValue extends AppStatusState {
  refresh: () => Promise<void>;
  refreshLlm: () => Promise<void>;
  refreshKb: () => Promise<void>;
}

const defaultState: AppStatusState = {
  llmLoaded: false,
  llmModelName: null,
  llmModelInfo: null,
  llmLoading: false,
  embeddingsLoaded: false,
  embeddingsModelName: null,
  vectorEnabled: false,
  vectorConsent: false,
  kbIndexed: false,
  kbDocumentCount: 0,
  kbChunkCount: 0,
  memoryKernelFeatureEnabled: false,
  memoryKernelReady: false,
  memoryKernelStatus: 'unknown',
  memoryKernelDetail: 'Not checked',
  memoryKernelReleaseTag: null,
  memoryKernelCommitSha: null,
  memoryKernelServiceContract: null,
  memoryKernelApiContract: null,
  memoryKernelIntegrationBaseline: null,
  initialized: false,
  lastUpdated: null,
};

const AppStatusContext = createContext<AppStatusContextValue | null>(null);

interface Props {
  children: React.ReactNode;
  pollInterval?: number; // ms, default 10000 (10 seconds)
}

export function AppStatusProvider({ children, pollInterval = 10000 }: Props) {
  const [state, setState] = useState<AppStatusState>(defaultState);
  const pollRef = useRef<number | null>(null);

  const refreshLlm = useCallback(async () => {
    try {
      const isLoaded = await invoke<boolean>('is_model_loaded');
      if (isLoaded) {
        const info = await invoke<ModelInfo | null>('get_model_info');
        setState(prev => ({
          ...prev,
          llmLoaded: true,
          llmModelName: info?.name ?? info?.id ?? 'Unknown',
          llmModelInfo: info,
          llmLoading: false,
        }));
      } else {
        setState(prev => ({
          ...prev,
          llmLoaded: false,
          llmModelName: null,
          llmModelInfo: null,
          llmLoading: false,
        }));
      }
    } catch (e) {
      console.error('Failed to check LLM status:', e);
    }
  }, []);

  const refreshEmbeddings = useCallback(async () => {
    try {
      const isLoaded = await invoke<boolean>('is_embedding_model_loaded');
      setState(prev => ({
        ...prev,
        embeddingsLoaded: isLoaded,
        embeddingsModelName: isLoaded ? 'default' : null,
      }));
    } catch {
      // Embedding check may not exist
    }
  }, []);

  const refreshVector = useCallback(async () => {
    try {
      const consent = await invoke<{ enabled: boolean; consented_at: string | null }>('get_vector_consent');
      setState(prev => ({
        ...prev,
        vectorEnabled: consent.enabled,
        vectorConsent: consent.enabled,
      }));
    } catch {
      // Vector consent may not be available
    }
  }, []);

  const refreshKb = useCallback(async () => {
    try {
      const stats = await invoke<{ document_count: number; chunk_count: number; namespace_count: number }>('get_kb_stats');
      setState(prev => ({
        ...prev,
        kbIndexed: stats.chunk_count > 0,
        kbDocumentCount: stats.document_count,
        kbChunkCount: stats.chunk_count,
      }));
    } catch {
      // KB stats may fail if not initialized
    }
  }, []);

  const refreshMemoryKernel = useCallback(async () => {
    try {
      const preflight = await invoke<MemoryKernelPreflightStatus>('get_memory_kernel_preflight_status');
      setState(prev => ({
        ...prev,
        memoryKernelFeatureEnabled: preflight.enabled,
        memoryKernelReady: preflight.ready && preflight.enrichment_enabled,
        memoryKernelStatus: preflight.status,
        memoryKernelDetail: preflight.message,
        memoryKernelReleaseTag: preflight.release_tag ?? null,
        memoryKernelCommitSha: preflight.commit_sha ?? null,
        memoryKernelServiceContract:
          preflight.service_contract_version ?? preflight.expected_service_contract_version ?? null,
        memoryKernelApiContract:
          preflight.api_contract_version ?? preflight.expected_api_contract_version ?? null,
        memoryKernelIntegrationBaseline: preflight.integration_baseline ?? null,
      }));
    } catch (err) {
      setState(prev => ({
        ...prev,
        memoryKernelFeatureEnabled: false,
        memoryKernelReady: false,
        memoryKernelStatus: 'error',
        memoryKernelDetail: `Preflight unavailable: ${String(err)}`,
        memoryKernelReleaseTag: null,
        memoryKernelCommitSha: null,
        memoryKernelServiceContract: null,
        memoryKernelApiContract: null,
        memoryKernelIntegrationBaseline: null,
      }));
    }
  }, []);

  const refresh = useCallback(async () => {
    await Promise.all([
      refreshLlm(),
      refreshEmbeddings(),
      refreshVector(),
      refreshKb(),
      refreshMemoryKernel(),
    ]);
    setState(prev => ({
      ...prev,
      initialized: true,
      lastUpdated: new Date(),
    }));
  }, [refreshLlm, refreshEmbeddings, refreshVector, refreshKb, refreshMemoryKernel]);

  // Initial load and polling
  useEffect(() => {
    refresh();

    if (pollInterval > 0) {
      pollRef.current = window.setInterval(refresh, pollInterval);
    }

    return () => {
      if (pollRef.current !== null) {
        clearInterval(pollRef.current);
      }
    };
  }, [refresh, pollInterval]);

  const value: AppStatusContextValue = {
    ...state,
    refresh,
    refreshLlm,
    refreshKb,
  };

  return (
    <AppStatusContext.Provider value={value}>
      {children}
    </AppStatusContext.Provider>
  );
}

export function useAppStatus(): AppStatusContextValue {
  const ctx = useContext(AppStatusContext);
  if (!ctx) {
    throw new Error('useAppStatus must be used within AppStatusProvider');
  }
  return ctx;
}
