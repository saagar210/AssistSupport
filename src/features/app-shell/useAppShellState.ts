import { useCallback, useEffect, useState } from 'react';
import type { RefObject } from 'react';
import type { SavedDraft } from '../../types';
import type { DraftTabHandle } from '../../components/Draft/DraftTab';
import type { TabId } from './types';
import type { QueueView } from '../inbox/queueModel';

interface UseAppShellStateParams {
  initIsFirstRun: boolean | undefined;
  draftRef: RefObject<DraftTabHandle | null>;
  addToast: (message: string, type?: 'info' | 'success' | 'warning' | 'error') => void;
}

const ONBOARDING_COMPLETED_KEY = 'onboarding-completed';
const SIDEBAR_COLLAPSED_KEY = 'sidebar-collapsed';

export function useAppShellState({ initIsFirstRun, draftRef, addToast }: UseAppShellStateParams) {
  const [activeTab, setActiveTab] = useState<TabId>('draft');
  const [pendingDraft, setPendingDraft] = useState<SavedDraft | null>(null);
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);
  const [sourceSearchQuery, setSourceSearchQuery] = useState<string | null>(null);
  const [pendingQueueView, setPendingQueueView] = useState<QueueView | null>(null);
  const [showOnboarding, setShowOnboarding] = useState(false);

  const handleNavigateToSource = useCallback((searchQuery: string) => {
    setSourceSearchQuery(searchQuery);
    setActiveTab('sources');
  }, []);

  const consumeSourceSearchQuery = useCallback(() => {
    setSourceSearchQuery(null);
  }, []);

  const handleNavigateToQueue = useCallback((queueView: QueueView) => {
    setPendingQueueView(queueView);
    setActiveTab('followups');
  }, []);

  const consumePendingQueueView = useCallback(() => {
    setPendingQueueView(null);
  }, []);

  const handleLoadDraft = useCallback((draft: SavedDraft) => {
    if (activeTab === 'draft' && draftRef.current) {
      draftRef.current.loadDraft(draft);
      return;
    }

    setPendingDraft(draft);
    setActiveTab('draft');
  }, [activeTab, draftRef]);

  useEffect(() => {
    if (activeTab === 'draft' && pendingDraft && draftRef.current) {
      draftRef.current.loadDraft(pendingDraft);
      setPendingDraft(null);
    }
  }, [activeTab, pendingDraft, draftRef]);

  useEffect(() => {
    if (!initIsFirstRun) {
      return;
    }

    const hasCompletedOnboarding = localStorage.getItem(ONBOARDING_COMPLETED_KEY);
    if (!hasCompletedOnboarding) {
      setShowOnboarding(true);
    }
  }, [initIsFirstRun]);

  const handleOnboardingComplete = useCallback(() => {
    localStorage.setItem(ONBOARDING_COMPLETED_KEY, 'true');
    setShowOnboarding(false);
    addToast('Setup complete! Start drafting responses with AI assistance.', 'success');
  }, [addToast]);

  const handleOnboardingSkip = useCallback(() => {
    localStorage.setItem(ONBOARDING_COMPLETED_KEY, 'true');
    setShowOnboarding(false);
    addToast('You can configure settings anytime from the Settings tab.', 'info');
  }, [addToast]);

  useEffect(() => {
    const saved = localStorage.getItem(SIDEBAR_COLLAPSED_KEY);
    if (saved !== null) {
      setSidebarCollapsed(saved === 'true');
    }
  }, []);

  const handleToggleSidebar = useCallback(() => {
    setSidebarCollapsed((prev) => {
      const next = !prev;
      localStorage.setItem(SIDEBAR_COLLAPSED_KEY, String(next));
      return next;
    });
  }, []);

  return {
    activeTab,
    setActiveTab,
    sidebarCollapsed,
    sourceSearchQuery,
    pendingQueueView,
    showOnboarding,
    handleNavigateToSource,
    handleNavigateToQueue,
    consumeSourceSearchQuery,
    consumePendingQueueView,
    handleLoadDraft,
    handleOnboardingComplete,
    handleOnboardingSkip,
    handleToggleSidebar,
  };
}
