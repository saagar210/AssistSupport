import { useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface ResponseRating {
  id: string;
  draft_id: string;
  rating: number;
  feedback_text: string | null;
  feedback_category: string | null;
  created_at: string;
}

export interface RatingStats {
  total_ratings: number;
  average_rating: number;
  distribution: number[]; // counts for ratings 1-5
}

export function useRatings() {
  const rateResponse = useCallback(async (draftId: string, rating: number, feedbackText?: string, feedbackCategory?: string): Promise<void> => {
    const id = crypto.randomUUID();
    await invoke('rate_response', {
      id,
      draftId,
      rating,
      feedbackText: feedbackText || null,
      feedbackCategory: feedbackCategory || null,
    });
  }, []);

  const getDraftRating = useCallback(async (draftId: string): Promise<ResponseRating | null> => {
    return invoke<ResponseRating | null>('get_draft_rating', { draftId });
  }, []);

  const getRatingStats = useCallback(async (): Promise<RatingStats> => {
    return invoke<RatingStats>('get_rating_stats');
  }, []);

  return { rateResponse, getDraftRating, getRatingStats };
}
