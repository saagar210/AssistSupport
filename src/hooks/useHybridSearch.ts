import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type {
  HybridSearchResponse,
  SearchApiHealthStatus,
  SearchApiStatsData,
} from '../types';
import {
  classifySearchApiFailure,
  classifySearchApiHealthStatus,
} from '../features/integrations/degradedSemantics';

export interface HybridSearchState {
  response: HybridSearchResponse | null;
  searching: boolean;
  error: string | null;
  apiHealthy: boolean | null;
  apiStatusMessage: string | null;
  degradedReason: string | null;
  degradedGuidance: string | null;
}

export function useHybridSearch() {
  const [state, setState] = useState<HybridSearchState>({
    response: null,
    searching: false,
    error: null,
    apiHealthy: null,
    apiStatusMessage: null,
    degradedReason: null,
    degradedGuidance: null,
  });

  const search = useCallback(async (query: string, topK = 10): Promise<HybridSearchResponse | null> => {
    setState(prev => ({
      ...prev,
      searching: true,
      error: null,
      degradedReason: null,
      degradedGuidance: null,
    }));
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
        degradedReason: null,
        degradedGuidance: null,
      }));
      return response;
    } catch (e) {
      const msg = String(e);
      const classified = classifySearchApiFailure(msg);
      setState(prev => ({
        ...prev,
        searching: false,
        error: msg,
        apiHealthy: classified.status === 'offline' ? false : prev.apiHealthy,
        apiStatusMessage: classified.guidance,
        degradedReason: classified.reason,
        degradedGuidance: classified.guidance,
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
      const classified = classifySearchApiHealthStatus(health.status, health.message);
      setState(prev => ({
        ...prev,
        apiHealthy: health.healthy,
        apiStatusMessage: health.healthy ? health.message : classified.guidance,
        degradedReason: health.healthy ? null : classified.reason,
        degradedGuidance: health.healthy ? null : classified.guidance,
      }));
      return health.healthy;
    } catch {
      setState(prev => ({
        ...prev,
        apiHealthy: false,
        apiStatusMessage: 'Unable to check Search API health',
        degradedReason: 'offline',
        degradedGuidance: 'Search API health check failed. Confirm local service is running.',
      }));
      return false;
    }
  }, []);

  const clearResults = useCallback(() => {
    setState(prev => ({
      ...prev,
      response: null,
      error: null,
      degradedReason: null,
      degradedGuidance: null,
    }));
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
