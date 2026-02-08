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

// Session token key in localStorage
const SESSION_TOKEN_KEY = 'assistsupport_session_token';

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
              .then(() => console.log('Auto-loaded LLM model:', modelState.llm_model_id))
              .catch(e => console.warn('Auto-load LLM failed (non-fatal):', e))
          );
        }

        if (modelState.embeddings_model_path && !modelState.embeddings_loaded) {
          autoLoadPromises.push(
            invoke('load_embedding_model', { path: modelState.embeddings_model_path })
              .then(() => console.log('Auto-loaded embedding model'))
              .catch(e => console.warn('Auto-load embeddings failed (non-fatal):', e))
          );
        }

        if (autoLoadPromises.length > 0) {
          await Promise.allSettled(autoLoadPromises);
        }
      } catch (e) {
        // Non-fatal - user can load models manually
        console.warn('Model auto-load check failed:', e);
      }
    } catch (e) {
      // Non-fatal - engines will init on first use
      console.warn('Background engine init completed with warnings:', e);
      setState(prev => ({ ...prev, enginesReady: true }));
    }
  }, []);

  useEffect(() => {
    async function initialize() {
      try {
        // CRITICAL PATH: Initialize the app (creates DB, loads master key)
        const result = await invoke<InitResult>('initialize_app');

        // CRITICAL PATH: Verify FTS5 is available (required for search)
        const fts5 = await invoke<boolean>('check_fts5_enabled');
        if (!fts5) {
          throw new Error('FTS5 full-text search is not available');
        }

        // NON-CRITICAL: Check vector consent status (can fail gracefully)
        let consent: VectorConsent | null = null;
        try {
          consent = await invoke<VectorConsent>('get_vector_consent');
        } catch (e) {
          console.warn('Vector consent check failed (using defaults):', e);
          consent = { enabled: false, consented_at: null, encryption_supported: false };
        }

        // NON-CRITICAL: MemoryKernel preflight gate (enrichment must not block startup)
        let memoryKernelPreflight: MemoryKernelPreflightStatus | null = null;
        try {
          memoryKernelPreflight = await invoke<MemoryKernelPreflightStatus>('get_memory_kernel_preflight_status');
        } catch (e) {
          console.warn('MemoryKernel preflight failed (enrichment disabled):', e);
        }

        // Create or validate session token for auto-unlock
        try {
          const savedToken = localStorage.getItem(SESSION_TOKEN_KEY);
          if (savedToken) {
            const isValid = await invoke<boolean>('validate_session_token', { sessionId: savedToken });
            if (!isValid) {
              localStorage.removeItem(SESSION_TOKEN_KEY);
              const newToken = await invoke<string>('create_session_token');
              localStorage.setItem(SESSION_TOKEN_KEY, newToken);
            }
          } else {
            const newToken = await invoke<string>('create_session_token');
            localStorage.setItem(SESSION_TOKEN_KEY, newToken);
          }
        } catch (e) {
          // Session token is a convenience feature, don't block on failure
          console.warn('Session token setup failed (non-fatal):', e);
        }

        // Mark as initialized - UI can render now
        setState({
          initialized: true,
          loading: false,
          error: null,
          initResult: result,
          vectorConsent: consent,
          memoryKernelPreflight,
          enginesReady: false,
        });

        // Start background initialization of LLM/embedding engines
        // This runs AFTER the UI is ready, so users see the app immediately
        initEnginesInBackground();

      } catch (e) {
        console.error('Critical initialization failed:', e);
        setState(prev => ({
          ...prev,
          loading: false,
          error: String(e),
        }));
      }
    }

    initialize();
  }, [initEnginesInBackground]);

  return state;
}
