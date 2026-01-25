/**
 * Header - Contextual header component
 * Shows current context, breadcrumbs, and quick actions
 */

import { Icon, IconName } from '../shared/Icon';
import type { Tab } from '../../types';
import './Header.css';

interface HeaderProps {
  activeTab: Tab;
  modelLoaded: boolean;
  modelName: string | null;
  onOpenCommandPalette?: () => void;
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
  settings: {
    title: 'Settings',
    description: 'App configuration',
    icon: 'settings'
  }
};

export function Header({ activeTab, modelLoaded, modelName, onOpenCommandPalette }: HeaderProps) {
  const info = tabInfo[activeTab];

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
        <div className={`model-indicator ${modelLoaded ? 'loaded' : 'not-loaded'}`}>
          <span className="status-dot" />
          <span className="model-status-text">
            {modelLoaded ? (modelName || 'Model loaded') : 'No model'}
          </span>
        </div>
      </div>
    </header>
  );
}
