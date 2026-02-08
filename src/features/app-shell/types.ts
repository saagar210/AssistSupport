export type TabId =
  | 'draft'
  | 'followups'
  | 'sources'
  | 'ingest'
  | 'knowledge'
  | 'analytics'
  | 'pilot'
  | 'search'
  | 'ops'
  | 'settings';

export const TAB_ORDER: TabId[] = [
  'draft',
  'followups',
  'sources',
  'ingest',
  'knowledge',
  'analytics',
  'pilot',
  'search',
  'ops',
  'settings',
];

export interface TabShortcutMap {
  shortcut: string;
  tab: TabId;
  label: string;
}

export const TAB_SHORTCUTS: TabShortcutMap[] = [
  { shortcut: 'Cmd+1', tab: 'draft', label: 'Draft' },
  { shortcut: 'Cmd+2', tab: 'followups', label: 'Follow-ups' },
  { shortcut: 'Cmd+3', tab: 'sources', label: 'Sources' },
  { shortcut: 'Cmd+4', tab: 'ingest', label: 'Ingest' },
  { shortcut: 'Cmd+5', tab: 'knowledge', label: 'Knowledge' },
  { shortcut: 'Cmd+6', tab: 'analytics', label: 'Analytics' },
  { shortcut: 'Cmd+7', tab: 'pilot', label: 'Pilot' },
  { shortcut: 'Cmd+8', tab: 'search', label: 'Search' },
  { shortcut: 'Cmd+9', tab: 'ops', label: 'Operations' },
  { shortcut: 'Cmd+0', tab: 'settings', label: 'Settings' },
];
