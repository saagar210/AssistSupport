import { useCallback, useState } from 'react';
import type { FormEvent } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { GgufFileInfo } from '../../../types';

type JiraFormState = {
  baseUrl: string;
  email: string;
  apiToken: string;
};

interface UseSettingsWorkspaceStateParams {
  loadModel: (modelId: string) => Promise<unknown>;
  unloadModel: () => Promise<unknown>;
  getLoadedModel: () => Promise<string | null>;
  listModels: () => Promise<string[]>;
  getContextWindow: () => Promise<number | null>;
  setContextWindow: (size: number | null) => Promise<unknown>;
  loadCustomModel: (path: string) => Promise<unknown>;
  validateGgufFile: (path: string) => Promise<GgufFileInfo>;
  setKbFolder: (folderPath: string) => Promise<unknown>;
  getKbFolder: () => Promise<string | null>;
  rebuildIndex: () => Promise<unknown>;
  getIndexStats: () => Promise<{ total_chunks: number; total_files: number } | null>;
  getVectorConsent: () => Promise<{ enabled: boolean } | null>;
  setVectorConsent: (enabled: boolean) => Promise<unknown>;
  generateEmbeddings: () => Promise<{ chunks_processed: number }>;
  downloadModel: (modelId: string) => Promise<unknown>;
  checkJiraConfig: () => Promise<boolean>;
  configureJira: (baseUrl: string, email: string, apiToken: string) => Promise<unknown>;
  disconnectJira: () => Promise<unknown>;
  initEmbeddingEngine: () => Promise<unknown>;
  loadEmbeddingModel: (path: string) => Promise<unknown>;
  unloadEmbeddingModel: () => Promise<unknown>;
  checkEmbeddingStatus: () => Promise<unknown>;
  isEmbeddingDownloaded: () => Promise<boolean>;
  getEmbeddingModelPath: (modelId: string) => Promise<string | null>;
  isEmbeddingLoaded: boolean;
  refreshDeploymentAndIntegrations: () => Promise<void>;
  loadVariables: () => Promise<void>;
  showSuccess: (message: string) => void;
  showError: (message: string) => void;
}

export function useSettingsWorkspaceState({
  loadModel,
  unloadModel,
  getLoadedModel,
  listModels,
  getContextWindow,
  setContextWindow,
  loadCustomModel,
  validateGgufFile,
  setKbFolder,
  getKbFolder,
  rebuildIndex,
  getIndexStats,
  getVectorConsent,
  setVectorConsent,
  generateEmbeddings,
  downloadModel,
  checkJiraConfig,
  configureJira,
  disconnectJira,
  initEmbeddingEngine,
  loadEmbeddingModel,
  unloadEmbeddingModel,
  checkEmbeddingStatus,
  isEmbeddingDownloaded,
  getEmbeddingModelPath,
  isEmbeddingLoaded,
  refreshDeploymentAndIntegrations,
  loadVariables,
  showSuccess,
  showError,
}: UseSettingsWorkspaceStateParams) {
  const [loadedModel, setLoadedModel] = useState<string | null>(null);
  const [downloadedModels, setDownloadedModels] = useState<string[]>([]);
  const [showOtherModels, setShowOtherModels] = useState(false);
  const [kbFolder, setKbFolderState] = useState<string | null>(null);
  const [indexStats, setIndexStats] = useState<{ total_chunks: number; total_files: number } | null>(null);
  const [vectorEnabled, setVectorEnabled] = useState(false);
  const [jiraConfigured, setJiraConfigured] = useState(false);
  const [jiraForm, setJiraForm] = useState<JiraFormState>({ baseUrl: '', email: '', apiToken: '' });
  const [contextWindowSize, setContextWindowSize] = useState<number | null>(null);
  const [embeddingDownloaded, setEmbeddingDownloaded] = useState(false);
  const [generatingEmbeddings, setGeneratingEmbeddings] = useState(false);
  const [loading, setLoading] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [backupLoading, setBackupLoading] = useState<'export' | 'import' | null>(null);

  const initializeWorkspaceState = useCallback(async () => {
    try {
      const [loaded, downloaded, folder, stats, consent, jiraConfigResult, ctxWindow, embDownloaded] = await Promise.all([
        getLoadedModel(),
        listModels(),
        getKbFolder(),
        getIndexStats().catch(() => null),
        getVectorConsent().catch(() => null),
        checkJiraConfig().catch(() => false),
        getContextWindow().catch(() => null),
        isEmbeddingDownloaded().catch(() => false),
      ]);
      setLoadedModel(loaded);
      setDownloadedModels(downloaded);
      setKbFolderState(folder);
      setIndexStats(stats);
      if (consent) {
        setVectorEnabled(consent.enabled);
      }
      setJiraConfigured(jiraConfigResult);
      setContextWindowSize(ctxWindow);
      setEmbeddingDownloaded(embDownloaded);
      await refreshDeploymentAndIntegrations();
      await checkEmbeddingStatus();
    } catch (err) {
      console.error('Failed to load settings state:', err);
    }
  }, [
    getLoadedModel,
    listModels,
    getKbFolder,
    getIndexStats,
    getVectorConsent,
    checkJiraConfig,
    getContextWindow,
    isEmbeddingDownloaded,
    refreshDeploymentAndIntegrations,
    checkEmbeddingStatus,
  ]);

  const handleVectorToggle = useCallback(async () => {
    const newValue = !vectorEnabled;
    try {
      await setVectorConsent(newValue);
      setVectorEnabled(newValue);
    } catch (err) {
      setError(`Failed to update vector consent: ${err}`);
    }
  }, [setVectorConsent, vectorEnabled]);

  const handleJiraConnect = useCallback(async (e: FormEvent) => {
    e.preventDefault();
    setError(null);
    try {
      await configureJira(jiraForm.baseUrl, jiraForm.email, jiraForm.apiToken);
      setJiraConfigured(true);
      setJiraForm({ baseUrl: '', email: '', apiToken: '' });
    } catch (err) {
      setError(`Failed to connect to Jira: ${err}`);
    }
  }, [configureJira, jiraForm.apiToken, jiraForm.baseUrl, jiraForm.email]);

  const handleJiraDisconnect = useCallback(async () => {
    setError(null);
    try {
      await disconnectJira();
      setJiraConfigured(false);
    } catch (err) {
      setError(`Failed to disconnect Jira: ${err}`);
    }
  }, [disconnectJira]);

  const handleLoadModel = useCallback(async (modelId: string) => {
    setLoading(modelId);
    setError(null);
    try {
      await loadModel(modelId);
      setLoadedModel(modelId);
    } catch (err) {
      setError(`Failed to load model: ${err}`);
    } finally {
      setLoading(null);
    }
  }, [loadModel]);

  const handleUnloadModel = useCallback(async () => {
    setLoading('unload');
    setError(null);
    try {
      await unloadModel();
      setLoadedModel(null);
    } catch (err) {
      setError(`Failed to unload model: ${err}`);
    } finally {
      setLoading(null);
    }
  }, [unloadModel]);

  const handleDownloadModel = useCallback(async (modelId: string) => {
    setError(null);
    try {
      await downloadModel(modelId);
      setDownloadedModels(prev => (prev.includes(modelId) ? prev : [...prev, modelId]));
    } catch (err) {
      setError(`Failed to download model: ${err}`);
    }
  }, [downloadModel]);

  const handleLoadCustomModel = useCallback(async () => {
    setError(null);
    setLoading('custom');
    try {
      const { open } = await import('@tauri-apps/plugin-dialog');
      const selected = await open({
        multiple: false,
        filters: [{
          name: 'GGUF Model',
          extensions: ['gguf'],
        }],
        title: 'Select GGUF Model File',
      });

      if (selected && typeof selected === 'string') {
        const validation = await validateGgufFile(selected);
        if (!validation.is_valid) {
          setError(`Invalid GGUF file: ${validation.file_name}. Please select a valid GGUF model file.`);
          return;
        }

        await loadCustomModel(selected);
        setLoadedModel(validation.file_name);
        showSuccess(`Loaded custom model: ${validation.file_name}`);
      }
    } catch (err) {
      setError(`Failed to load custom model: ${err}`);
    } finally {
      setLoading(null);
    }
  }, [loadCustomModel, showSuccess, validateGgufFile]);

  const handleSelectKbFolder = useCallback(async () => {
    setError(null);
    try {
      const { open } = await import('@tauri-apps/plugin-dialog');
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Select Knowledge Base Folder',
      });
      if (selected && typeof selected === 'string') {
        await setKbFolder(selected);
        setKbFolderState(selected);
        const stats = await getIndexStats().catch(() => null);
        setIndexStats(stats);
      }
    } catch (err) {
      setError(`Failed to set KB folder: ${err}`);
    }
  }, [getIndexStats, setKbFolder]);

  const handleRebuildIndex = useCallback(async () => {
    if (!kbFolder) return;
    setLoading('rebuild');
    setError(null);
    try {
      await rebuildIndex();
      const stats = await getIndexStats();
      setIndexStats(stats);
    } catch (err) {
      setError(`Failed to rebuild index: ${err}`);
    } finally {
      setLoading(null);
    }
  }, [getIndexStats, kbFolder, rebuildIndex]);

  const handleContextWindowChange = useCallback(async (value: string) => {
    const newSize = value === '' ? null : parseInt(value, 10);
    setError(null);
    try {
      await setContextWindow(newSize);
      setContextWindowSize(newSize);
      showSuccess('Context window updated');
    } catch (err) {
      setError(`Failed to update context window: ${err}`);
    }
  }, [setContextWindow, showSuccess]);

  const handleDownloadEmbeddingModel = useCallback(async () => {
    setError(null);
    try {
      await downloadModel('nomic-embed-text');
      setEmbeddingDownloaded(true);
      showSuccess('Embedding model downloaded');
    } catch (err) {
      setError(`Failed to download embedding model: ${err}`);
    }
  }, [downloadModel, showSuccess]);

  const handleLoadEmbeddingModel = useCallback(async () => {
    setError(null);
    try {
      await initEmbeddingEngine();
      const path = await getEmbeddingModelPath('nomic-embed-text');
      if (!path) {
        showError('Embedding model file not found. Try re-downloading.');
        return;
      }
      await loadEmbeddingModel(path);
      showSuccess('Embedding model loaded');
    } catch (err) {
      const msg = `Failed to load embedding model: ${err}`;
      showError(msg);
      setError(msg);
    }
  }, [getEmbeddingModelPath, initEmbeddingEngine, loadEmbeddingModel, showError, showSuccess]);

  const handleUnloadEmbeddingModel = useCallback(async () => {
    setError(null);
    try {
      await unloadEmbeddingModel();
      showSuccess('Embedding model unloaded');
    } catch (err) {
      setError(`Failed to unload embedding model: ${err}`);
    }
  }, [showSuccess, unloadEmbeddingModel]);

  const handleGenerateEmbeddings = useCallback(async () => {
    if (!vectorEnabled || !isEmbeddingLoaded) {
      setError('Vector search and embedding model must be enabled');
      return;
    }
    setGeneratingEmbeddings(true);
    setError(null);
    try {
      const result = await generateEmbeddings();
      showSuccess(`Generated embeddings for ${result.chunks_processed} chunks`);
    } catch (err) {
      showError(`Failed to generate embeddings: ${err}`);
    } finally {
      setGeneratingEmbeddings(false);
    }
  }, [generateEmbeddings, isEmbeddingLoaded, showError, showSuccess, vectorEnabled]);

  const handleExportBackup = useCallback(async () => {
    setBackupLoading('export');
    setError(null);
    try {
      const result = await invoke<{ drafts_count: number; templates_count: number; variables_count: number; trees_count: number }>('export_backup');
      showSuccess(`Exported ${result.drafts_count} drafts, ${result.templates_count} templates, ${result.variables_count} variables, ${result.trees_count} trees`);
    } catch (err) {
      if (String(err) !== 'Export cancelled') {
        showError(`Export failed: ${err}`);
      }
    } finally {
      setBackupLoading(null);
    }
  }, [showSuccess, showError]);

  const handleImportBackup = useCallback(async () => {
    setBackupLoading('import');
    setError(null);
    try {
      const result = await invoke<{ drafts_imported: number; templates_imported: number; variables_imported: number; trees_imported: number }>('import_backup');
      showSuccess(`Imported ${result.drafts_imported} drafts, ${result.templates_imported} templates, ${result.variables_imported} variables, ${result.trees_imported} trees`);
      await initializeWorkspaceState();
      await loadVariables();
    } catch (err) {
      if (String(err) !== 'Import cancelled') {
        showError(`Import failed: ${err}`);
      }
    } finally {
      setBackupLoading(null);
    }
  }, [initializeWorkspaceState, loadVariables, showSuccess, showError]);

  return {
    loadedModel,
    downloadedModels,
    showOtherModels,
    kbFolder,
    indexStats,
    vectorEnabled,
    jiraConfigured,
    jiraForm,
    contextWindowSize,
    embeddingDownloaded,
    generatingEmbeddings,
    loading,
    error,
    backupLoading,
    setShowOtherModels,
    setJiraForm,
    initializeWorkspaceState,
    handleVectorToggle,
    handleJiraConnect,
    handleJiraDisconnect,
    handleLoadModel,
    handleUnloadModel,
    handleDownloadModel,
    handleLoadCustomModel,
    handleSelectKbFolder,
    handleRebuildIndex,
    handleContextWindowChange,
    handleDownloadEmbeddingModel,
    handleLoadEmbeddingModel,
    handleUnloadEmbeddingModel,
    handleGenerateEmbeddings,
    handleExportBackup,
    handleImportBackup,
  };
}
