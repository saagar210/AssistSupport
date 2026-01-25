import { useState, useCallback, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import type {
  KbDocument,
  IndexStats,
  IndexResult,
  SearchResult,
  IndexedFile,
  VectorConsent,
} from '../types';

export interface IndexingProgress {
  current: number;
  total: number;
  currentFile: string | null;
  percentage: number;
}

export interface KbState {
  folderPath: string | null;
  stats: IndexStats | null;
  documents: KbDocument[];
  indexing: boolean;
  indexingProgress: IndexingProgress | null;
  searching: boolean;
  error: string | null;
}

// Event payload types from backend
interface IndexProgressStarted {
  Started: { total_files: number };
}
interface IndexProgressProcessing {
  Processing: { current: number; total: number; file_name: string };
}
interface IndexProgressCompleted {
  Completed: { indexed: number; skipped: number; errors: number };
}
interface IndexProgressError {
  Error: { file_name: string; message: string };
}
type IndexProgressEvent = IndexProgressStarted | IndexProgressProcessing | IndexProgressCompleted | IndexProgressError;

export function useKb() {
  const [state, setState] = useState<KbState>({
    folderPath: null,
    stats: null,
    documents: [],
    indexing: false,
    indexingProgress: null,
    searching: false,
    error: null,
  });

  // Listen for indexing progress events
  useEffect(() => {
    let unlisten: UnlistenFn | null = null;

    listen<IndexProgressEvent>('kb:indexing:progress', (event) => {
      const payload = event.payload;

      if ('Started' in payload) {
        setState(prev => ({
          ...prev,
          indexingProgress: {
            current: 0,
            total: payload.Started.total_files,
            currentFile: null,
            percentage: 0,
          },
        }));
      } else if ('Processing' in payload) {
        const { current, total, file_name } = payload.Processing;
        setState(prev => ({
          ...prev,
          indexingProgress: {
            current,
            total,
            currentFile: file_name,
            percentage: Math.round((current / total) * 100),
          },
        }));
      } else if ('Completed' in payload) {
        setState(prev => ({
          ...prev,
          indexingProgress: null,
        }));
      } else if ('Error' in payload) {
        // Log error but continue indexing
        console.warn(`Indexing error for ${payload.Error.file_name}: ${payload.Error.message}`);
      }
    }).then(fn => {
      unlisten = fn;
    });

    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  const loadKbInfo = useCallback(async () => {
    try {
      const folder = await invoke<string | null>('get_kb_folder');
      const stats = await invoke<IndexStats>('get_kb_stats');
      const docs = await invoke<KbDocument[]>('list_kb_documents');

      setState(prev => ({
        ...prev,
        folderPath: folder,
        stats,
        documents: docs,
        error: null,
      }));
    } catch (e) {
      setState(prev => ({
        ...prev,
        error: String(e),
      }));
    }
  }, []);

  const getKbFolder = useCallback(async (): Promise<string | null> => {
    try {
      return await invoke<string | null>('get_kb_folder');
    } catch {
      return null;
    }
  }, []);

  const getIndexStats = useCallback(async (): Promise<{ total_chunks: number; total_files: number }> => {
    const stats = await invoke<IndexStats>('get_kb_stats');
    return {
      total_chunks: stats.chunk_count,
      total_files: stats.document_count,
    };
  }, []);

  const listFiles = useCallback(async (): Promise<IndexedFile[]> => {
    try {
      const docs = await invoke<KbDocument[]>('list_kb_documents');
      return docs.map(doc => ({
        file_path: doc.file_path,
        title: doc.title ?? null,
        chunk_count: doc.chunk_count ?? 0,
        indexed_at: doc.indexed_at ?? new Date().toISOString(),
      }));
    } catch {
      return [];
    }
  }, []);

  const setKbFolder = useCallback(async (path: string) => {
    try {
      await invoke('set_kb_folder', { folderPath: path });
      setState(prev => ({
        ...prev,
        folderPath: path,
        error: null,
      }));
    } catch (e) {
      setState(prev => ({
        ...prev,
        error: String(e),
      }));
      throw e;
    }
  }, []);

  const indexKb = useCallback(async (): Promise<IndexResult> => {
    setState(prev => ({ ...prev, indexing: true, error: null }));
    try {
      const result = await invoke<IndexResult>('index_kb');

      // Refresh stats and documents after indexing
      const stats = await invoke<IndexStats>('get_kb_stats');
      const docs = await invoke<KbDocument[]>('list_kb_documents');

      setState(prev => ({
        ...prev,
        indexing: false,
        stats,
        documents: docs,
      }));
      return result;
    } catch (e) {
      setState(prev => ({
        ...prev,
        indexing: false,
        error: String(e),
      }));
      throw e;
    }
  }, []);

  const rebuildIndex = useCallback(async (): Promise<void> => {
    await indexKb();
  }, [indexKb]);

  const generateEmbeddings = useCallback(async (): Promise<{ chunks_processed: number; vectors_created: number }> => {
    setState(prev => ({ ...prev, indexing: true, error: null }));
    try {
      const result = await invoke<{ chunks_processed: number; vectors_created: number }>('generate_kb_embeddings');
      setState(prev => ({ ...prev, indexing: false }));
      return result;
    } catch (e) {
      setState(prev => ({
        ...prev,
        indexing: false,
        error: String(e),
      }));
      throw e;
    }
  }, []);

  const search = useCallback(async (
    query: string,
    limit?: number,
    namespaceId?: string | null
  ): Promise<SearchResult[]> => {
    setState(prev => ({ ...prev, searching: true, error: null }));
    try {
      const results = await invoke<SearchResult[]>('search_kb', {
        query,
        limit: limit ?? 10,
        namespaceId: namespaceId ?? null,
      });
      setState(prev => ({ ...prev, searching: false }));
      return results;
    } catch (e) {
      setState(prev => ({
        ...prev,
        searching: false,
        error: String(e),
      }));
      throw e;
    }
  }, []);

  const getSearchContext = useCallback(async (
    query: string,
    limit?: number,
    namespaceId?: string | null
  ): Promise<string> => {
    try {
      return await invoke<string>('get_search_context', {
        query,
        limit: limit ?? 5,
        namespaceId: namespaceId ?? null,
      });
    } catch (e) {
      setState(prev => ({
        ...prev,
        error: String(e),
      }));
      throw e;
    }
  }, []);

  const getVectorConsent = useCallback(async (): Promise<VectorConsent> => {
    return invoke<VectorConsent>('get_vector_consent');
  }, []);

  const setVectorConsent = useCallback(async (enabled: boolean): Promise<void> => {
    await invoke('set_vector_consent', {
      enabled,
      encryptionSupported: true, // Always true since we use SQLCipher
    });
  }, []);

  const removeDocument = useCallback(async (filePath: string): Promise<boolean> => {
    try {
      const removed = await invoke<boolean>('remove_kb_document', { filePath });

      // Refresh stats and documents after removal
      if (removed) {
        const stats = await invoke<IndexStats>('get_kb_stats');
        const docs = await invoke<KbDocument[]>('list_kb_documents');
        setState(prev => ({
          ...prev,
          stats,
          documents: docs,
        }));
      }

      return removed;
    } catch (e) {
      setState(prev => ({
        ...prev,
        error: String(e),
      }));
      throw e;
    }
  }, []);

  return {
    ...state,
    loadKbInfo,
    getKbFolder,
    getIndexStats,
    listFiles,
    setKbFolder,
    indexKb,
    rebuildIndex,
    generateEmbeddings,
    search,
    getSearchContext,
    getVectorConsent,
    setVectorConsent,
    removeDocument,
  };
}
