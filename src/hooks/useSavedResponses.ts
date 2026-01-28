import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { SavedResponseTemplate } from '../types';

export function useSavedResponses() {
  const [templates, setTemplates] = useState<SavedResponseTemplate[]>([]);
  const [suggestions, setSuggestions] = useState<SavedResponseTemplate[]>([]);
  const [loading, setLoading] = useState(false);

  const loadTemplates = useCallback(async (limit?: number) => {
    setLoading(true);
    try {
      const data = await invoke<SavedResponseTemplate[]>('list_saved_response_templates', {
        limit: limit ?? 20,
      });
      setTemplates(data);
      return data;
    } catch (err) {
      console.error('Failed to load saved templates:', err);
      return [];
    } finally {
      setLoading(false);
    }
  }, []);

  const saveAsTemplate = useCallback(async (
    name: string,
    content: string,
    opts?: {
      sourceDraftId?: string;
      sourceRating?: number;
      category?: string;
      variablesJson?: string;
    }
  ): Promise<string | null> => {
    try {
      const id = await invoke<string>('save_response_as_template', {
        sourceDraftId: opts?.sourceDraftId ?? null,
        sourceRating: opts?.sourceRating ?? null,
        name,
        category: opts?.category ?? null,
        content,
        variablesJson: opts?.variablesJson ?? null,
      });
      await loadTemplates();
      return id;
    } catch (err) {
      console.error('Failed to save template:', err);
      return null;
    }
  }, [loadTemplates]);

  const incrementUsage = useCallback(async (templateId: string) => {
    try {
      await invoke('increment_saved_template_usage', { templateId });
    } catch (err) {
      console.error('Failed to increment template usage:', err);
    }
  }, []);

  const findSimilar = useCallback(async (inputText: string, limit?: number) => {
    if (!inputText.trim() || inputText.trim().length < 10) {
      setSuggestions([]);
      return [];
    }
    try {
      const data = await invoke<SavedResponseTemplate[]>('find_similar_saved_responses', {
        inputText,
        limit: limit ?? 3,
      });
      setSuggestions(data);
      return data;
    } catch (err) {
      console.error('Failed to find similar responses:', err);
      setSuggestions([]);
      return [];
    }
  }, []);

  return {
    templates,
    suggestions,
    loading,
    loadTemplates,
    saveAsTemplate,
    incrementUsage,
    findSimilar,
  };
}
