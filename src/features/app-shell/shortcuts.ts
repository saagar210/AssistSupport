import { TAB_ORDER, type TabId } from './types';

export function mapShortcutIndexToTab(index: number): TabId | null {
  if (index < 1 || index > TAB_ORDER.length) {
    return null;
  }

  return TAB_ORDER[index - 1] ?? null;
}
