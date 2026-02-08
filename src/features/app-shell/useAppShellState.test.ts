import { act, renderHook } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { SavedDraft } from '../../types';
import { useAppShellState } from './useAppShellState';

function makeDraftRef() {
  return {
    current: {
      loadDraft: vi.fn(),
    },
  };
}

const sampleDraft: SavedDraft = {
  id: 'd-1',
  input_text: 'input',
  summary_text: null,
  diagnosis_json: null,
  response_text: null,
  ticket_id: null,
  kb_sources_json: null,
  created_at: '2026-02-08T00:00:00Z',
  updated_at: '2026-02-08T00:00:00Z',
  is_autosave: false,
};

describe('useAppShellState', () => {
  beforeEach(() => {
    window.localStorage.clear();
  });

  it('opens onboarding on first run when not previously completed', () => {
    const draftRef = makeDraftRef();
    const addToast = vi.fn();

    const { result } = renderHook(() =>
      useAppShellState({
        initIsFirstRun: true,
        draftRef,
        addToast,
      }),
    );

    expect(result.current.showOnboarding).toBe(true);

    act(() => {
      result.current.handleOnboardingComplete();
    });

    expect(window.localStorage.getItem('onboarding-completed')).toBe('true');
    expect(addToast).toHaveBeenCalledWith(
      'Setup complete! Start drafting responses with AI assistance.',
      'success',
    );
    expect(result.current.showOnboarding).toBe(false);
  });

  it('persists sidebar collapse toggle and navigates to source tab', () => {
    const draftRef = makeDraftRef();

    const { result } = renderHook(() =>
      useAppShellState({
        initIsFirstRun: false,
        draftRef,
        addToast: vi.fn(),
      }),
    );

    expect(result.current.sidebarCollapsed).toBe(false);

    act(() => {
      result.current.handleToggleSidebar();
    });

    expect(result.current.sidebarCollapsed).toBe(true);
    expect(window.localStorage.getItem('sidebar-collapsed')).toBe('true');

    act(() => {
      result.current.handleNavigateToSource('vpn denied policy');
    });

    expect(result.current.activeTab).toBe('sources');
    expect(result.current.sourceSearchQuery).toBe('vpn denied policy');

    act(() => {
      result.current.consumeSourceSearchQuery();
    });

    expect(result.current.sourceSearchQuery).toBeNull();

    act(() => {
      result.current.handleNavigateToQueue('at_risk');
    });

    expect(result.current.activeTab).toBe('followups');
    expect(result.current.pendingQueueView).toBe('at_risk');

    act(() => {
      result.current.consumePendingQueueView();
    });

    expect(result.current.pendingQueueView).toBeNull();
  });

  it('loads follow-up draft via deferred draft-tab handoff', () => {
    const draftRef = makeDraftRef();

    const { result } = renderHook(() =>
      useAppShellState({
        initIsFirstRun: false,
        draftRef,
        addToast: vi.fn(),
      }),
    );

    act(() => {
      result.current.setActiveTab('followups');
    });

    act(() => {
      result.current.handleLoadDraft(sampleDraft);
    });

    expect(result.current.activeTab).toBe('draft');
    expect(draftRef.current.loadDraft).toHaveBeenCalledWith(sampleDraft);
  });
});
