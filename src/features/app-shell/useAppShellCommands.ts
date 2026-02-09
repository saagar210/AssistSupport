import { useMemo } from 'react';
import { buildAppShellCommands } from './commands';
import type { TabId } from './types';
import type { QueueView } from '../inbox/queueModel';
import type { RevampFlags } from '../revamp';

interface UseAppShellCommandsParams {
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
  clearDraft: () => void;
}

export function useAppShellCommands({
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
}: UseAppShellCommandsParams) {
  return useMemo(() => buildAppShellCommands({
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
  }), [
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
  ]);
}
