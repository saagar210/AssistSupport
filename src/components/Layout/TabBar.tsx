import type { Tab } from '../../types';
import './TabBar.css';

interface TabBarProps {
  activeTab: Tab;
  onTabChange: (tab: Tab) => void;
}

const tabs: { id: Tab; label: string; shortcut?: string }[] = [
  { id: 'draft', label: 'Draft', shortcut: '1' },
  { id: 'followups', label: 'Follow-ups', shortcut: '2' },
  { id: 'sources', label: 'Sources', shortcut: '3' },
  { id: 'ingest', label: 'Ingest', shortcut: '4' },
  { id: 'knowledge', label: 'Knowledge', shortcut: '5' },
  { id: 'analytics', label: 'Analytics', shortcut: '6' },
  { id: 'pilot', label: 'Pilot', shortcut: '7' },
  { id: 'search', label: 'Search', shortcut: '8' },
  { id: 'ops', label: 'Ops' },
  { id: 'settings', label: 'Settings', shortcut: '9' },
];

export function TabBar({ activeTab, onTabChange }: TabBarProps) {
  return (
    <nav className="tab-bar">
      {tabs.map(tab => (
        <button
          key={tab.id}
          className={`tab-item ${activeTab === tab.id ? 'active' : ''}`}
          onClick={() => onTabChange(tab.id)}
          title={tab.shortcut ? `${tab.label} (Cmd+${tab.shortcut})` : tab.label}
        >
          {tab.label}
        </button>
      ))}
    </nav>
  );
}
