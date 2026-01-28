import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { ResponseAlternative } from '../types';

export function useAlternatives() {
  const [alternatives, setAlternatives] = useState<ResponseAlternative[]>([]);
  const [loading, setLoading] = useState(false);

  const loadAlternatives = useCallback(async (draftId: string) => {
    setLoading(true);
    try {
      const data = await invoke<ResponseAlternative[]>('get_alternatives_for_draft', { draftId });
      setAlternatives(data);
      return data;
    } catch (err) {
      console.error('Failed to load alternatives:', err);
      return [];
    } finally {
      setLoading(false);
    }
  }, []);

  const saveAlternative = useCallback(async (
    draftId: string,
    originalText: string,
    alternativeText: string,
    opts?: {
      sourcesJson?: string;
      metricsJson?: string;
      generationParamsJson?: string;
    }
  ): Promise<string | null> => {
    try {
      const id = await invoke<string>('save_response_alternative', {
        draftId,
        originalText,
        alternativeText,
        sourcesJson: opts?.sourcesJson ?? null,
        metricsJson: opts?.metricsJson ?? null,
        generationParamsJson: opts?.generationParamsJson ?? null,
      });
      await loadAlternatives(draftId);
      return id;
    } catch (err) {
      console.error('Failed to save alternative:', err);
      return null;
    }
  }, [loadAlternatives]);

  const chooseAlternative = useCallback(async (
    alternativeId: string,
    choice: 'original' | 'alternative'
  ) => {
    try {
      await invoke('choose_alternative', { alternativeId, choice });
    } catch (err) {
      console.error('Failed to choose alternative:', err);
    }
  }, []);

  return {
    alternatives,
    loading,
    loadAlternatives,
    saveAlternative,
    chooseAlternative,
  };
}
