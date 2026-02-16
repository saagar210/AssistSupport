import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type {
  HybridSearchResponse,
  SearchApiHealthStatus,
  SearchApiStatsData,
} from '../types';

export interface HybridSearchState {
  response: HybridSearchResponse | null;
  searching: boolean;
  error: string | null;
  apiHealthy: boolean | null;
  apiStatusMessage: string | null;
}

export function useHybridSearch() {
  const [state, setState] = useState<HybridSearchState>({
    response: null,
    searching: false,
    error: null,
    apiHealthy: null,
    apiStatusMessage: null,
  });

  const search = useCallback(async (query: string, topK = 10): Promise<HybridSearchResponse | null> => {
    setState(prev => ({ ...prev, searching: true, error: null }));
    try {
      const response = await invoke<HybridSearchResponse>('hybrid_search', {
        query,
        topK,
      });
      setState(prev => ({
        ...prev,
        searching: false,
        response,
        apiHealthy: true,
        apiStatusMessage: 'Connected',
      }));
      return response;
    } catch (e) {
      const msg = String(e);
      setState(prev => ({
        ...prev,
        searching: false,
        error: msg,
        apiHealthy: msg.includes('unavailable') ? false : prev.apiHealthy,
        apiStatusMessage: msg.includes('unavailable') ? msg : prev.apiStatusMessage,
      }));
      return null;
    }
  }, []);

  const submitFeedback = useCallback(async (
    queryId: string,
    resultRank: number,
    rating: 'helpful' | 'not_helpful' | 'incorrect',
    comment?: string,
  ): Promise<boolean> => {
    try {
      await invoke('submit_search_feedback', {
        queryId,
        resultRank,
        rating,
        comment: comment ?? '',
      });
      return true;
    } catch (e) {
      console.error('Feedback submission failed:', e);
      return false;
    }
  }, []);

  const getStats = useCallback(async (): Promise<SearchApiStatsData | null> => {
    try {
      return await invoke<SearchApiStatsData>('get_search_api_stats');
    } catch (e) {
      console.error('Failed to get stats:', e);
      return null;
    }
  }, []);

  const checkHealth = useCallback(async (): Promise<boolean> => {
    try {
      const health = await invoke<SearchApiHealthStatus>('get_search_api_health_status');
      setState(prev => ({
        ...prev,
        apiHealthy: health.healthy,
        apiStatusMessage: health.message,
      }));
      return health.healthy;
    } catch {
      setState(prev => ({
        ...prev,
        apiHealthy: false,
        apiStatusMessage: 'Unable to check Search API health',
      }));
      return false;
    }
  }, []);

  const clearResults = useCallback(() => {
    setState(prev => ({ ...prev, response: null, error: null }));
  }, []);

  return {
    ...state,
    search,
    submitFeedback,
    getStats,
    checkHealth,
    clearResults,
  };
}
