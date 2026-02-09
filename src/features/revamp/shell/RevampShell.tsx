import type { ReactNode } from 'react';
import { useEffect, useMemo, useRef, useState } from 'react';
import { Icon } from '../../../components/shared/Icon';
import type { Tab } from '../../../types';
import type { RevampFlags } from '../flags';
import { isTabEnabled } from '../../app-shell/tabPolicy';
import { useAppStatus } from '../../../contexts/AppStatusContext';
import { AsButton, Badge, Panel } from '../ui';
import { WorkspaceQueueContext } from '../../workspace/WorkspaceQueueContext';
import type { QueueView } from '../../inbox/queueModel';
import '../../../styles/revamp/index.css';
import './revampShell.css';

export interface RevampShellProps {
  activeTab: Tab;
  onTabChange: (tab: Tab) => void;
  revampFlags: RevampFlags;
  onNavigateToQueue?: (queueView: QueueView) => void;
  onOpenCommandPalette: () => void;
  onOpenShortcuts: () => void;
  children: ReactNode;
}

interface NavItem {
  id: Tab;
  label: string;
  icon: Parameters<typeof Icon>[0]['name'];
  section: 'Primary' | 'Knowledge' | 'Operations' | 'Advanced';
}

const NAV: NavItem[] = [
  { id: 'draft', label: 'Draft', icon: 'draft', section: 'Primary' },
  { id: 'followups', label: 'Follow-ups', icon: 'followups', section: 'Primary' },
  { id: 'sources', label: 'Sources', icon: 'sources', section: 'Knowledge' },
  { id: 'knowledge', label: 'Knowledge', icon: 'knowledge', section: 'Knowledge' },
  { id: 'ops', label: 'Ops', icon: 'terminal', section: 'Operations' },
  { id: 'settings', label: 'Settings', icon: 'settings', section: 'Operations' },
  // Advanced surfaces are policy gated and default off.
  { id: 'ingest', label: 'Ingest', icon: 'ingest', section: 'Advanced' },
  { id: 'analytics', label: 'Analytics', icon: 'sparkles', section: 'Advanced' },
  { id: 'pilot', label: 'Pilot', icon: 'list', section: 'Advanced' },
  { id: 'search', label: 'Search', icon: 'database', section: 'Advanced' },
];

function tabTitle(tab: Tab): string {
  switch (tab) {
    case 'draft':
      return 'Draft Workbench';
    case 'followups':
      return 'Queue / Follow-ups';
    case 'sources':
      return 'Sources';
    case 'knowledge':
      return 'Knowledge';
    case 'ops':
      return 'Operations';
    case 'settings':
      return 'Settings';
    case 'ingest':
      return 'Ingest';
    case 'analytics':
      return 'Analytics';
    case 'pilot':
      return 'Pilot';
    case 'search':
      return 'Search';
  }
}

export function RevampShell({
  activeTab,
  onTabChange,
  revampFlags,
  onNavigateToQueue,
  onOpenCommandPalette,
  onOpenShortcuts,
  children,
}: RevampShellProps) {
  const appStatus = useAppStatus();
  const [statusOpen, setStatusOpen] = useState(false);
  const statusPopoverRef = useRef<HTMLDivElement | null>(null);
  const statusButtonRef = useRef<HTMLButtonElement | null>(null);

  const enabledNav = useMemo(() => {
    return NAV.filter((item) => isTabEnabled(item.id, revampFlags));
  }, [revampFlags]);

  const checks = [
    appStatus.llmLoaded,
    appStatus.embeddingsLoaded,
    appStatus.kbIndexed,
  ];
  if (appStatus.memoryKernelFeatureEnabled) {
    checks.push(appStatus.memoryKernelReady);
  }
  const healthyCount = checks.filter(Boolean).length;
  const totalChecks = checks.length;
  const healthTone = healthyCount === totalChecks ? 'good' : healthyCount > 0 ? 'warn' : 'bad';
  const healthLabel = healthyCount === totalChecks ? 'Ready' : healthyCount > 0 ? 'Degraded' : 'Setup required';

  const grouped = (section: NavItem['section']) => enabledNav.filter((n) => n.section === section);

  const needsModel = !appStatus.llmLoaded;
  const needsKb = !appStatus.kbIndexed;
  const memoryKernelDegraded = appStatus.memoryKernelFeatureEnabled && !appStatus.memoryKernelReady;

  useEffect(() => {
    if (!statusOpen) return;

    const handleClickOutside = (event: MouseEvent) => {
      const target = event.target as Node | null;
      if (!target) return;

      const popover = statusPopoverRef.current;
      const button = statusButtonRef.current;
      if (popover?.contains(target) || button?.contains(target)) {
        return;
      }
      setStatusOpen(false);
    };

    const handleEscape = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        setStatusOpen(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    document.addEventListener('keydown', handleEscape);
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
      document.removeEventListener('keydown', handleEscape);
    };
  }, [statusOpen]);

  return (
    <div className="as-shell">
      <aside className="as-shell__nav" aria-label="Primary navigation">
        <div className="as-shell__brand">
          <div className="as-shell__brandMark" aria-hidden="true">A</div>
          <div className="as-shell__brandText">
            <div className="as-shell__brandName">AssistSupport</div>
            <div className="as-shell__brandMeta">Local-first support console</div>
          </div>
        </div>

        {(['Primary', 'Knowledge', 'Operations', 'Advanced'] as const).map((section) => {
          const items = grouped(section);
          if (items.length === 0) return null;
          return (
            <div key={section} className="as-shell__navSection">
              <div className="as-shell__navSectionTitle">{section}</div>
              <ul className="as-shell__navList">
                {items.map((item) => (
                  <li key={item.id}>
                    <button
                      type="button"
                      className={['as-shell__navItem', activeTab === item.id ? 'is-active' : ''].filter(Boolean).join(' ')}
                      onClick={() => onTabChange(item.id)}
                      aria-current={activeTab === item.id ? 'page' : undefined}
                    >
                      <Icon name={item.icon} size={18} />
                      <span>{item.label}</span>
                    </button>
                  </li>
                ))}
              </ul>
            </div>
          );
        })}
      </aside>

      <div className="as-shell__main">
        <header className="as-shell__topbar">
          <div className="as-shell__topbarLeft">
            <div className="as-shell__pageTitle">{tabTitle(activeTab)}</div>
            <div className="as-shell__pageSub">
              Cmd+K for commands · Cmd+0 for settings
            </div>
          </div>

          <button type="button" className="as-shell__command" onClick={onOpenCommandPalette}>
            <Icon name="search" size={16} />
            <span className="as-shell__commandText">Search or type a command...</span>
            <span className="as-shell__commandKbd" aria-hidden="true">⌘ K</span>
          </button>

          <div className="as-shell__topbarRight">
            <button
              type="button"
              className="as-shell__iconBtn"
              onClick={onOpenShortcuts}
              title="Keyboard shortcuts (Cmd+?)"
            >
              <Icon name="help-circle" size={18} />
            </button>

            <button
              type="button"
              className="as-shell__statusBtn"
              onClick={() => setStatusOpen((v) => !v)}
              title="View system status"
              ref={statusButtonRef}
            >
              <Badge tone={healthTone}>{healthLabel}</Badge>
              <span className="as-shell__statusModel">
                {appStatus.llmLoaded ? (appStatus.llmModelName || 'Model loaded') : 'Model not loaded'}
              </span>
              <Icon name="chevron-down" size={14} />
            </button>
          </div>

          {statusOpen && (
            <div
              className="as-shell__statusPopover"
              role="dialog"
              aria-label="System status"
              ref={statusPopoverRef}
            >
              <Panel
                title="System Status"
                subtitle="Local-only health checks"
                actions={
                  <button
                    type="button"
                    className="as-shell__miniBtn"
                    onClick={() => {
                      void appStatus.refresh();
                    }}
                    title="Refresh status"
                  >
                    <Icon name="refresh" size={14} /> Refresh
                  </button>
                }
              >
                <div className="as-shell__statusGrid">
                  <StatusRow label="LLM Engine" value={appStatus.llmLoaded ? 'Loaded' : 'Not loaded'} tone={appStatus.llmLoaded ? 'good' : 'bad'} />
                  <StatusRow label="Embeddings" value={appStatus.embeddingsLoaded ? 'Loaded' : 'Not loaded'} tone={appStatus.embeddingsLoaded ? 'good' : 'warn'} />
                  <StatusRow label="Knowledge Base" value={`${appStatus.kbDocumentCount} docs · ${appStatus.kbChunkCount} chunks`} tone={appStatus.kbIndexed ? 'good' : 'warn'} />
                  <StatusRow label="MemoryKernel" value={appStatus.memoryKernelFeatureEnabled ? (appStatus.memoryKernelStatus || 'Unknown') : 'Disabled'} tone={appStatus.memoryKernelFeatureEnabled ? (appStatus.memoryKernelReady ? 'good' : 'warn') : 'neutral'} />
                </div>
                {(needsModel || needsKb || memoryKernelDegraded) && (
                  <div style={{ marginTop: 'var(--as-space-4)', display: 'flex', gap: 'var(--as-space-2)', flexWrap: 'wrap' }}>
                    {needsModel && (
                      <AsButton
                        size="small"
                        tone="primary"
                        onClick={() => {
                          setStatusOpen(false);
                          onTabChange('settings');
                        }}
                      >
                        Open Settings
                      </AsButton>
                    )}
                    {needsKb && (
                      <AsButton
                        size="small"
                        tone="primary"
                        onClick={() => {
                          setStatusOpen(false);
                          onTabChange('knowledge');
                        }}
                      >
                        Fix Knowledge Base
                      </AsButton>
                    )}
                    {memoryKernelDegraded && (
                      <AsButton
                        size="small"
                        tone="default"
                        onClick={() => {
                          setStatusOpen(false);
                          onTabChange('ops');
                        }}
                      >
                        Open Ops
                      </AsButton>
                    )}
                  </div>
                )}
              </Panel>
            </div>
          )}
        </header>

        <div className="as-shell__content">
          <main className="as-shell__workspace" aria-label="Workspace">
            <div className="as-shell__workspaceInner">{children}</div>
          </main>
          <aside className="as-shell__rail" aria-label="Diagnostics and guidance">
            {activeTab === 'draft' && revampFlags.ASSISTSUPPORT_REVAMP_WORKSPACE && (
              <>
                <WorkspaceQueueContext onNavigateToQueue={onNavigateToQueue} revampUi />
                <Panel title="Response playbook" subtitle="Keep responses consistent across the team">
                  <ol className="as-shell__railBullets">
                    <li>Capture the issue in plain language.</li>
                    <li>Validate policy and approval requirements.</li>
                    <li>Generate and edit response with cited context.</li>
                    <li>Save to follow-ups for handoff continuity.</li>
                  </ol>
                </Panel>
              </>
            )}
            <Panel
              title="AI Status & Guarantees"
              subtitle="Predictable local AI; never blocking"
            >
              <ul className="as-shell__railList">
                <li>
                  <span className="as-shell__railKey">Model</span>
                  <span className="as-shell__railVal">{appStatus.llmLoaded ? (appStatus.llmModelName || 'Loaded') : 'Not loaded'}</span>
                </li>
                <li>
                  <span className="as-shell__railKey">Citations</span>
                  <span className="as-shell__railVal">Required for copy (override audited)</span>
                </li>
                <li>
                  <span className="as-shell__railKey">MemoryKernel</span>
                  <span className="as-shell__railVal">{appStatus.memoryKernelFeatureEnabled ? 'Optional enrichment' : 'Disabled'}</span>
                </li>
              </ul>
            </Panel>

            <Panel
              title="Next Best Actions"
              subtitle="Always have a deterministic next step"
            >
              <ul className="as-shell__railBullets">
                {needsModel && (
                  <li>
                    Load a local model in Settings.
                    <div style={{ marginTop: 'var(--as-space-2)' }}>
                      <AsButton
                        size="small"
                        tone="primary"
                        onClick={() => onTabChange('settings')}
                      >
                        Open Settings
                      </AsButton>
                    </div>
                  </li>
                )}
                {needsKb && (
                  <li>
                    Point Knowledge Base to your local docs folder, then rebuild the index.
                    <div style={{ marginTop: 'var(--as-space-2)' }}>
                      <AsButton
                        size="small"
                        tone="primary"
                        onClick={() => onTabChange('knowledge')}
                      >
                        Open Knowledge
                      </AsButton>
                    </div>
                  </li>
                )}
                {memoryKernelDegraded && (
                  <li>
                    MemoryKernel is degraded. Draft generation will continue with deterministic fallback.
                    <div style={{ marginTop: 'var(--as-space-2)' }}>
                      <AsButton
                        size="small"
                        onClick={() => onTabChange('ops')}
                      >
                        Open Ops
                      </AsButton>
                    </div>
                  </li>
                )}
                {!needsModel && !needsKb && !memoryKernelDegraded && (
                  <li>Use Cmd+K to jump between Queue, Draft, Sources, and Ops.</li>
                )}
              </ul>
            </Panel>
          </aside>
        </div>
      </div>
    </div>
  );
}

function StatusRow({ label, value, tone }: { label: string; value: string; tone: 'neutral' | 'good' | 'warn' | 'bad' }) {
  return (
    <div className="as-shell__statusRow">
      <div className="as-shell__statusLabel">{label}</div>
      <div className="as-shell__statusValue">
        <Badge tone={tone}>{value}</Badge>
      </div>
    </div>
  );
}
