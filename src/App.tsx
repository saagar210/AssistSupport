import { useEffect, useMemo, useRef } from 'react';
import { Sidebar, Header, TabBar } from './components/Layout';
import { type DraftTabHandle } from './components/Draft/DraftTab';
import { Toast, ToastContainer } from './components/shared/Toast';
import { Button } from './components/shared/Button';
import { CommandPalette, useCommandPalette } from './components/shared/CommandPalette';
import { KeyboardShortcuts, useKeyboardShortcutsHelp } from './components/shared/KeyboardShortcuts';
import { OnboardingWizard } from './components/shared/OnboardingWizard';
import { PassphraseUnlockScreen } from './components/shared/PassphraseUnlockScreen';
import { RecoveryScreen } from './components/shared/RecoveryScreen';
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
import { isTabEnabled } from './features/app-shell/tabPolicy';
import { getEnabledRevampFlags, resolveRevampFlags } from './features/revamp';
import { RevampShell } from './features/revamp/shell/RevampShell';
import './App.css';

function AppContent() {
  const { initResult, loading, error, unlockWithPassphrase } = useInitialize();
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
      if (tab && isTabEnabled(tab, revampFlags)) {
        setActiveTab(tab);
      }
    },
  });

  useEffect(() => {
    if (!isTabEnabled(activeTab, revampFlags)) {
      setActiveTab('draft');
    }
  }, [activeTab, revampFlags, setActiveTab]);

  const commands = useAppShellCommands({
    activeTab,
    sidebarCollapsed,
    revampCommandPaletteV2Enabled: revampFlags.ASSISTSUPPORT_REVAMP_COMMAND_PALETTE_V2,
    queueFirstInboxEnabled: revampFlags.ASSISTSUPPORT_REVAMP_INBOX,
    revampFlags,
    setActiveTab,
    openQueueView: handleNavigateToQueue,
    handleGenerate,
    handleSaveDraft,
    handleCopyResponse,
    handleExport,
    handleCancelGeneration,
    handleToggleSidebar,
    onOpenShortcuts: shortcutsHelp.open,
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
    if (initResult?.passphrase_required) {
      return (
        <PassphraseUnlockScreen
          error={error}
          onUnlock={unlockWithPassphrase}
        />
      );
    }

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

  if (initResult?.passphrase_required) {
    return (
      <PassphraseUnlockScreen
        error={null}
        onUnlock={unlockWithPassphrase}
      />
    );
  }

  if (initResult?.recovery_issue) {
    return <RecoveryScreen issue={initResult.recovery_issue} />;
  }

  const renderedTabContent = renderActiveTab({
    activeTab,
    draftRef,
    sourceSearchQuery,
    pendingQueueView,
    onSearchQueryConsumed: consumeSourceSearchQuery,
    onQueueViewConsumed: consumePendingQueueView,
    onNavigateToSource: handleNavigateToSource,
    onNavigateToQueue: handleNavigateToQueue,
    onLoadDraft: handleLoadDraft,
    revampFlags,
  });

  const activeTabContent = revampFlags.ASSISTSUPPORT_REVAMP_APP_SHELL ? (
    <div className="app-main">{renderedTabContent}</div>
  ) : (
    <main className="app-main">{renderedTabContent}</main>
  );

  const legacyLayout = (
    <div className="app">
      {/* Mobile navigation - visible only on small screens */}
      <div className="mobile-nav">
        <TabBar activeTab={activeTab} onTabChange={setActiveTab} revampFlags={revampFlags} />
      </div>

      {/* Desktop sidebar - hidden on small screens */}
      <Sidebar
        activeTab={activeTab}
        onTabChange={setActiveTab}
        collapsed={sidebarCollapsed}
        onToggleCollapse={handleToggleSidebar}
        revampFlags={revampFlags}
      />

      <div className="app-content">
        <Header
          activeTab={activeTab}
          onOpenCommandPalette={commandPalette.open}
          onOpenShortcuts={shortcutsHelp.open}
        />
        {activeTabContent}
      </div>
    </div>
  );

  const revampLayout = (
    <div className="app app-shell-revamp" data-revamp-shell="1">
      <RevampShell
        activeTab={activeTab}
        onTabChange={setActiveTab}
        revampFlags={revampFlags}
        onNavigateToQueue={handleNavigateToQueue}
        onOpenCommandPalette={commandPalette.open}
        onOpenShortcuts={shortcutsHelp.open}
      >
        {activeTabContent}
      </RevampShell>
    </div>
  );

  return (
    <>
      {revampFlags.ASSISTSUPPORT_REVAMP_APP_SHELL ? revampLayout : legacyLayout}

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
    </>
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
