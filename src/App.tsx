import { useMemo, useRef } from 'react';
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
import {
  mapShortcutIndexToTab,
  renderActiveTab,
  useAppShellCommands,
  useAppShellState,
  useDraftActions,
} from './features/app-shell';
import { getEnabledRevampFlags, resolveRevampFlags } from './features/revamp';
import './App.css';

function AppContent() {
  const { initResult, loading, error } = useInitialize();
  const { toasts, addToast, removeToast } = useToastContext();
  const draftRef = useRef<DraftTabHandle>(null);
  const commandPalette = useCommandPalette();
  const shortcutsHelp = useKeyboardShortcutsHelp();
  const revampFlags = useMemo(() => resolveRevampFlags(), []);
  const revampEnabled = useMemo(() => getEnabledRevampFlags(revampFlags).length > 0, [revampFlags]);

  const {
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
  } = useAppShellState({
    initIsFirstRun: initResult?.is_first_run,
    draftRef,
    addToast,
  });

  const {
    handleGenerate,
    handleSaveDraft,
    handleCopyResponse,
    handleCancelGeneration,
    handleExport,
    clearDraft,
  } = useDraftActions({ activeTab, draftRef });

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

  const commands = useAppShellCommands({
    activeTab,
    sidebarCollapsed,
    revampCommandPaletteV2Enabled: revampFlags.ASSISTSUPPORT_REVAMP_COMMAND_PALETTE_V2,
    setActiveTab,
    openQueueView: handleNavigateToQueue,
    handleGenerate,
    handleSaveDraft,
    handleCopyResponse,
    handleExport,
    handleCancelGeneration,
    handleToggleSidebar,
    onOpenShortcuts: shortcutsHelp.open,
    addToast,
    clearDraft,
  });

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
            pendingQueueView,
            onSearchQueryConsumed: consumeSourceSearchQuery,
            onQueueViewConsumed: consumePendingQueueView,
            onNavigateToSource: handleNavigateToSource,
            onLoadDraft: handleLoadDraft,
            revampFlags,
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
        subtitle={revampEnabled ? 'Revamp preview mode active' : undefined}
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
