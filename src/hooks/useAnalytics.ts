import { useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface AnalyticsSummary {
  total_events: number;
  responses_generated: number;
  searches_performed: number;
  drafts_saved: number;
  daily_counts: DailyCount[];
  average_rating: number;
  total_ratings: number;
}

export interface DailyCount {
  date: string;
  count: number;
}

export interface ArticleUsage {
  document_id: string;
  title: string;
  usage_count: number;
}

export function useAnalytics() {
  const logEvent = useCallback(async (eventType: string, eventData?: Record<string, unknown>): Promise<void> => {
    try {
      const id = crypto.randomUUID();
      await invoke('log_analytics_event', {
        id,
        eventType,
        eventDataJson: eventData ? JSON.stringify(eventData) : null,
      });
    } catch (err) {
      console.error('Failed to log analytics event:', err);
    }
  }, []);

  const getSummary = useCallback(async (periodDays?: number): Promise<AnalyticsSummary> => {
    return invoke<AnalyticsSummary>('get_analytics_summary', {
      periodDays: periodDays ?? null,
    });
  }, []);

  const getKbUsage = useCallback(async (periodDays?: number): Promise<ArticleUsage[]> => {
    return invoke<ArticleUsage[]>('get_kb_usage_stats', {
      periodDays: periodDays ?? null,
    });
  }, []);

  return { logEvent, getSummary, getKbUsage };
}
