import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type {
  KbDocument,
  IndexStats,
  IndexResult,
  SearchResult,
  IndexedFile,
  VectorConsent,
} from '../types';

export interface KbState {
  folderPath: string | null;
  stats: IndexStats | null;
  documents: KbDocument[];
  indexing: boolean;
  searching: boolean;
  error: string | null;
}

export function useKb() {
  const [state, setState] = useState<KbState>({
    folderPath: null,
    stats: null,
    documents: [],
    indexing: false,
    searching: false,
    error: null,
  });

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
      total_chunks: stats.total_chunks,
      total_files: stats.total_files,
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
    limit?: number
  ): Promise<SearchResult[]> => {
    setState(prev => ({ ...prev, searching: true, error: null }));
    try {
      const results = await invoke<SearchResult[]>('search_kb', {
        query,
        limit: limit ?? 10,
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
    limit?: number
  ): Promise<string> => {
    try {
      return await invoke<string>('get_search_context', {
        query,
        limit: limit ?? 5,
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
