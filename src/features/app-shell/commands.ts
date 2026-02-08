import type { Command } from '../../components/shared/CommandPalette';
import type { TabId } from './types';
import type { QueueView } from '../inbox/queueModel';

interface BuildCommandsParams {
  activeTab: TabId;
  sidebarCollapsed: boolean;
  revampCommandPaletteV2Enabled: boolean;
  setActiveTab: (tab: TabId) => void;
  openQueueView: (queueView: QueueView) => void;
  handleGenerate: () => void;
  handleSaveDraft: () => void;
  handleCopyResponse: () => void;
  handleExport: () => void;
  handleCancelGeneration: () => void;
  handleToggleSidebar: () => void;
  onOpenShortcuts: () => void;
  addToast: (message: string, type?: 'info' | 'success' | 'warning' | 'error') => void;
  clearDraft?: () => void;
}

export function buildAppShellCommands({
  activeTab,
  sidebarCollapsed,
  revampCommandPaletteV2Enabled,
  setActiveTab,
  openQueueView,
  handleGenerate,
  handleSaveDraft,
  handleCopyResponse,
  handleExport,
  handleCancelGeneration,
  handleToggleSidebar,
  onOpenShortcuts,
  addToast,
  clearDraft,
}: BuildCommandsParams): Command[] {
  const commands: Command[] = [
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
      id: 'nav-pilot',
      label: 'Go to Pilot',
      description: 'View pilot feedback dashboard',
      icon: 'sparkles',
      shortcut: 'Cmd+7',
      category: 'navigation',
      action: () => setActiveTab('pilot'),
    },
    {
      id: 'nav-search',
      label: 'Go to Search',
      description: 'Hybrid PostgreSQL search',
      icon: 'database',
      shortcut: 'Cmd+8',
      category: 'navigation',
      action: () => setActiveTab('search'),
    },
    {
      id: 'nav-ops',
      label: 'Go to Operations',
      description: 'Deployment, eval, triage, and runbooks',
      icon: 'terminal',
      shortcut: 'Cmd+9',
      category: 'navigation',
      action: () => setActiveTab('ops'),
    },
    {
      id: 'nav-settings',
      label: 'Go to Settings',
      description: 'Configure app preferences',
      icon: 'settings',
      shortcut: 'Cmd+0',
      category: 'navigation',
      action: () => setActiveTab('settings'),
    },
    {
      id: 'action-new-draft',
      label: 'New Draft',
      description: 'Clear current draft and start fresh',
      icon: 'plus',
      shortcut: 'Cmd+N',
      category: 'action',
      action: () => {
        setActiveTab('draft');
        clearDraft?.();
      },
    },
    {
      id: 'action-focus-search',
      label: 'Focus Search',
      description: 'Jump to knowledge base search',
      icon: 'search',
      shortcut: 'Cmd+/',
      category: 'action',
      action: () => setActiveTab('sources'),
    },
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
      action: onOpenShortcuts,
    },
    {
      id: 'feature-templates',
      label: 'Open Templates',
      description: 'Use response templates for common scenarios',
      icon: 'draft',
      category: 'action',
      action: () => {
        setActiveTab('draft');
        addToast('Templates quick-launch is not available yet. Use Save Draft/History for now.', 'info');
      },
    },
    {
      id: 'feature-batch',
      label: 'Start Batch Processing',
      description: 'Process multiple queries at once',
      icon: 'list',
      category: 'action',
      action: () => {
        setActiveTab('draft');
        addToast('Batch processing is planned but not available in this release.', 'info');
      },
    },
    {
      id: 'feature-voice',
      label: 'Start Voice Input',
      description: 'Use voice dictation for input',
      icon: 'play',
      category: 'action',
      action: () => {
        setActiveTab('draft');
        addToast('Voice input is planned but not available in this release.', 'info');
      },
    },
  ];

  if (revampCommandPaletteV2Enabled) {
    commands.push(
      {
        id: 'queue-open-unassigned',
        label: 'Queue: Unassigned',
        description: 'Jump to unassigned follow-ups queue',
        icon: 'followups',
        category: 'action',
        action: () => openQueueView('unassigned'),
      },
      {
        id: 'queue-open-at-risk',
        label: 'Queue: At Risk',
        description: 'Jump to SLA-at-risk follow-ups queue',
        icon: 'alert-triangle',
        category: 'action',
        action: () => openQueueView('at_risk'),
      },
      {
        id: 'queue-open-in-progress',
        label: 'Queue: In Progress',
        description: 'Jump to in-progress queue items',
        icon: 'play',
        category: 'action',
        action: () => openQueueView('in_progress'),
      },
      {
        id: 'queue-open-resolved',
        label: 'Queue: Resolved',
        description: 'Review resolved queue items',
        icon: 'check',
        category: 'action',
        action: () => openQueueView('resolved'),
      },
    );
  }

  return commands;
}
