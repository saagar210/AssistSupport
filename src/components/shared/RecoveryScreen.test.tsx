// @vitest-environment jsdom
import React from 'react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { RecoveryScreen } from './RecoveryScreen';
import type { StartupRecoveryIssue } from '../../types';

const invokeMock = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

vi.mock('./Button', () => ({
  Button: ({
    children,
    type,
    onClick,
    disabled,
  }: {
    children: React.ReactNode;
    type?: 'button' | 'submit';
    onClick?: () => void;
    disabled?: boolean;
  }) => (
    <button type={type ?? 'button'} onClick={onClick} disabled={disabled}>
      {children}
    </button>
  ),
}));

vi.mock('./Icon', () => ({
  Icon: ({ name }: { name: string }) => <span>{name}</span>,
}));

const baseIssue: StartupRecoveryIssue = {
  code: 'database_recovery_required',
  summary: 'Startup entered recovery mode',
  details: 'Database integrity check failed.',
  can_repair: true,
  can_restore_backup: true,
  requires_manual_resolution: false,
  migration_conflicts: [],
};

afterEach(() => {
  cleanup();
  invokeMock.mockReset();
});

describe('RecoveryScreen', () => {
  it('renders repair and restore actions for repairable issues', () => {
    render(<RecoveryScreen issue={baseIssue} />);

    expect(screen.getByRole('heading', { name: 'Startup entered recovery mode' })).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Repair Database' })).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Restore From Backup' })).toBeTruthy();
  });

  it('invokes the repair command and shows its message', async () => {
    invokeMock.mockResolvedValue({
      component: 'Database',
      success: true,
      action_taken: 'Ran VACUUM',
      message: 'Database integrity restored',
    });

    render(<RecoveryScreen issue={baseIssue} />);
    fireEvent.click(screen.getByRole('button', { name: 'Repair Database' }));

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith('repair_database_cmd');
    });
    expect(screen.getByRole('status').textContent).toContain('Database integrity restored');
  });

  it('renders migration conflicts when manual resolution is required', () => {
    render(
      <RecoveryScreen
        issue={{
          ...baseIssue,
          can_repair: false,
          can_restore_backup: false,
          requires_manual_resolution: true,
          migration_conflicts: [
            {
              name: 'assistsupport.db',
              old_path: '/old/assistsupport.db',
              new_path: '/new/assistsupport.db',
              reason: 'Both locations contain data',
            },
          ],
        }}
      />,
    );

    expect(screen.queryByRole('button', { name: 'Repair Database' })).toBeNull();
    expect(screen.getByText('Migration Conflicts')).toBeTruthy();
    expect(screen.getByText('Both locations contain data')).toBeTruthy();
  });
});
