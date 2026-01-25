import { useState, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { SavedDraft, ResponseTemplate } from '../types';

const AUTOSAVE_DEBOUNCE_MS = 5000;
const AUTOSAVE_KEEP_COUNT = 10;

export function useDrafts() {
  const [drafts, setDrafts] = useState<SavedDraft[]>([]);
  const [autosaves, setAutosaves] = useState<SavedDraft[]>([]);
  const [templates, setTemplates] = useState<ResponseTemplate[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Autosave debounce timer ref
  const autosaveTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const loadDrafts = useCallback(async (limit?: number) => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<SavedDraft[]>('list_drafts', { limit: limit ?? 50 });
      setDrafts(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  const searchDrafts = useCallback(async (query: string, limit?: number): Promise<SavedDraft[]> => {
    if (!query.trim()) {
      return drafts;
    }
    try {
      const result = await invoke<SavedDraft[]>('search_drafts', {
        query: query.trim(),
        limit: limit ?? 50
      });
      return result;
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      return [];
    }
  }, [drafts]);

  const loadTemplates = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<ResponseTemplate[]>('list_templates');
      setTemplates(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  const getDraft = useCallback(async (draftId: string): Promise<SavedDraft | null> => {
    try {
      return await invoke<SavedDraft>('get_draft', { draftId });
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      return null;
    }
  }, []);

  const saveDraft = useCallback(async (draft: Omit<SavedDraft, 'id' | 'created_at' | 'updated_at'>): Promise<string | null> => {
    try {
      const now = new Date().toISOString();
      const fullDraft: SavedDraft = {
        ...draft,
        id: crypto.randomUUID(),
        created_at: now,
        updated_at: now,
      };
      const id = await invoke<string>('save_draft', { draft: fullDraft });
      await loadDrafts();
      return id;
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      return null;
    }
  }, [loadDrafts]);

  const updateDraft = useCallback(async (draft: SavedDraft): Promise<string | null> => {
    try {
      const updated: SavedDraft = {
        ...draft,
        updated_at: new Date().toISOString(),
      };
      const id = await invoke<string>('save_draft', { draft: updated });
      await loadDrafts();
      return id;
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      return null;
    }
  }, [loadDrafts]);

  const deleteDraft = useCallback(async (draftId: string): Promise<boolean> => {
    try {
      await invoke('delete_draft', { draftId });
      await loadDrafts();
      return true;
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      return false;
    }
  }, [loadDrafts]);

  const loadAutosaves = useCallback(async (limit?: number) => {
    try {
      const result = await invoke<SavedDraft[]>('list_autosaves', { limit: limit ?? AUTOSAVE_KEEP_COUNT });
      setAutosaves(result);
    } catch (err) {
      console.error('Failed to load autosaves:', err);
    }
  }, []);

  const cleanupAutosaves = useCallback(async (keepCount?: number) => {
    try {
      await invoke('cleanup_autosaves', { keepCount: keepCount ?? AUTOSAVE_KEEP_COUNT });
    } catch (err) {
      console.error('Failed to cleanup autosaves:', err);
    }
  }, []);

  const triggerAutosave = useCallback((draftData: Omit<SavedDraft, 'id' | 'created_at' | 'updated_at' | 'is_autosave'>) => {
    // Cancel any pending autosave
    if (autosaveTimeoutRef.current) {
      clearTimeout(autosaveTimeoutRef.current);
    }

    // Skip if no meaningful content
    if (!draftData.input_text?.trim()) {
      return;
    }

    // Debounce the autosave
    autosaveTimeoutRef.current = setTimeout(async () => {
      try {
        const now = new Date().toISOString();
        const fullDraft: SavedDraft = {
          ...draftData,
          id: crypto.randomUUID(),
          created_at: now,
          updated_at: now,
          is_autosave: true,
        };
        await invoke<string>('save_draft', { draft: fullDraft });
        await cleanupAutosaves(AUTOSAVE_KEEP_COUNT);
        await loadAutosaves();
      } catch (err) {
        console.error('Autosave failed:', err);
      }
    }, AUTOSAVE_DEBOUNCE_MS);
  }, [cleanupAutosaves, loadAutosaves]);

  const cancelAutosave = useCallback(() => {
    if (autosaveTimeoutRef.current) {
      clearTimeout(autosaveTimeoutRef.current);
      autosaveTimeoutRef.current = null;
    }
  }, []);

  const getTemplate = useCallback(async (templateId: string): Promise<ResponseTemplate | null> => {
    try {
      return await invoke<ResponseTemplate>('get_template', { templateId });
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      return null;
    }
  }, []);

  const saveTemplate = useCallback(async (template: Omit<ResponseTemplate, 'id' | 'created_at' | 'updated_at'>): Promise<string | null> => {
    try {
      const now = new Date().toISOString();
      const fullTemplate: ResponseTemplate = {
        ...template,
        id: crypto.randomUUID(),
        created_at: now,
        updated_at: now,
      };
      const id = await invoke<string>('save_template', { template: fullTemplate });
      await loadTemplates();
      return id;
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      return null;
    }
  }, [loadTemplates]);

  const updateTemplate = useCallback(async (template: ResponseTemplate): Promise<string | null> => {
    try {
      const updated: ResponseTemplate = {
        ...template,
        updated_at: new Date().toISOString(),
      };
      const id = await invoke<string>('save_template', { template: updated });
      await loadTemplates();
      return id;
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      return null;
    }
  }, [loadTemplates]);

  const deleteTemplate = useCallback(async (templateId: string): Promise<boolean> => {
    try {
      await invoke('delete_template', { templateId });
      await loadTemplates();
      return true;
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      return false;
    }
  }, [loadTemplates]);

  return {
    drafts,
    autosaves,
    templates,
    loading,
    error,
    loadDrafts,
    searchDrafts,
    loadAutosaves,
    loadTemplates,
    getDraft,
    saveDraft,
    updateDraft,
    deleteDraft,
    triggerAutosave,
    cancelAutosave,
    cleanupAutosaves,
    getTemplate,
    saveTemplate,
    updateTemplate,
    deleteTemplate,
  };
}
