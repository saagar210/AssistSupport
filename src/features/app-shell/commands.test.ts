import { describe, expect, it, vi } from 'vitest';
import { buildAppShellCommands } from './commands';

function createParams(activeTab: 'draft' | 'sources' = 'draft') {
  return {
    activeTab,
    sidebarCollapsed: false,
    revampCommandPaletteV2Enabled: false,
    setActiveTab: vi.fn(),
    openQueueView: vi.fn(),
    handleGenerate: vi.fn(),
    handleSaveDraft: vi.fn(),
    handleCopyResponse: vi.fn(),
    handleExport: vi.fn(),
    handleCancelGeneration: vi.fn(),
    handleToggleSidebar: vi.fn(),
    onOpenShortcuts: vi.fn(),
    addToast: vi.fn(),
    clearDraft: vi.fn(),
  };
}

describe('buildAppShellCommands', () => {
  it('returns core navigation and draft commands', () => {
    const params = createParams('draft');
    const commands = buildAppShellCommands(params);

    expect(commands.some(c => c.id === 'nav-draft')).toBe(true);
    expect(commands.some(c => c.id === 'nav-ops')).toBe(true);
    expect(commands.some(c => c.id === 'nav-settings')).toBe(true);
    expect(commands.some(c => c.id === 'action-generate')).toBe(true);
  });

  it('disables draft-only commands when not on draft tab', () => {
    const params = createParams('sources');
    const commands = buildAppShellCommands(params);

    expect(commands.find(c => c.id === 'action-generate')?.disabled).toBe(true);
    expect(commands.find(c => c.id === 'action-save')?.disabled).toBe(true);
    expect(commands.find(c => c.id === 'action-copy')?.disabled).toBe(true);
  });

  it('new draft command clears draft and focuses draft tab', () => {
    const params = createParams('sources');
    const commands = buildAppShellCommands(params);
    const newDraft = commands.find(c => c.id === 'action-new-draft');

    expect(newDraft).toBeDefined();
    newDraft?.action();

    expect(params.setActiveTab).toHaveBeenCalledWith('draft');
    expect(params.clearDraft).toHaveBeenCalled();
  });

  it('includes queue commands when revamp command palette v2 is enabled', () => {
    const params = createParams('sources');
    params.revampCommandPaletteV2Enabled = true;
    const commands = buildAppShellCommands(params);

    expect(commands.some((c) => c.id === 'queue-open-unassigned')).toBe(true);
    expect(commands.some((c) => c.id === 'queue-open-at-risk')).toBe(true);

    commands.find((c) => c.id === 'queue-open-at-risk')?.action();
    expect(params.openQueueView).toHaveBeenCalledWith('at_risk');
  });
});
