/**
 * Header - Contextual header component
 * Shows current context, breadcrumbs, and quick actions
 */

import { useState } from 'react';
import { Icon, IconName } from '../shared/Icon';
import { useAppStatus } from '../../contexts/AppStatusContext';
import type { Tab } from '../../types';
import './Header.css';

interface HeaderProps {
  activeTab: Tab;
  onOpenCommandPalette?: () => void;
  onOpenShortcuts?: () => void;
}

interface TabInfo {
  title: string;
  description: string;
  icon: IconName;
}

const tabInfo: Record<Tab, TabInfo> = {
  draft: {
    title: 'Draft',
    description: 'Compose AI-assisted responses',
    icon: 'draft'
  },
  followups: {
    title: 'Follow-ups',
    description: 'Saved drafts and history',
    icon: 'followups'
  },
  sources: {
    title: 'Sources',
    description: 'Knowledge base files',
    icon: 'sources'
  },
  ingest: {
    title: 'Ingest',
    description: 'Add new content',
    icon: 'ingest'
  },
  knowledge: {
    title: 'Knowledge',
    description: 'Search and explore',
    icon: 'knowledge'
  },
  analytics: {
    title: 'Analytics',
    description: 'Usage analytics and statistics',
    icon: 'sparkles'
  },
  pilot: {
    title: 'Pilot',
    description: 'Feedback dashboard',
    icon: 'list'
  },
  search: {
    title: 'Hybrid Search',
    description: 'PostgreSQL BM25 + HNSW search',
    icon: 'database'
  },
  ops: {
    title: 'Operations',
    description: 'Deployment, eval harness, triage, and runbooks',
    icon: 'terminal'
  },
  settings: {
    title: 'Settings',
    description: 'App configuration',
    icon: 'settings'
  }
};

export function Header({ activeTab, onOpenCommandPalette, onOpenShortcuts }: HeaderProps) {
  const info = tabInfo[activeTab];
  const appStatus = useAppStatus();
  const [showStatusPanel, setShowStatusPanel] = useState(false);
  const memoryKernelCommitShort = appStatus.memoryKernelCommitSha
    ? appStatus.memoryKernelCommitSha.slice(0, 12)
    : null;
  const memoryKernelTrace = [
    appStatus.memoryKernelReleaseTag ? `tag ${appStatus.memoryKernelReleaseTag}` : null,
    memoryKernelCommitShort ? `sha ${memoryKernelCommitShort}` : null,
    appStatus.memoryKernelServiceContract && appStatus.memoryKernelApiContract
      ? `${appStatus.memoryKernelServiceContract}/${appStatus.memoryKernelApiContract}`
      : null,
    appStatus.memoryKernelIntegrationBaseline
      ? appStatus.memoryKernelIntegrationBaseline
      : null,
  ]
    .filter(Boolean)
    .join(' · ');
  const memoryKernelDetail = appStatus.memoryKernelFeatureEnabled
    ? [appStatus.memoryKernelDetail, memoryKernelTrace].filter(Boolean).join(' · ')
    : 'Feature disabled';

  // Compute overall health
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
  const overallHealth = healthyCount === totalChecks ? 'good' : healthyCount > 0 ? 'partial' : 'none';

  return (
    <header className="app-header">
      <div className="header-left">
        <div className="header-title-group">
          <Icon name={info.icon} size={20} className="header-icon" />
          <div className="header-text">
            <h1 className="header-title">{info.title}</h1>
            <span className="header-description">{info.description}</span>
          </div>
        </div>
      </div>

      <div className="header-center">
        <button
          className="command-trigger"
          onClick={onOpenCommandPalette}
          title="Open command palette (Cmd+K)"
        >
          <Icon name="search" size={16} />
          <span className="command-placeholder">Search or type a command...</span>
          <div className="command-shortcut">
            <kbd>&#8984;</kbd>
            <kbd>K</kbd>
          </div>
        </button>
      </div>

      <div className="header-right">
        {onOpenShortcuts && (
          <button
            className="header-help-btn"
            onClick={onOpenShortcuts}
            title="Keyboard shortcuts (Cmd+?)"
          >
            <Icon name="help-circle" size={18} />
          </button>
        )}
        <button
          className={`status-indicator status-${overallHealth}`}
          onClick={() => setShowStatusPanel(!showStatusPanel)}
          title="View system status"
        >
          <span className="status-dot" />
          <span className="status-text">
            {appStatus.llmLoaded ? (appStatus.llmModelName || 'Ready') : 'Setup required'}
          </span>
          <Icon name="chevron-down" size={14} className={`status-chevron ${showStatusPanel ? 'open' : ''}`} />
        </button>

        {showStatusPanel && (
          <div className="status-panel">
            <div className="status-panel-header">
              <span className="status-panel-title">System Status</span>
              <button className="status-refresh" onClick={() => appStatus.refresh()} title="Refresh">
                <Icon name="refresh" size={14} />
              </button>
            </div>
            <div className="status-panel-items">
              <StatusItem
                label="LLM Engine"
                status={appStatus.llmLoaded}
                detail={appStatus.llmModelName || 'Not loaded'}
                loading={appStatus.llmLoading}
              />
              <StatusItem
                label="Embeddings"
                status={appStatus.embeddingsLoaded}
                detail={appStatus.embeddingsLoaded ? 'Loaded' : 'Not loaded'}
              />
              <StatusItem
                label="Vector Store"
                status={appStatus.vectorEnabled}
                detail={appStatus.vectorEnabled ? 'Enabled' : 'Disabled'}
              />
              <StatusItem
                label="Knowledge Base"
                status={appStatus.kbIndexed}
                detail={`${appStatus.kbDocumentCount} docs, ${appStatus.kbChunkCount} chunks`}
              />
              <StatusItem
                label="MemoryKernel"
                status={appStatus.memoryKernelReady}
                detail={memoryKernelDetail}
              />
            </div>
          </div>
        )}
      </div>
    </header>
  );
}

interface StatusItemProps {
  label: string;
  status: boolean;
  detail: string;
  loading?: boolean;
}

function StatusItem({ label, status, detail, loading }: StatusItemProps) {
  return (
    <div className="status-item">
      <div className="status-item-left">
        <span className={`status-item-dot ${status ? 'active' : 'inactive'} ${loading ? 'loading' : ''}`} />
        <span className="status-item-label">{label}</span>
      </div>
      <span className="status-item-detail">{loading ? 'Loading...' : detail}</span>
    </div>
  );
}
