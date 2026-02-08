import { useMemo } from 'react';
import { buildAppShellCommands } from './commands';
import type { TabId } from './types';
import type { QueueView } from '../inbox/queueModel';

interface UseAppShellCommandsParams {
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
  clearDraft: () => void;
}

export function useAppShellCommands({
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
}: UseAppShellCommandsParams) {
  return useMemo(() => buildAppShellCommands({
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
  }), [
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
  ]);
}
