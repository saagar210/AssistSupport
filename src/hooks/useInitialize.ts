import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { InitResult, VectorConsent } from '../types';

export interface AppInitState {
  initialized: boolean;
  loading: boolean;
  error: string | null;
  initResult: InitResult | null;
  vectorConsent: VectorConsent | null;
  enginesReady: boolean;
}

// Timeout for optional initialization operations (5 seconds)
const INIT_TIMEOUT = 5000;

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

        // Mark as initialized - UI can render now
        setState({
          initialized: true,
          loading: false,
          error: null,
          initResult: result,
          vectorConsent: consent,
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
