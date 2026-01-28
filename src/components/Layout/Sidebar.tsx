/**
 * Sidebar - Primary navigation component
 * Persistent sidebar with icons and labels
 */

import { Icon, IconName } from '../shared/Icon';
import type { Tab } from '../../types';
import './Sidebar.css';

interface SidebarProps {
  activeTab: Tab;
  onTabChange: (tab: Tab) => void;
  collapsed: boolean;
  onToggleCollapse: () => void;
}

interface NavItem {
  id: Tab;
  label: string;
  icon: IconName;
  shortcut: string;
  description: string;
}

const navItems: NavItem[] = [
  {
    id: 'draft',
    label: 'Draft',
    icon: 'draft',
    shortcut: '1',
    description: 'Compose responses with AI assistance'
  },
  {
    id: 'followups',
    label: 'Follow-ups',
    icon: 'followups',
    shortcut: '2',
    description: 'Manage saved drafts and history'
  },
  {
    id: 'sources',
    label: 'Sources',
    icon: 'sources',
    shortcut: '3',
    description: 'Browse knowledge base sources'
  },
  {
    id: 'ingest',
    label: 'Ingest',
    icon: 'ingest',
    shortcut: '4',
    description: 'Add new content to knowledge base'
  },
  {
    id: 'knowledge',
    label: 'Knowledge',
    icon: 'knowledge',
    shortcut: '5',
    description: 'Search and explore knowledge base'
  },
  {
    id: 'analytics',
    label: 'Analytics',
    icon: 'sparkles',
    shortcut: '6',
    description: 'View usage analytics and statistics'
  },
  {
    id: 'pilot',
    label: 'Pilot',
    icon: 'list',
    shortcut: '7',
    description: 'View pilot feedback dashboard'
  },
];

const settingsItem: NavItem = {
  id: 'settings',
  label: 'Settings',
  icon: 'settings',
  shortcut: '8',
  description: 'Configure app and model settings'
};

export function Sidebar({ activeTab, onTabChange, collapsed, onToggleCollapse }: SidebarProps) {
  return (
    <aside className={`sidebar ${collapsed ? 'collapsed' : ''}`}>
      <div className="sidebar-header">
        {!collapsed && (
          <div className="sidebar-brand">
            <span className="brand-icon">A</span>
            <span className="brand-text">AssistSupport</span>
          </div>
        )}
        <button
          className="sidebar-toggle"
          onClick={onToggleCollapse}
          title={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
          aria-label={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
        >
          <Icon name={collapsed ? 'chevron-right' : 'chevron-left'} size={16} />
        </button>
      </div>

      <nav className="sidebar-nav" role="navigation" aria-label="Main navigation">
        <ul className="nav-list">
          {navItems.map(item => (
            <li key={item.id}>
              <button
                className={`nav-item ${activeTab === item.id ? 'active' : ''}`}
                onClick={() => onTabChange(item.id)}
                title={collapsed ? `${item.label} (Cmd+${item.shortcut})` : item.description}
                aria-current={activeTab === item.id ? 'page' : undefined}
              >
                <Icon name={item.icon} size={20} className="nav-icon" />
                {!collapsed && (
                  <>
                    <span className="nav-label">{item.label}</span>
                    <span className="nav-shortcut">
                      <kbd>&#8984;</kbd>
                      <kbd>{item.shortcut}</kbd>
                    </span>
                  </>
                )}
              </button>
            </li>
          ))}
        </ul>
      </nav>

      <div className="sidebar-footer">
        <button
          className={`nav-item ${activeTab === 'settings' ? 'active' : ''}`}
          onClick={() => onTabChange('settings')}
          title={collapsed ? `Settings (Cmd+${settingsItem.shortcut})` : settingsItem.description}
          aria-current={activeTab === 'settings' ? 'page' : undefined}
        >
          <Icon name={settingsItem.icon} size={20} className="nav-icon" />
          {!collapsed && (
            <>
              <span className="nav-label">{settingsItem.label}</span>
              <span className="nav-shortcut">
                <kbd>&#8984;</kbd>
                <kbd>{settingsItem.shortcut}</kbd>
              </span>
            </>
          )}
        </button>
      </div>
    </aside>
  );
}
