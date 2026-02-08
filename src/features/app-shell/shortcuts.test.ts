import { describe, it, expect } from 'vitest';
import { mapShortcutIndexToTab } from './shortcuts';

describe('mapShortcutIndexToTab', () => {
  it('maps valid indices to tabs', () => {
    expect(mapShortcutIndexToTab(1)).toBe('draft');
    expect(mapShortcutIndexToTab(9)).toBe('ops');
    expect(mapShortcutIndexToTab(10)).toBe('settings');
  });

  it('returns null for out-of-range indices', () => {
    expect(mapShortcutIndexToTab(0)).toBeNull();
    expect(mapShortcutIndexToTab(11)).toBeNull();
    expect(mapShortcutIndexToTab(-1)).toBeNull();
  });
});
