// @vitest-environment jsdom
import React from 'react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { KnowledgeBrowser } from './KnowledgeBrowser';

const useKnowledgeMock = vi.fn();

vi.mock('../../hooks/useKnowledge', () => ({
  useKnowledge: () => useKnowledgeMock(),
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

vi.mock('./KbHealthPanel', () => ({
  KbHealthPanel: () => <div>Health Panel</div>,
}));

vi.mock('./ChunkEditor', () => ({
  ChunkEditor: () => <div>Chunk Editor</div>,
}));

afterEach(() => {
  cleanup();
  useKnowledgeMock.mockReset();
});

describe('KnowledgeBrowser', () => {
  it('uses semantic buttons for namespace and document selection', () => {
    const selectNamespace = vi.fn();
    const selectDocument = vi.fn();

    useKnowledgeMock.mockReturnValue({
      namespaces: [
        { id: 'default', name: 'Default', documentCount: 1, sourceCount: 1 },
      ],
      selectedNamespace: 'default',
      documents: [
        {
          id: 'doc-1',
          title: 'VPN Guide',
          file_path: '/kb/vpn.md',
          chunk_count: 2,
          indexed_at: '2026-03-01T00:00:00Z',
          last_reviewed_at: null,
          source_type: 'file',
        },
      ],
      selectedDocument: null,
      chunks: [],
      loading: false,
      error: null,
      loadNamespaces: vi.fn(),
      selectNamespace,
      selectDocument,
      deleteNamespace: vi.fn(),
      deleteSource: vi.fn(),
      deleteDocument: vi.fn(),
      clearAll: vi.fn(),
    });

    render(<KnowledgeBrowser />);

    fireEvent.click(screen.getByRole('button', { name: /Default/i }));
    fireEvent.click(screen.getByRole('button', { name: /VPN Guide/i }));

    expect(selectNamespace).toHaveBeenCalledWith('default');
    expect(selectDocument).toHaveBeenCalled();
  });

  it('opens a confirmation dialog before delete actions', () => {
    useKnowledgeMock.mockReturnValue({
      namespaces: [
        { id: 'default', name: 'Default', documentCount: 1, sourceCount: 1 },
      ],
      selectedNamespace: 'default',
      documents: [
        {
          id: 'doc-1',
          title: 'VPN Guide',
          file_path: '/kb/vpn.md',
          chunk_count: 2,
          indexed_at: '2026-03-01T00:00:00Z',
          last_reviewed_at: null,
          source_type: 'file',
        },
      ],
      selectedDocument: null,
      chunks: [],
      loading: false,
      error: null,
      loadNamespaces: vi.fn(),
      selectNamespace: vi.fn(),
      selectDocument: vi.fn(),
      deleteNamespace: vi.fn(),
      deleteSource: vi.fn(),
      deleteDocument: vi.fn(),
      clearAll: vi.fn(),
    });

    render(<KnowledgeBrowser />);

    fireEvent.click(screen.getByTitle('Delete document'));

    expect(screen.getByRole('dialog')).toBeTruthy();
    expect(screen.getByText('Confirm Delete')).toBeTruthy();
  });
});
