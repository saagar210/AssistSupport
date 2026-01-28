import { useState, useEffect, useRef, useCallback, useMemo } from 'react';
import { Sidebar, Header, TabBar } from './components/Layout';
import { DraftTab, DraftTabHandle } from './components/Draft/DraftTab';
import { FollowUpsTab } from './components/FollowUps/FollowUpsTab';
import { SourcesTab } from './components/Sources/SourcesTab';
import { IngestTab } from './components/Ingest/IngestTab';
import { KnowledgeBrowser } from './components/Knowledge';
import { SettingsTab } from './components/Settings/SettingsTab';
import { AnalyticsTab } from './components/Analytics/AnalyticsTab';
import { Toast, ToastContainer } from './components/shared/Toast';
import { ErrorBoundary } from './components/shared/ErrorBoundary';
import { Button } from './components/shared/Button';
import { CommandPalette, useCommandPalette, type Command } from './components/shared/CommandPalette';
import { KeyboardShortcuts, useKeyboardShortcutsHelp } from './components/shared/KeyboardShortcuts';
import { OnboardingWizard } from './components/shared/OnboardingWizard';
import { useInitialize } from './hooks/useInitialize';
import { useToastContext } from './contexts/ToastContext';
import { AppStatusProvider } from './contexts/AppStatusContext';
import { useKeyboardShortcuts } from './hooks/useKeyboard';
import type { SavedDraft } from './types';
import './App.css';

type TabId = 'draft' | 'followups' | 'sources' | 'ingest' | 'knowledge' | 'analytics' | 'settings';

function AppContent() {
  const { initResult, loading, error } = useInitialize();
  const { toasts, addToast, removeToast } = useToastContext();
  const [activeTab, setActiveTab] = useState<TabId>('draft');
  const [pendingDraft, setPendingDraft] = useState<SavedDraft | null>(null);
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);
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
      const tabs: TabId[] = ['draft', 'followups', 'sources', 'ingest', 'knowledge', 'analytics', 'settings'];
      if (n >= 1 && n <= 7) setActiveTab(tabs[n - 1]);
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
  const commands: Command[] = useMemo(() => [
    // Navigation commands
    {
      id: 'nav-draft',
      label: 'Go to Draft',
      description: 'Create and edit support responses',
      icon: 'draft',
      shortcut: 'Cmd+1',
      category: 'navigation',
      action: () => setActiveTab('draft'),
    },
    {
      id: 'nav-followups',
      label: 'Go to Follow-ups',
      description: 'View saved drafts and history',
      icon: 'followups',
      shortcut: 'Cmd+2',
      category: 'navigation',
      action: () => setActiveTab('followups'),
    },
    {
      id: 'nav-sources',
      label: 'Go to Sources',
      description: 'Search knowledge base',
      icon: 'sources',
      shortcut: 'Cmd+3',
      category: 'navigation',
      action: () => setActiveTab('sources'),
    },
    {
      id: 'nav-ingest',
      label: 'Go to Ingest',
      description: 'Add content to knowledge base',
      icon: 'ingest',
      shortcut: 'Cmd+4',
      category: 'navigation',
      action: () => setActiveTab('ingest'),
    },
    {
      id: 'nav-knowledge',
      label: 'Go to Knowledge',
      description: 'Browse indexed documents',
      icon: 'knowledge',
      shortcut: 'Cmd+5',
      category: 'navigation',
      action: () => setActiveTab('knowledge'),
    },
    {
      id: 'nav-analytics',
      label: 'Go to Analytics',
      description: 'View usage analytics and statistics',
      icon: 'sparkles',
      shortcut: 'Cmd+6',
      category: 'navigation',
      action: () => setActiveTab('analytics'),
    },
    {
      id: 'nav-settings',
      label: 'Go to Settings',
      description: 'Configure app preferences',
      icon: 'settings',
      shortcut: 'Cmd+7',
      category: 'navigation',
      action: () => setActiveTab('settings'),
    },
    // Quick actions
    {
      id: 'action-new-draft',
      label: 'New Draft',
      description: 'Clear current draft and start fresh',
      icon: 'plus',
      shortcut: 'Cmd+N',
      category: 'action',
      action: () => {
        setActiveTab('draft');
        // Clear draft via ref if available
        if (draftRef.current) {
          draftRef.current.clearDraft?.();
        }
      },
    },
    {
      id: 'action-focus-search',
      label: 'Focus Search',
      description: 'Jump to knowledge base search',
      icon: 'search',
      shortcut: 'Cmd+/',
      category: 'action',
      action: () => {
        setActiveTab('sources');
      },
    },
    // Draft actions
    {
      id: 'action-generate',
      label: 'Generate Response',
      description: 'Generate AI response for current draft',
      icon: 'sparkles',
      shortcut: 'Cmd+Enter',
      category: 'draft',
      action: handleGenerate,
      disabled: activeTab !== 'draft',
    },
    {
      id: 'action-save',
      label: 'Save Draft',
      description: 'Save current draft to history',
      icon: 'save',
      shortcut: 'Cmd+S',
      category: 'draft',
      action: handleSaveDraft,
      disabled: activeTab !== 'draft',
    },
    {
      id: 'action-copy',
      label: 'Copy Response',
      description: 'Copy generated response to clipboard',
      icon: 'copy',
      shortcut: 'Cmd+Shift+C',
      category: 'draft',
      action: handleCopyResponse,
      disabled: activeTab !== 'draft',
    },
    {
      id: 'action-export',
      label: 'Export Response',
      description: 'Export response as file',
      icon: 'download',
      shortcut: 'Cmd+E',
      category: 'draft',
      action: handleExport,
      disabled: activeTab !== 'draft',
    },
    {
      id: 'action-cancel',
      label: 'Cancel Generation',
      description: 'Stop current AI generation',
      icon: 'x',
      shortcut: 'Escape',
      category: 'draft',
      action: handleCancelGeneration,
      disabled: activeTab !== 'draft',
    },
    // Settings actions
    {
      id: 'settings-toggle-sidebar',
      label: sidebarCollapsed ? 'Expand Sidebar' : 'Collapse Sidebar',
      description: 'Toggle sidebar visibility',
      icon: sidebarCollapsed ? 'panelLeftOpen' : 'panelLeftClose',
      category: 'settings',
      action: handleToggleSidebar,
    },
    {
      id: 'settings-shortcuts',
      label: 'Keyboard Shortcuts',
      description: 'View all keyboard shortcuts',
      icon: 'command',
      shortcut: 'Cmd+Shift+/',
      category: 'settings',
      action: shortcutsHelp.open,
    },
    // Feature discovery commands
    {
      id: 'feature-templates',
      label: 'Open Templates',
      description: 'Use response templates for common scenarios',
      icon: 'draft',
      category: 'action',
      action: () => setActiveTab('draft'),
    },
    {
      id: 'feature-batch',
      label: 'Start Batch Processing',
      description: 'Process multiple queries at once',
      icon: 'list',
      category: 'action',
      action: () => setActiveTab('draft'),
    },
    {
      id: 'feature-voice',
      label: 'Start Voice Input',
      description: 'Use voice dictation for input',
      icon: 'play',
      category: 'action',
      action: () => setActiveTab('draft'),
    },
  ], [activeTab, sidebarCollapsed, handleGenerate, handleSaveDraft, handleCopyResponse, handleExport, handleCancelGeneration, handleToggleSidebar, shortcutsHelp.open]);

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
      case 'ingest':
        return (
          <ErrorBoundary fallbackTitle="Ingest tab encountered an error">
            <IngestTab />
          </ErrorBoundary>
        );
      case 'knowledge':
        return (
          <ErrorBoundary fallbackTitle="Knowledge tab encountered an error">
            <KnowledgeBrowser />
          </ErrorBoundary>
        );
      case 'analytics':
        return (
          <ErrorBoundary fallbackTitle="Analytics tab encountered an error">
            <AnalyticsTab />
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
          {renderTab()}
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
