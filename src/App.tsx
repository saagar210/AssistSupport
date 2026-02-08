import { useState, useEffect, useRef, useCallback, useMemo } from 'react';
import { Sidebar, Header, TabBar } from './components/Layout';
import { type DraftTabHandle } from './components/Draft/DraftTab';
import { Toast, ToastContainer } from './components/shared/Toast';
import { Button } from './components/shared/Button';
import { CommandPalette, useCommandPalette } from './components/shared/CommandPalette';
import { KeyboardShortcuts, useKeyboardShortcutsHelp } from './components/shared/KeyboardShortcuts';
import { OnboardingWizard } from './components/shared/OnboardingWizard';
import { useInitialize } from './hooks/useInitialize';
import { useToastContext } from './contexts/ToastContext';
import { AppStatusProvider } from './contexts/AppStatusContext';
import { useKeyboardShortcuts } from './hooks/useKeyboard';
import type { SavedDraft } from './types';
import {
  buildAppShellCommands,
  mapShortcutIndexToTab,
  renderActiveTab,
  type TabId,
} from './features/app-shell';
import './App.css';

function AppContent() {
  const { initResult, loading, error } = useInitialize();
  const { toasts, addToast, removeToast } = useToastContext();
  const [activeTab, setActiveTab] = useState<TabId>('draft');
  const [pendingDraft, setPendingDraft] = useState<SavedDraft | null>(null);
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);
  const [sourceSearchQuery, setSourceSearchQuery] = useState<string | null>(null);
  const [showOnboarding, setShowOnboarding] = useState(false);
  const draftRef = useRef<DraftTabHandle>(null);
  const commandPalette = useCommandPalette();
  const shortcutsHelp = useKeyboardShortcutsHelp();

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

  const handleNavigateToSource = useCallback((searchQuery: string) => {
    setSourceSearchQuery(searchQuery);
    setActiveTab('sources');
  }, []);

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
      const tab = mapShortcutIndexToTab(n);
      if (tab) {
        setActiveTab(tab);
      }
    },
  });

  // Show onboarding on first run (check localStorage to not show again after completion)
  useEffect(() => {
    if (initResult?.is_first_run) {
      const hasCompletedOnboarding = localStorage.getItem('onboarding-completed');
      if (!hasCompletedOnboarding) {
        setShowOnboarding(true);
      }
    }
  }, [initResult?.is_first_run]);

  const handleOnboardingComplete = useCallback(() => {
    localStorage.setItem('onboarding-completed', 'true');
    setShowOnboarding(false);
    addToast('Setup complete! Start drafting responses with AI assistance.', 'success');
  }, [addToast]);

  const handleOnboardingSkip = useCallback(() => {
    localStorage.setItem('onboarding-completed', 'true');
    setShowOnboarding(false);
    addToast('You can configure settings anytime from the Settings tab.', 'info');
  }, [addToast]);

  // Persist sidebar state
  useEffect(() => {
    const saved = localStorage.getItem('sidebar-collapsed');
    if (saved !== null) {
      setSidebarCollapsed(saved === 'true');
    }
  }, []);

  const handleToggleSidebar = useCallback(() => {
    setSidebarCollapsed(prev => {
      const next = !prev;
      localStorage.setItem('sidebar-collapsed', String(next));
      return next;
    });
  }, []);

  // Command palette commands
  const commands = useMemo(() => buildAppShellCommands({
    activeTab,
    sidebarCollapsed,
    setActiveTab,
    handleGenerate,
    handleSaveDraft,
    handleCopyResponse,
    handleExport,
    handleCancelGeneration,
    handleToggleSidebar,
    onOpenShortcuts: shortcutsHelp.open,
    addToast,
    clearDraft: () => draftRef.current?.clearDraft?.(),
  }), [
    activeTab,
    sidebarCollapsed,
    setActiveTab,
    handleGenerate,
    handleSaveDraft,
    handleCopyResponse,
    handleExport,
    handleCancelGeneration,
    handleToggleSidebar,
    shortcutsHelp.open,
    addToast,
  ]);

  if (loading) {
    return (
      <div className="app-loading">
        <div className="loading-spinner" />
        <p className="app-loading-text">Initializing AssistSupport...</p>
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

  return (
    <div className="app">
      {/* Mobile navigation - visible only on small screens */}
      <div className="mobile-nav">
        <TabBar activeTab={activeTab} onTabChange={setActiveTab} />
      </div>

      {/* Desktop sidebar - hidden on small screens */}
      <Sidebar
        activeTab={activeTab}
        onTabChange={setActiveTab}
        collapsed={sidebarCollapsed}
        onToggleCollapse={handleToggleSidebar}
      />

      <div className="app-content">
        <Header
          activeTab={activeTab}
          onOpenCommandPalette={commandPalette.open}
          onOpenShortcuts={shortcutsHelp.open}
        />
        <main className="app-main">
          {renderActiveTab({
            activeTab,
            draftRef,
            sourceSearchQuery,
            onSearchQueryConsumed: () => setSourceSearchQuery(null),
            onNavigateToSource: handleNavigateToSource,
            onLoadDraft: handleLoadDraft,
          })}
        </main>
      </div>

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

      <CommandPalette
        isOpen={commandPalette.isOpen}
        onClose={commandPalette.close}
        commands={commands}
      />

      <KeyboardShortcuts
        isOpen={shortcutsHelp.isOpen}
        onClose={shortcutsHelp.close}
      />

      {showOnboarding && (
        <OnboardingWizard
          onComplete={handleOnboardingComplete}
          onSkip={handleOnboardingSkip}
        />
      )}
    </div>
  );
}

// Main App wrapper with providers
function App() {
  return (
    <AppStatusProvider pollInterval={10000}>
      <AppContent />
    </AppStatusProvider>
  );
}

export default App;
