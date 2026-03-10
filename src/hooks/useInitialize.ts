import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { InitResult, MemoryKernelPreflightStatus, VectorConsent, ModelStateResult } from '../types';

export interface AppInitState {
  initialized: boolean;
  loading: boolean;
  error: string | null;
  initResult: InitResult | null;
  vectorConsent: VectorConsent | null;
  memoryKernelPreflight: MemoryKernelPreflightStatus | null;
  enginesReady: boolean;
}

// Timeout for optional initialization operations (5 seconds)
const INIT_TIMEOUT = 5000;

const IS_DEV = Boolean(import.meta.env?.DEV);

function logNonFatal(message: string, error: unknown) {
  if (!IS_DEV) return;
  // eslint-disable-next-line no-console
  console.warn(message, error);
}

// Helper to create a timeout promise
function withTimeout<T>(promise: Promise<T>, ms: number, label: string): Promise<T> {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      reject(new Error(`${label} timed out after ${ms}ms`));
    }, ms);

    promise
      .then(value => {
        clearTimeout(timer);
        resolve(value);
      })
      .catch(err => {
        clearTimeout(timer);
        reject(err);
      });
  });
}

export function useInitialize() {
  const [state, setState] = useState<AppInitState>({
    initialized: false,
    loading: true,
    error: null,
    initResult: null,
    vectorConsent: null,
    memoryKernelPreflight: null,
    enginesReady: false,
  });

  // Initialize engines in background (non-blocking)
  const initEnginesInBackground = useCallback(async () => {
    try {
      // Run LLM and embedding engine initialization in parallel with timeout
      await Promise.allSettled([
        withTimeout(invoke('init_llm_engine'), INIT_TIMEOUT, 'LLM engine init'),
        withTimeout(invoke('init_embedding_engine'), INIT_TIMEOUT, 'Embedding engine init'),
      ]);

      setState(prev => ({ ...prev, enginesReady: true }));

      // Auto-load last-used models in background (non-blocking)
      try {
        const modelState = await invoke<ModelStateResult>('get_model_state');
        const autoLoadPromises: Promise<unknown>[] = [];

        if (modelState.llm_model_id && !modelState.llm_loaded) {
          autoLoadPromises.push(
            invoke('load_model', { modelId: modelState.llm_model_id })
              .catch(e => logNonFatal('Auto-load LLM failed (non-fatal):', e))
          );
        }

        if (modelState.embeddings_model_path && !modelState.embeddings_loaded) {
          autoLoadPromises.push(
            invoke('load_embedding_model', { path: modelState.embeddings_model_path })
              .catch(e => logNonFatal('Auto-load embeddings failed (non-fatal):', e))
          );
        }

        if (autoLoadPromises.length > 0) {
          await Promise.allSettled(autoLoadPromises);
        }
      } catch (e) {
        // Non-fatal - user can load models manually
        logNonFatal('Model auto-load check failed:', e);
      }
    } catch (e) {
      // Non-fatal - engines will init on first use
      logNonFatal('Background engine init completed with warnings:', e);
      setState(prev => ({ ...prev, enginesReady: true }));
    }
  }, []);

  // Refresh MemoryKernel preflight in background (non-blocking)
  const refreshMemoryKernelPreflightInBackground = useCallback(async () => {
    try {
      const status = await withTimeout(
        invoke<MemoryKernelPreflightStatus>('get_memory_kernel_preflight_status'),
        INIT_TIMEOUT,
        'MemoryKernel preflight',
      );
      setState(prev => ({ ...prev, memoryKernelPreflight: status }));
    } catch (e) {
      logNonFatal('MemoryKernel preflight failed (enrichment disabled):', e);
      setState(prev => ({ ...prev, memoryKernelPreflight: null }));
    }
  }, []);

  const finishInitializedState = useCallback(async (result: InitResult) => {
    const fts5 = await invoke<boolean>('check_fts5_enabled');
    if (!fts5) {
      throw new Error('FTS5 full-text search is not available');
    }

    let consent: VectorConsent | null = null;
    try {
      consent = await invoke<VectorConsent>('get_vector_consent');
    } catch (e) {
      logNonFatal('Vector consent check failed (using defaults):', e);
      consent = { enabled: false, consented_at: null, encryption_supported: false };
    }

    setState({
      initialized: true,
      loading: false,
      error: null,
      initResult: result,
      vectorConsent: consent,
      memoryKernelPreflight: null,
      enginesReady: false,
    });

    refreshMemoryKernelPreflightInBackground();
    initEnginesInBackground();
  }, [initEnginesInBackground, refreshMemoryKernelPreflightInBackground]);

  const setRecoveryState = useCallback((result: InitResult) => {
    setState({
      initialized: false,
      loading: false,
      error: null,
      initResult: result,
      vectorConsent: null,
      memoryKernelPreflight: null,
      enginesReady: false,
    });
  }, []);

  const unlockWithPassphrase = useCallback(async (passphrase: string) => {
    try {
      const result = await invoke<InitResult>('unlock_with_passphrase', { passphrase });
      if (result.recovery_issue) {
        setRecoveryState(result);
        return;
      }
      await finishInitializedState(result);
    } catch (e) {
      if (IS_DEV) {
        // eslint-disable-next-line no-console
        console.error('Passphrase unlock failed:', e);
      }
      setState(prev => ({
        ...prev,
        loading: false,
        error: String(e),
      }));
      throw e;
    }
  }, [finishInitializedState, setRecoveryState]);

  useEffect(() => {
    async function initialize() {
      try {
        // CRITICAL PATH: Initialize the app (creates DB, loads master key)
        const result = await invoke<InitResult>('initialize_app');

        if (result.passphrase_required) {
          setState({
            initialized: false,
            loading: false,
            error: null,
            initResult: result,
            vectorConsent: null,
            memoryKernelPreflight: null,
            enginesReady: false,
          });
          return;
        }
        if (result.recovery_issue) {
          setRecoveryState(result);
          return;
        }
        await finishInitializedState(result);

      } catch (e) {
        if (IS_DEV) {
          // eslint-disable-next-line no-console
          console.error('Critical initialization failed:', e);
        }
        setState(prev => ({
          ...prev,
          loading: false,
          error: String(e),
        }));
      }
    }

    initialize();
  }, [finishInitializedState, setRecoveryState]);

  return {
    ...state,
    unlockWithPassphrase,
  };
}
