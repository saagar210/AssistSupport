import { act, renderHook } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { useDraftActions } from './useDraftActions';

function createDraftRef() {
  return {
    current: {
      generate: vi.fn(),
      saveDraft: vi.fn(),
      copyResponse: vi.fn(),
      cancelGeneration: vi.fn(),
      exportResponse: vi.fn(),
      clearDraft: vi.fn(),
    },
  };
}

describe('useDraftActions', () => {
  it('executes draft actions only on draft tab', () => {
    const draftRef = createDraftRef();
    const { result, rerender } = renderHook(
      ({ activeTab }) => useDraftActions({ activeTab, draftRef }),
      { initialProps: { activeTab: 'draft' as const } },
    );

    act(() => {
      result.current.handleGenerate();
      result.current.handleSaveDraft();
      result.current.handleCopyResponse();
      result.current.handleCancelGeneration();
      result.current.handleExport();
      result.current.clearDraft();
    });

    expect(draftRef.current.generate).toHaveBeenCalledTimes(1);
    expect(draftRef.current.saveDraft).toHaveBeenCalledTimes(1);
    expect(draftRef.current.copyResponse).toHaveBeenCalledTimes(1);
    expect(draftRef.current.cancelGeneration).toHaveBeenCalledTimes(1);
    expect(draftRef.current.exportResponse).toHaveBeenCalledTimes(1);
    expect(draftRef.current.clearDraft).toHaveBeenCalledTimes(1);

    rerender({ activeTab: 'sources' });

    act(() => {
      result.current.handleGenerate();
      result.current.handleSaveDraft();
      result.current.handleCopyResponse();
      result.current.handleCancelGeneration();
      result.current.handleExport();
      result.current.clearDraft();
    });

    expect(draftRef.current.generate).toHaveBeenCalledTimes(1);
    expect(draftRef.current.saveDraft).toHaveBeenCalledTimes(1);
    expect(draftRef.current.copyResponse).toHaveBeenCalledTimes(1);
    expect(draftRef.current.cancelGeneration).toHaveBeenCalledTimes(1);
    expect(draftRef.current.exportResponse).toHaveBeenCalledTimes(1);
    expect(draftRef.current.clearDraft).toHaveBeenCalledTimes(2);
  });
});
