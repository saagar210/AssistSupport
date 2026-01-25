import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type {
  Namespace,
  IngestSource,
  IngestResult,
  BatchIngestResult,
  KbDocumentInfo,
  DocumentChunk,
} from '../types';

export interface IngestState {
  namespaces: Namespace[];
  sources: IngestSource[];
  documents: KbDocumentInfo[];
  ingesting: boolean;
  ytdlpAvailable: boolean | null;
  error: string | null;
}

export function useIngest() {
  const [state, setState] = useState<IngestState>({
    namespaces: [],
    sources: [],
    documents: [],
    ingesting: false,
    ytdlpAvailable: null,
    error: null,
  });

  // Load namespaces
  const loadNamespaces = useCallback(async () => {
    try {
      const namespaces = await invoke<Namespace[]>('list_namespaces');
      setState(prev => ({ ...prev, namespaces, error: null }));
      return namespaces;
    } catch (e) {
      setState(prev => ({ ...prev, error: String(e) }));
      throw e;
    }
  }, []);

  // Create a new namespace
  const createNamespace = useCallback(async (
    name: string,
    description?: string,
    color?: string
  ): Promise<Namespace> => {
    try {
      const namespace = await invoke<Namespace>('create_namespace', {
        name,
        description,
        color,
      });
      setState(prev => ({
        ...prev,
        namespaces: [...prev.namespaces, namespace],
        error: null,
      }));
      return namespace;
    } catch (e) {
      setState(prev => ({ ...prev, error: String(e) }));
      throw e;
    }
  }, []);

  // Rename a namespace
  const renameNamespace = useCallback(async (oldName: string, newName: string): Promise<void> => {
    try {
      await invoke('rename_namespace', { oldName, newName });
      await loadNamespaces();
    } catch (e) {
      setState(prev => ({ ...prev, error: String(e) }));
      throw e;
    }
  }, [loadNamespaces]);

  // Delete a namespace
  const deleteNamespace = useCallback(async (name: string): Promise<void> => {
    try {
      await invoke('delete_namespace', { name });
      setState(prev => ({
        ...prev,
        namespaces: prev.namespaces.filter(n => n.id !== name),
        error: null,
      }));
    } catch (e) {
      setState(prev => ({ ...prev, error: String(e) }));
      throw e;
    }
  }, []);

  // Load sources
  const loadSources = useCallback(async (namespaceId?: string) => {
    try {
      const sources = await invoke<IngestSource[]>('list_ingest_sources', {
        namespaceId,
      });
      setState(prev => ({ ...prev, sources, error: null }));
      return sources;
    } catch (e) {
      setState(prev => ({ ...prev, error: String(e) }));
      throw e;
    }
  }, []);

  // Delete a source
  const deleteSource = useCallback(async (sourceId: string): Promise<void> => {
    try {
      await invoke('delete_ingest_source', { sourceId });
      setState(prev => ({
        ...prev,
        sources: prev.sources.filter(s => s.id !== sourceId),
        error: null,
      }));
    } catch (e) {
      setState(prev => ({ ...prev, error: String(e) }));
      throw e;
    }
  }, []);

  // Load documents
  const loadDocuments = useCallback(async (
    namespaceId?: string,
    sourceId?: string
  ) => {
    try {
      const documents = await invoke<KbDocumentInfo[]>('list_kb_documents', {
        namespaceId,
        sourceId,
      });
      setState(prev => ({ ...prev, documents, error: null }));
      return documents;
    } catch (e) {
      setState(prev => ({ ...prev, error: String(e) }));
      throw e;
    }
  }, []);

  // Get document chunks
  const getDocumentChunks = useCallback(async (documentId: string): Promise<DocumentChunk[]> => {
    try {
      return await invoke<DocumentChunk[]>('get_document_chunks', { documentId });
    } catch (e) {
      setState(prev => ({ ...prev, error: String(e) }));
      throw e;
    }
  }, []);

  // Delete a document
  const deleteDocument = useCallback(async (documentId: string): Promise<void> => {
    try {
      await invoke('delete_kb_document', { documentId });
      setState(prev => ({
        ...prev,
        documents: prev.documents.filter(d => d.id !== documentId),
        error: null,
      }));
    } catch (e) {
      setState(prev => ({ ...prev, error: String(e) }));
      throw e;
    }
  }, []);

  // Clear all knowledge data
  const clearKnowledgeData = useCallback(async (namespaceId?: string): Promise<void> => {
    try {
      await invoke('clear_knowledge_data', { namespaceId });
      setState(prev => ({
        ...prev,
        documents: namespaceId
          ? prev.documents.filter(d => d.namespace_id !== namespaceId)
          : [],
        error: null,
      }));
    } catch (e) {
      setState(prev => ({ ...prev, error: String(e) }));
      throw e;
    }
  }, []);

  // Check yt-dlp availability
  const checkYtdlp = useCallback(async (): Promise<boolean> => {
    try {
      const available = await invoke<boolean>('check_ytdlp_available');
      setState(prev => ({ ...prev, ytdlpAvailable: available }));
      return available;
    } catch {
      setState(prev => ({ ...prev, ytdlpAvailable: false }));
      return false;
    }
  }, []);

  // Ingest a URL
  const ingestUrl = useCallback(async (
    url: string,
    namespaceId: string
  ): Promise<IngestResult> => {
    setState(prev => ({ ...prev, ingesting: true, error: null }));
    try {
      const result = await invoke<IngestResult>('ingest_url', {
        url,
        namespaceId,
      });
      setState(prev => ({ ...prev, ingesting: false }));
      return result;
    } catch (e) {
      setState(prev => ({ ...prev, ingesting: false, error: String(e) }));
      throw e;
    }
  }, []);

  // Ingest a YouTube video
  const ingestYoutube = useCallback(async (
    url: string,
    namespaceId: string
  ): Promise<IngestResult> => {
    setState(prev => ({ ...prev, ingesting: true, error: null }));
    try {
      const result = await invoke<IngestResult>('ingest_youtube', {
        url,
        namespaceId,
      });
      setState(prev => ({ ...prev, ingesting: false }));
      return result;
    } catch (e) {
      setState(prev => ({ ...prev, ingesting: false, error: String(e) }));
      throw e;
    }
  }, []);

  // Ingest a GitHub repository
  const ingestGithub = useCallback(async (
    repoPath: string,
    namespaceId: string
  ): Promise<IngestResult[]> => {
    setState(prev => ({ ...prev, ingesting: true, error: null }));
    try {
      const results = await invoke<IngestResult[]>('ingest_github', {
        repoPath,
        namespaceId,
      });
      setState(prev => ({ ...prev, ingesting: false }));
      return results;
    } catch (e) {
      setState(prev => ({ ...prev, ingesting: false, error: String(e) }));
      throw e;
    }
  }, []);

  // Process a YAML source file
  const processSourceFile = useCallback(async (
    filePath: string
  ): Promise<BatchIngestResult> => {
    setState(prev => ({ ...prev, ingesting: true, error: null }));
    try {
      const result = await invoke<BatchIngestResult>('process_source_file', {
        filePath,
      });
      setState(prev => ({ ...prev, ingesting: false }));
      return result;
    } catch (e) {
      setState(prev => ({ ...prev, ingesting: false, error: String(e) }));
      throw e;
    }
  }, []);

  return {
    ...state,
    loadNamespaces,
    createNamespace,
    renameNamespace,
    deleteNamespace,
    loadSources,
    deleteSource,
    loadDocuments,
    getDocumentChunks,
    deleteDocument,
    clearKnowledgeData,
    checkYtdlp,
    ingestUrl,
    ingestYoutube,
    ingestGithub,
    processSourceFile,
  };
}
