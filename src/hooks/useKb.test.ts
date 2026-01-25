import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useKb } from './useKb';
import { invoke } from '@tauri-apps/api/core';

vi.mock('@tauri-apps/api/core');

const mockInvoke = vi.mocked(invoke);

describe('useKb', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('initializes with default state', () => {
    const { result } = renderHook(() => useKb());

    expect(result.current.folderPath).toBeNull();
    expect(result.current.stats).toBeNull();
    expect(result.current.documents).toEqual([]);
    expect(result.current.indexing).toBe(false);
    expect(result.current.searching).toBe(false);
    expect(result.current.error).toBeNull();
  });

  it('loadKbInfo fetches folder, stats, and documents', async () => {
    const mockStats = { chunk_count: 100, document_count: 5, total_words: 5000 };
    const mockDocs = [
      { id: '1', file_path: '/kb/doc1.md', title: 'Doc 1', indexed_at: '2024-01-01', chunk_count: 10 },
      { id: '2', file_path: '/kb/doc2.md', title: 'Doc 2', indexed_at: '2024-01-02', chunk_count: 20 },
    ];

    mockInvoke.mockImplementation(async (cmd: string) => {
      switch (cmd) {
        case 'get_kb_folder':
          return '/path/to/kb';
        case 'get_kb_stats':
          return mockStats;
        case 'list_kb_documents':
          return mockDocs;
        default:
          return undefined;
      }
    });

    const { result } = renderHook(() => useKb());

    await act(async () => {
      await result.current.loadKbInfo();
    });

    expect(result.current.folderPath).toBe('/path/to/kb');
    expect(result.current.stats).toEqual(mockStats);
    expect(result.current.documents).toEqual(mockDocs);
    expect(result.current.error).toBeNull();
  });

  it('setKbFolder updates folder path', async () => {
    mockInvoke.mockResolvedValue(undefined);

    const { result } = renderHook(() => useKb());

    await act(async () => {
      await result.current.setKbFolder('/new/kb/path');
    });

    expect(mockInvoke).toHaveBeenCalledWith('set_kb_folder', { folderPath: '/new/kb/path' });
    expect(result.current.folderPath).toBe('/new/kb/path');
    expect(result.current.error).toBeNull();
  });

  it('setKbFolder sets error on failure', async () => {
    mockInvoke.mockRejectedValue(new Error('Folder does not exist'));

    const { result } = renderHook(() => useKb());

    await act(async () => {
      try {
        await result.current.setKbFolder('/invalid/path');
      } catch {
        // Expected to throw
      }
    });

    expect(result.current.error).toContain('Folder does not exist');
  });

  it('indexKb sets indexing state and refreshes after completion', async () => {
    const mockResult = { total_files: 5, indexed: 5, skipped: 0, errors: 0 };
    const mockStats = { chunk_count: 100, document_count: 5, total_words: 5000 };
    const mockDocs = [{ id: '1', file_path: '/kb/doc1.md', title: 'Doc 1', indexed_at: '2024-01-01', chunk_count: 20 }];

    mockInvoke.mockImplementation(async (cmd: string) => {
      switch (cmd) {
        case 'index_kb':
          return mockResult;
        case 'get_kb_stats':
          return mockStats;
        case 'list_kb_documents':
          return mockDocs;
        default:
          return undefined;
      }
    });

    const { result } = renderHook(() => useKb());

    let indexPromise: Promise<unknown>;
    act(() => {
      indexPromise = result.current.indexKb();
    });

    // Should be indexing
    expect(result.current.indexing).toBe(true);

    await act(async () => {
      await indexPromise;
    });

    expect(result.current.indexing).toBe(false);
    expect(result.current.stats).toEqual(mockStats);
    expect(result.current.documents).toEqual(mockDocs);
  });

  it('search sets searching state and returns results', async () => {
    const mockResults = [
      {
        chunk_id: 'c1',
        document_id: 'd1',
        file_path: '/kb/doc.md',
        title: 'Test Doc',
        heading_path: 'Section > Subsection',
        content: 'Test content',
        score: 0.95,
      },
    ];

    mockInvoke.mockResolvedValue(mockResults);

    const { result } = renderHook(() => useKb());

    let searchPromise: Promise<unknown>;
    let searchResults: unknown;

    act(() => {
      searchPromise = result.current.search('test query');
    });

    expect(result.current.searching).toBe(true);

    await act(async () => {
      searchResults = await searchPromise;
    });

    expect(result.current.searching).toBe(false);
    expect(searchResults).toEqual(mockResults);
    expect(mockInvoke).toHaveBeenCalledWith('search_kb', { query: 'test query', limit: 10 });
  });

  it('search with custom limit', async () => {
    mockInvoke.mockResolvedValue([]);

    const { result } = renderHook(() => useKb());

    await act(async () => {
      await result.current.search('test', 5);
    });

    expect(mockInvoke).toHaveBeenCalledWith('search_kb', { query: 'test', limit: 5 });
  });

  it('getSearchContext returns formatted context', async () => {
    mockInvoke.mockResolvedValue('Formatted KB context');

    const { result } = renderHook(() => useKb());

    let context: string = '';
    await act(async () => {
      context = await result.current.getSearchContext('query');
    });

    expect(context).toBe('Formatted KB context');
    expect(mockInvoke).toHaveBeenCalledWith('get_search_context', { query: 'query', limit: 5 });
  });

  it('getVectorConsent returns consent status', async () => {
    const mockConsent = { enabled: true, consented_at: '2024-01-01', encryption_supported: true };
    mockInvoke.mockResolvedValue(mockConsent);

    const { result } = renderHook(() => useKb());

    let consent: unknown;
    await act(async () => {
      consent = await result.current.getVectorConsent();
    });

    expect(consent).toEqual(mockConsent);
    expect(mockInvoke).toHaveBeenCalledWith('get_vector_consent');
  });

  it('setVectorConsent invokes with correct params', async () => {
    mockInvoke.mockResolvedValue(undefined);

    const { result } = renderHook(() => useKb());

    await act(async () => {
      await result.current.setVectorConsent(true);
    });

    expect(mockInvoke).toHaveBeenCalledWith('set_vector_consent', {
      enabled: true,
      encryptionSupported: true,
    });
  });
});
