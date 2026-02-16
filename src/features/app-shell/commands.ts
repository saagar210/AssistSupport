import type { Command } from '../../components/shared/CommandPalette';
import type { TabId } from './types';
import type { QueueView } from '../inbox/queueModel';
import type { RevampFlags } from '../revamp';
import { isTabEnabled } from './tabPolicy';

interface BuildCommandsParams {
  activeTab: TabId;
  sidebarCollapsed: boolean;
  revampCommandPaletteV2Enabled: boolean;
  queueFirstInboxEnabled: boolean;
  revampFlags: RevampFlags;
  setActiveTab: (tab: TabId) => void;
  openQueueView: (queueView: QueueView) => void;
  handleGenerate: () => void;
  handleSaveDraft: () => void;
  handleCopyResponse: () => void;
  handleExport: () => void;
  handleCancelGeneration: () => void;
  handleToggleSidebar: () => void;
  onOpenShortcuts: () => void;
  clearDraft?: () => void;
}

export function buildAppShellCommands({
  activeTab,
  sidebarCollapsed,
  revampCommandPaletteV2Enabled,
  queueFirstInboxEnabled,
  revampFlags,
  setActiveTab,
  openQueueView,
  handleGenerate,
  handleSaveDraft,
  handleCopyResponse,
  handleExport,
  handleCancelGeneration,
  handleToggleSidebar,
  onOpenShortcuts,
  clearDraft,
}: BuildCommandsParams): Command[] {
  const makeNavCommand = (
    tab: TabId,
    payload: Omit<Command, 'action' | 'disabled'> & { action?: Command['action'] },
  ): Command | null => {
    if (!isTabEnabled(tab, revampFlags)) {
      return null;
    }
    return {
      ...payload,
      action: () => setActiveTab(tab),
    };
  };

  const commands: Array<Command | null> = [
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
    makeNavCommand('ingest', {
      id: 'nav-ingest',
      label: 'Go to Ingest',
      description: 'Add content to knowledge base (network ingest)',
      icon: 'ingest',
      shortcut: 'Cmd+4',
      category: 'navigation',
    }),
    {
      id: 'nav-knowledge',
      label: 'Go to Knowledge',
      description: 'Browse indexed documents',
      icon: 'knowledge',
      shortcut: 'Cmd+5',
      category: 'navigation',
      action: () => setActiveTab('knowledge'),
    },
    makeNavCommand('analytics', {
      id: 'nav-analytics',
      label: 'Go to Analytics',
      description: 'View usage analytics and statistics',
      icon: 'sparkles',
      shortcut: 'Cmd+6',
      category: 'navigation',
    }),
    makeNavCommand('pilot', {
      id: 'nav-pilot',
      label: 'Go to Pilot',
      description: 'View pilot feedback dashboard',
      icon: 'sparkles',
      shortcut: 'Cmd+7',
      category: 'navigation',
    }),
    makeNavCommand('search', {
      id: 'nav-search',
      label: 'Go to Search',
      description: 'Hybrid PostgreSQL search',
      icon: 'database',
      shortcut: 'Cmd+8',
      category: 'navigation',
    }),
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
  ];

  // Filter out nulls from conditional nav commands.
  const filtered: Command[] = commands.filter((cmd): cmd is Command => Boolean(cmd));

  if (revampCommandPaletteV2Enabled) {
    filtered.push(
      {
        id: 'queue-open-unassigned',
        label: 'Queue: Unassigned',
        description: 'Jump to unassigned follow-ups queue',
        icon: 'followups',
        category: 'action',
        action: () => openQueueView('unassigned'),
        disabled: !queueFirstInboxEnabled,
      },
      {
        id: 'queue-open-at-risk',
        label: 'Queue: At Risk',
        description: 'Jump to SLA-at-risk follow-ups queue',
        icon: 'alert-triangle',
        category: 'action',
        action: () => openQueueView('at_risk'),
        disabled: !queueFirstInboxEnabled,
      },
      {
        id: 'queue-open-in-progress',
        label: 'Queue: In Progress',
        description: 'Jump to in-progress queue items',
        icon: 'play',
        category: 'action',
        action: () => openQueueView('in_progress'),
        disabled: !queueFirstInboxEnabled,
      },
      {
        id: 'queue-open-resolved',
        label: 'Queue: Resolved',
        description: 'Review resolved queue items',
        icon: 'check',
        category: 'action',
        action: () => openQueueView('resolved'),
        disabled: !queueFirstInboxEnabled,
      },
    );
  }

  return filtered;
}
