import { useMemo } from 'react';
import { buildAppShellCommands } from './commands';
import type { TabId } from './types';

interface UseAppShellCommandsParams {
  activeTab: TabId;
  sidebarCollapsed: boolean;
  setActiveTab: (tab: TabId) => void;
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
  setActiveTab,
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
    setActiveTab,
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
    setActiveTab,
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
