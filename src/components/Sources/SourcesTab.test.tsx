// @vitest-environment jsdom
import React from 'react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { SourcesTab } from './SourcesTab';

const invokeMock = vi.fn();
const useKbMock = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

vi.mock('../../hooks/useKb', () => ({
  useKb: () => useKbMock(),
}));

vi.mock('../../contexts/ToastContext', () => ({
  useToastContext: () => ({
    success: vi.fn(),
    error: vi.fn(),
  }),
}));

vi.mock('../shared/Button', () => ({
  Button: ({
    children,
    onClick,
    disabled,
  }: {
    children: React.ReactNode;
    onClick?: () => void;
    disabled?: boolean;
  }) => (
    <button type="button" onClick={onClick} disabled={disabled}>
      {children}
    </button>
  ),
}));

vi.mock('../shared/Skeleton', () => ({
  Skeleton: () => <div>loading</div>,
}));

afterEach(() => {
  cleanup();
  invokeMock.mockReset();
  useKbMock.mockReset();
});

describe('SourcesTab', () => {
  it('shows a loading state before the knowledge base folder resolves', () => {
    useKbMock.mockReturnValue({
      getKbFolder: () => new Promise(() => {}),
      listFiles: () => Promise.resolve([]),
      rebuildIndex: vi.fn(),
      getIndexStats: () => Promise.resolve(null),
      search: vi.fn(),
      removeDocument: vi.fn(),
    });
    invokeMock.mockResolvedValue([]);

    render(<SourcesTab />);

    expect(screen.getByText('Loading Knowledge Base')).toBeTruthy();
    expect(screen.queryByText('No Knowledge Base Configured')).toBeNull();
  });

  it('opens an accessible remove confirmation dialog', async () => {
    useKbMock.mockReturnValue({
      getKbFolder: () => Promise.resolve('/mock/kb'),
      listFiles: () => Promise.resolve([
        {
          file_path: '/mock/kb/policy.md',
          title: 'Policy',
          chunk_count: 2,
          indexed_at: '2026-03-01T00:00:00Z',
        },
      ]),
      rebuildIndex: vi.fn(),
      getIndexStats: () => Promise.resolve({ total_chunks: 2, total_files: 1 }),
      search: vi.fn(),
      removeDocument: vi.fn().mockResolvedValue(true),
    });
    invokeMock.mockResolvedValue([]);

    render(<SourcesTab />);

    await waitFor(() => {
      expect(screen.getByText('Policy')).toBeTruthy();
    });

    fireEvent.click(screen.getByRole('button', { name: 'Remove' }));

    expect(screen.getByRole('dialog')).toBeTruthy();
    expect(screen.getByText('Remove from Knowledge Base')).toBeTruthy();
  });
});
