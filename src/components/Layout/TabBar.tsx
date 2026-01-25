import type { Tab } from '../../types';
import './TabBar.css';

interface TabBarProps {
  activeTab: Tab;
  onTabChange: (tab: Tab) => void;
}

const tabs: { id: Tab; label: string; shortcut: string }[] = [
  { id: 'draft', label: 'Draft', shortcut: '1' },
  { id: 'followups', label: 'Follow-ups', shortcut: '2' },
  { id: 'sources', label: 'Sources', shortcut: '3' },
  { id: 'settings', label: 'Settings', shortcut: '4' },
];

export function TabBar({ activeTab, onTabChange }: TabBarProps) {
  return (
    <nav className="tab-bar">
      {tabs.map(tab => (
        <button
          key={tab.id}
          className={`tab-item ${activeTab === tab.id ? 'active' : ''}`}
          onClick={() => onTabChange(tab.id)}
          title={`${tab.label} (Cmd+${tab.shortcut})`}
        >
          {tab.label}
        </button>
      ))}
    </nav>
  );
}
