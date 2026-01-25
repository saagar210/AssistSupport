import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { InitResult, VectorConsent } from '../types';

export interface AppInitState {
  initialized: boolean;
  loading: boolean;
  error: string | null;
  initResult: InitResult | null;
  vectorConsent: VectorConsent | null;
}

export function useInitialize() {
  const [state, setState] = useState<AppInitState>({
    initialized: false,
    loading: true,
    error: null,
    initResult: null,
    vectorConsent: null,
  });

  useEffect(() => {
    async function initialize() {
      try {
        // Initialize the app (creates DB, loads master key from Keychain)
        const result = await invoke<InitResult>('initialize_app');

        // Verify FTS5 is available
        const fts5 = await invoke<boolean>('check_fts5_enabled');
        if (!fts5) {
          throw new Error('FTS5 full-text search is not available');
        }

        // Check vector consent status
        const consent = await invoke<VectorConsent>('get_vector_consent');

        // Try to initialize LLM engine (non-fatal if it fails)
        try {
          await invoke('init_llm_engine');
        } catch (e) {
          console.warn('LLM engine init failed (will init on first use):', e);
        }

        // Try to initialize embedding engine (non-fatal if it fails)
        try {
          await invoke('init_embedding_engine');
        } catch (e) {
          console.warn('Embedding engine init failed (will init on first use):', e);
        }

        setState({
          initialized: true,
          loading: false,
          error: null,
          initResult: result,
          vectorConsent: consent,
        });
      } catch (e) {
        console.error('Initialization failed:', e);
        setState(prev => ({
          ...prev,
          loading: false,
          error: String(e),
        }));
      }
    }

    initialize();
  }, []);

  return state;
}
