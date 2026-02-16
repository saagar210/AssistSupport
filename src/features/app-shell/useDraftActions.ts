import { useCallback } from 'react';
import type { RefObject } from 'react';
import type { DraftTabHandle } from '../../components/Draft/DraftTab';
import type { TabId } from './types';

interface UseDraftActionsParams {
  activeTab: TabId;
  draftRef: RefObject<DraftTabHandle | null>;
}

export function useDraftActions({ activeTab, draftRef }: UseDraftActionsParams) {
  const handleGenerate = useCallback(() => {
    if (activeTab === 'draft') {
      draftRef.current?.generate();
    }
  }, [activeTab, draftRef]);

  const handleSaveDraft = useCallback(() => {
    if (activeTab === 'draft') {
      draftRef.current?.saveDraft();
    }
  }, [activeTab, draftRef]);

  const handleCopyResponse = useCallback(() => {
    if (activeTab === 'draft') {
      draftRef.current?.copyResponse();
    }
  }, [activeTab, draftRef]);

  const handleCancelGeneration = useCallback(() => {
    if (activeTab === 'draft') {
      draftRef.current?.cancelGeneration();
    }
  }, [activeTab, draftRef]);

  const handleExport = useCallback(() => {
    if (activeTab === 'draft') {
      draftRef.current?.exportResponse();
    }
  }, [activeTab, draftRef]);

  const clearDraft = useCallback(() => {
    draftRef.current?.clearDraft?.();
  }, [draftRef]);

  return {
    handleGenerate,
    handleSaveDraft,
    handleCopyResponse,
    handleCancelGeneration,
    handleExport,
    clearDraft,
  };
}
