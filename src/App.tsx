import { useState, useEffect, useRef, useCallback } from 'react';
import { TabBar } from './components/Layout/TabBar';
import { DraftTab, DraftTabHandle } from './components/Draft/DraftTab';
import { FollowUpsTab } from './components/FollowUps/FollowUpsTab';
import { SourcesTab } from './components/Sources/SourcesTab';
import { SettingsTab } from './components/Settings/SettingsTab';
import { Toast, ToastContainer } from './components/shared/Toast';
import { ErrorBoundary } from './components/shared/ErrorBoundary';
import { Button } from './components/shared/Button';
import { useInitialize } from './hooks/useInitialize';
import { useToastContext } from './contexts/ToastContext';
import { useKeyboardShortcuts } from './hooks/useKeyboard';
import type { SavedDraft } from './types';
import './App.css';

type TabId = 'draft' | 'followups' | 'sources' | 'settings';

function App() {
  const { initResult, loading, error } = useInitialize();
  const { toasts, addToast, removeToast } = useToastContext();
  const [activeTab, setActiveTab] = useState<TabId>('draft');
  const [pendingDraft, setPendingDraft] = useState<SavedDraft | null>(null);
  const draftRef = useRef<DraftTabHandle>(null);

  const handleGenerate = useCallback(() => {
    if (activeTab === 'draft') {
      draftRef.current?.generate();
    }
  }, [activeTab]);

  const handleSaveDraft = useCallback(() => {
    if (activeTab === 'draft') {
      draftRef.current?.saveDraft();
    }
  }, [activeTab]);

  const handleCopyResponse = useCallback(() => {
    if (activeTab === 'draft') {
      draftRef.current?.copyResponse();
    }
  }, [activeTab]);

  const handleCancelGeneration = useCallback(() => {
    if (activeTab === 'draft') {
      draftRef.current?.cancelGeneration();
    }
  }, [activeTab]);

  const handleExport = useCallback(() => {
    if (activeTab === 'draft') {
      draftRef.current?.exportResponse();
    }
  }, [activeTab]);

  const handleLoadDraft = useCallback((draft: SavedDraft) => {
    // If already on draft tab, load directly via ref
    if (activeTab === 'draft' && draftRef.current) {
      draftRef.current.loadDraft(draft);
    } else {
      // Set pending draft and switch to draft tab
      setPendingDraft(draft);
      setActiveTab('draft');
    }
  }, [activeTab]);

  // Clear pending draft when DraftTab mounts and loads it
  useEffect(() => {
    if (activeTab === 'draft' && pendingDraft && draftRef.current) {
      draftRef.current.loadDraft(pendingDraft);
      setPendingDraft(null);
    }
  }, [activeTab, pendingDraft]);

  useKeyboardShortcuts({
    onGenerate: handleGenerate,
    onSaveDraft: handleSaveDraft,
    onCopyResponse: handleCopyResponse,
    onCancelGeneration: handleCancelGeneration,
    onExport: handleExport,
    onSwitchTab: (n) => {
      const tabs: TabId[] = ['draft', 'followups', 'sources', 'settings'];
      if (n >= 1 && n <= 4) setActiveTab(tabs[n - 1]);
    },
  });

  useEffect(() => {
    if (initResult?.is_first_run) {
      addToast('Welcome to AssistSupport! Configure your settings to get started.', 'info');
    }
  }, [initResult?.is_first_run, addToast]);

  if (loading) {
    return (
      <div className="app-loading">
        <div className="loading-spinner" />
        <p>Initializing AssistSupport...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="app-error">
        <h1>Initialization Error</h1>
        <pre>{error}</pre>
        <p className="error-hint">
          Try restarting the application. If the problem persists, check the console for details.
        </p>
        <Button variant="primary" onClick={() => window.location.reload()}>
          Retry
        </Button>
      </div>
    );
  }

  function renderTab() {
    switch (activeTab) {
      case 'draft':
        return (
          <ErrorBoundary fallbackTitle="Draft tab encountered an error">
            <DraftTab ref={draftRef} />
          </ErrorBoundary>
        );
      case 'followups':
        return (
          <ErrorBoundary fallbackTitle="Follow-ups tab encountered an error">
            <FollowUpsTab onLoadDraft={handleLoadDraft} />
          </ErrorBoundary>
        );
      case 'sources':
        return (
          <ErrorBoundary fallbackTitle="Sources tab encountered an error">
            <SourcesTab />
          </ErrorBoundary>
        );
      case 'settings':
        return (
          <ErrorBoundary fallbackTitle="Settings tab encountered an error">
            <SettingsTab />
          </ErrorBoundary>
        );
    }
  }

  return (
    <div className="app">
      <TabBar activeTab={activeTab} onTabChange={setActiveTab} />
      <main className="app-main">
        {renderTab()}
      </main>
      <ToastContainer>
        {toasts.map(toast => (
          <Toast
            key={toast.id}
            message={toast.message}
            type={toast.type}
            onClose={() => removeToast(toast.id)}
          />
        ))}
      </ToastContainer>
    </div>
  );
}

export default App;
