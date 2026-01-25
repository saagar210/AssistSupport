import { useState, useCallback, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type {
  Namespace,
  NamespaceWithCounts as BackendNamespaceWithCounts,
  IngestSource,
  KbDocumentInfo,
  DocumentChunk,
} from '../types';

// Frontend type with camelCase properties
export interface NamespaceWithCounts extends Namespace {
  documentCount: number;
  sourceCount: number;
}

export interface KnowledgeState {
  namespaces: NamespaceWithCounts[];
  selectedNamespace: string | null;
  sources: IngestSource[];
  documents: KbDocumentInfo[];
  selectedDocument: KbDocumentInfo | null;
  chunks: DocumentChunk[];
  loading: boolean;
  error: string | null;
}

// Simple cache for expensive operations
let namespacesCache: { data: NamespaceWithCounts[]; timestamp: number } | null = null;
const CACHE_TTL = 30000; // 30 seconds

function isCacheValid(): boolean {
  return namespacesCache !== null && (Date.now() - namespacesCache.timestamp) < CACHE_TTL;
}

function invalidateCache() {
  namespacesCache = null;
}

export function useKnowledge() {
  const [state, setState] = useState<KnowledgeState>({
    namespaces: [],
    selectedNamespace: null,
    sources: [],
    documents: [],
    selectedDocument: null,
    chunks: [],
    loading: false,
    error: null,
  });

  // Load all namespaces with counts using optimized single query
  const loadNamespaces = useCallback(async (forceRefresh = false) => {
    // Use cache if valid and not forcing refresh
    if (!forceRefresh && isCacheValid()) {
      setState(prev => ({
        ...prev,
        namespaces: namespacesCache!.data,
        loading: false,
      }));
      return namespacesCache!.data;
    }

    setState(prev => ({ ...prev, loading: true, error: null }));
    try {
      // Use optimized backend command that returns counts in single query
      const backendNamespaces = await invoke<BackendNamespaceWithCounts[]>('list_namespaces_with_counts');

      // Map snake_case to camelCase for frontend consistency
      const namespacesWithCounts: NamespaceWithCounts[] = backendNamespaces.map(ns => ({
        ...ns,
        documentCount: ns.document_count,
        sourceCount: ns.source_count,
      }));

      // Update cache
      namespacesCache = {
        data: namespacesWithCounts,
        timestamp: Date.now(),
      };

      setState(prev => ({
        ...prev,
        namespaces: namespacesWithCounts,
        loading: false,
      }));
      return namespacesWithCounts;
    } catch (e) {
      setState(prev => ({ ...prev, loading: false, error: String(e) }));
      throw e;
    }
  }, []);

  // Select a namespace and load its content
  const selectNamespace = useCallback(async (namespaceId: string | null) => {
    setState(prev => ({
      ...prev,
      selectedNamespace: namespaceId,
      selectedDocument: null,
      chunks: [],
      loading: true,
      error: null,
    }));

    if (!namespaceId) {
      setState(prev => ({
        ...prev,
        sources: [],
        documents: [],
        loading: false,
      }));
      return;
    }

    try {
      // Parallel fetch for sources and documents
      const [sources, documents] = await Promise.all([
        invoke<IngestSource[]>('list_ingest_sources', { namespaceId }),
        invoke<KbDocumentInfo[]>('list_kb_documents', {
          namespaceId,
          sourceId: null,
        }),
      ]);

      setState(prev => ({
        ...prev,
        sources,
        documents,
        loading: false,
      }));
    } catch (e) {
      setState(prev => ({ ...prev, loading: false, error: String(e) }));
    }
  }, []);

  // Select a document and load its chunks
  const selectDocument = useCallback(async (document: KbDocumentInfo | null) => {
    setState(prev => ({
      ...prev,
      selectedDocument: document,
      chunks: [],
      loading: document !== null,
    }));

    if (!document) return;

    try {
      const chunks = await invoke<DocumentChunk[]>('get_document_chunks', {
        documentId: document.id,
      });
      setState(prev => ({ ...prev, chunks, loading: false }));
    } catch (e) {
      setState(prev => ({ ...prev, loading: false, error: String(e) }));
    }
  }, []);

  // Delete a namespace
  const deleteNamespace = useCallback(async (namespaceId: string): Promise<void> => {
    try {
      await invoke('delete_namespace', { name: namespaceId });
      invalidateCache(); // Invalidate cache on mutation
      setState(prev => ({
        ...prev,
        namespaces: prev.namespaces.filter(ns => ns.id !== namespaceId),
        selectedNamespace: prev.selectedNamespace === namespaceId ? null : prev.selectedNamespace,
      }));
    } catch (e) {
      setState(prev => ({ ...prev, error: String(e) }));
      throw e;
    }
  }, []);

  // Delete a source
  const deleteSource = useCallback(async (sourceId: string): Promise<void> => {
    try {
      await invoke('delete_ingest_source', { sourceId });
      invalidateCache(); // Invalidate cache on mutation
      setState(prev => ({
        ...prev,
        sources: prev.sources.filter(s => s.id !== sourceId),
        documents: prev.documents.filter(d => d.source_id !== sourceId),
      }));
    } catch (e) {
      setState(prev => ({ ...prev, error: String(e) }));
      throw e;
    }
  }, []);

  // Delete a document
  const deleteDocument = useCallback(async (documentId: string): Promise<void> => {
    try {
      await invoke('delete_kb_document', { documentId });
      invalidateCache(); // Invalidate cache on mutation
      setState(prev => ({
        ...prev,
        documents: prev.documents.filter(d => d.id !== documentId),
        selectedDocument: prev.selectedDocument?.id === documentId ? null : prev.selectedDocument,
        chunks: prev.selectedDocument?.id === documentId ? [] : prev.chunks,
      }));
    } catch (e) {
      setState(prev => ({ ...prev, error: String(e) }));
      throw e;
    }
  }, []);

  // Clear all knowledge data
  const clearAll = useCallback(async (namespaceId?: string): Promise<void> => {
    try {
      await invoke('clear_knowledge_data', { namespaceId });
      invalidateCache(); // Invalidate cache on mutation
      if (namespaceId) {
        // Reload the specific namespace
        await selectNamespace(namespaceId);
        await loadNamespaces(true);
      } else {
        // Clear everything
        setState(prev => ({
          ...prev,
          documents: [],
          sources: [],
          selectedDocument: null,
          chunks: [],
        }));
        await loadNamespaces(true);
      }
    } catch (e) {
      setState(prev => ({ ...prev, error: String(e) }));
      throw e;
    }
  }, [loadNamespaces, selectNamespace]);

  // Memoize the return value to prevent unnecessary re-renders
  const actions = useMemo(() => ({
    loadNamespaces,
    selectNamespace,
    selectDocument,
    deleteNamespace,
    deleteSource,
    deleteDocument,
    clearAll,
  }), [loadNamespaces, selectNamespace, selectDocument, deleteNamespace, deleteSource, deleteDocument, clearAll]);

  return {
    ...state,
    ...actions,
  };
}
