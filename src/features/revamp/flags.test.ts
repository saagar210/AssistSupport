import { describe, expect, it } from 'vitest';
import { getEnabledRevampFlags, resolveRevampFlags } from './flags';

const makeStorage = (values: Record<string, string>): Pick<Storage, 'getItem'> => ({
  getItem: (key: string) => values[key] ?? null,
});

describe('resolveRevampFlags', () => {
  it('defaults all flags to false when no values are present', () => {
    const flags = resolveRevampFlags({ env: {}, storage: makeStorage({}) });
    expect(getEnabledRevampFlags(flags)).toEqual([]);
  });

  it('reads environment booleans for revamp toggles', () => {
    const flags = resolveRevampFlags({
      env: {
        VITE_ASSISTSUPPORT_REVAMP_INBOX: '1',
        VITE_ASSISTSUPPORT_REVAMP_COMMAND_PALETTE_V2: 'true',
      },
      storage: makeStorage({}),
    });

    expect(flags.ASSISTSUPPORT_REVAMP_INBOX).toBe(true);
    expect(flags.ASSISTSUPPORT_REVAMP_COMMAND_PALETTE_V2).toBe(true);
    expect(flags.ASSISTSUPPORT_REVAMP_APP_SHELL).toBe(false);
  });

  it('prefers local storage override over environment values', () => {
    const flags = resolveRevampFlags({
      env: {
        VITE_ASSISTSUPPORT_REVAMP_INBOX: '0',
      },
      storage: makeStorage({
        'assistsupport.flag.ASSISTSUPPORT_REVAMP_INBOX': '1',
      }),
    });

    expect(flags.ASSISTSUPPORT_REVAMP_INBOX).toBe(true);
  });
});
