import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Button } from '../shared/Button';
import { useLlm } from '../../hooks/useLlm';
import { useKb } from '../../hooks/useKb';
import { useDownload } from '../../hooks/useDownload';
import { useJira } from '../../hooks/useJira';
import { useEmbedding } from '../../hooks/useEmbedding';
import { useCustomVariables } from '../../hooks/useCustomVariables';
import { useFeatureOps } from '../../hooks/useFeatureOps';
import { useTheme } from '../../contexts/ThemeContext';
import { useToastContext } from '../../contexts/ToastContext';
import type {
  AuditEntry,
  CustomVariable,
  DeploymentHealthSummary,
  IntegrationConfigRecord,
  ModelInfo,
  StartupMetricsResult,
} from '../../types';
import './SettingsTab.css';

const AVAILABLE_MODELS: ModelInfo[] = [
  {
    id: 'llama-3.2-1b-instruct',
    name: 'Llama 3.2 1B Instruct',
    size: '1.3 GB',
    description: 'Fast, lightweight model for quick responses',
  },
  {
    id: 'llama-3.2-3b-instruct',
    name: 'Llama 3.2 3B Instruct',
    size: '2.0 GB',
    description: 'Balanced performance and quality',
  },
  {
    id: 'phi-3-mini-4k-instruct',
    name: 'Phi-3 Mini 4K',
    size: '2.4 GB',
    description: 'Microsoft model, good for reasoning',
  },
];

// Audit event types can be either a plain string (unit variants like "key_generated")
// or an object (data variants like { custom: "value" }). Normalize for display.
function formatAuditEvent(event: string | Record<string, string>): string {
  if (typeof event === 'string') return event;
  if (typeof event === 'object' && event !== null) {
    const key = Object.keys(event)[0];
    return key ? `${key}: ${event[key]}` : JSON.stringify(event);
  }
  return String(event);
}

const CONTEXT_WINDOW_OPTIONS = [
  { value: null, label: 'Model Default' },
  { value: 2048, label: '2K (2,048 tokens)' },
  { value: 4096, label: '4K (4,096 tokens)' },
  { value: 8192, label: '8K (8,192 tokens)' },
  { value: 16384, label: '16K (16,384 tokens)' },
  { value: 32768, label: '32K (32,768 tokens)' },
];

// Helper to format bytes for display
function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
}

// Helper to format download speed
function formatSpeed(bps: number): string {
  if (bps === 0) return '';
  return `${formatBytes(bps)}/s`;
}

export function SettingsTab() {
  const { loadModel, unloadModel, getLoadedModel, listModels, getContextWindow, setContextWindow, loadCustomModel, validateGgufFile } = useLlm();
  const { setKbFolder, getKbFolder, rebuildIndex, getIndexStats, getVectorConsent, setVectorConsent, generateEmbeddings } = useKb();
  const { downloadModel, downloadProgress, isDownloading, cancelDownload } = useDownload();
  const { checkConfiguration: checkJiraConfig, configure: configureJira, disconnect: disconnectJira, config: jiraConfig, loading: jiraLoading } = useJira();
  const {
    initEngine: initEmbeddingEngine,
    loadModel: loadEmbeddingModel,
    unloadModel: unloadEmbeddingModel,
    checkModelStatus: checkEmbeddingStatus,
    isModelDownloaded: isEmbeddingDownloaded,
    getModelPath: getEmbeddingModelPath,
    isLoaded: isEmbeddingLoaded,
    modelInfo: embeddingModelInfo,
    loading: embeddingLoading,
  } = useEmbedding();
  const {
    getDeploymentHealthSummary,
    runDeploymentPreflight,
    listIntegrations,
    configureIntegration,
  } = useFeatureOps();
  const { theme, setTheme } = useTheme();
  const { success: showSuccess, error: showError } = useToastContext();
  const {
    variables: customVariables,
    loadVariables,
    saveVariable,
    deleteVariable,
  } = useCustomVariables();

  const [loadedModel, setLoadedModel] = useState<string | null>(null);
  const [downloadedModels, setDownloadedModels] = useState<string[]>([]);
  const [kbFolder, setKbFolderState] = useState<string | null>(null);
  const [indexStats, setIndexStats] = useState<{ total_chunks: number; total_files: number } | null>(null);
  const [vectorEnabled, setVectorEnabled] = useState(false);
  const [jiraConfigured, setJiraConfigured] = useState(false);
  const [jiraForm, setJiraForm] = useState({ baseUrl: '', email: '', apiToken: '' });
  const [contextWindowSize, setContextWindowSize] = useState<number | null>(null);
  const [embeddingDownloaded, setEmbeddingDownloaded] = useState(false);
  const [generatingEmbeddings, setGeneratingEmbeddings] = useState(false);
  const [loading, setLoading] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [backupLoading, setBackupLoading] = useState<'export' | 'import' | null>(null);
  const [auditEntries, setAuditEntries] = useState<AuditEntry[]>([]);
  const [auditLoading, setAuditLoading] = useState(false);
  const [auditExporting, setAuditExporting] = useState(false);

  // Startup metrics and session state
  const [startupMetrics, setStartupMetrics] = useState<StartupMetricsResult | null>(null);
  const [lockingApp, setLockingApp] = useState(false);
  const [deploymentHealth, setDeploymentHealth] = useState<DeploymentHealthSummary | null>(null);
  const [deployPreflightChecks, setDeployPreflightChecks] = useState<string[]>([]);
  const [deployPreflightRunning, setDeployPreflightRunning] = useState(false);
  const [integrations, setIntegrations] = useState<IntegrationConfigRecord[]>([]);

  // Custom variables state
  const [editingVariable, setEditingVariable] = useState<CustomVariable | null>(null);
  const [variableForm, setVariableForm] = useState({ name: '', value: '' });
  const [showVariableForm, setShowVariableForm] = useState(false);
  const [variableFormError, setVariableFormError] = useState<string | null>(null);

  const loadAuditEntries = useCallback(async () => {
    setAuditLoading(true);
    try {
      const entries = await invoke<AuditEntry[]>('get_audit_entries', { limit: 200 });
      setAuditEntries(entries ?? []);
    } catch (err) {
      showError(`Failed to load audit logs: ${err}`);
    } finally {
      setAuditLoading(false);
    }
  }, [showError]);

  useEffect(() => {
    Promise.resolve(loadInitialState()).catch(err => console.error('Settings init failed:', err));
    Promise.resolve(loadVariables()).catch(err => console.error('Variables load failed:', err));
    Promise.resolve(loadAuditEntries()).catch(err => console.error('Audit load failed:', err));
    invoke<StartupMetricsResult>('get_startup_metrics')
      .then(m => setStartupMetrics(m))
      .catch(() => {});
  }, [loadVariables, loadAuditEntries]);

  async function loadInitialState() {
    try {
      const [loaded, downloaded, folder, stats, consent, jiraConfigResult, ctxWindow, embDownloaded, deployHealth, integrationsList] = await Promise.all([
        getLoadedModel(),
        listModels(),
        getKbFolder(),
        getIndexStats().catch(() => null),
        getVectorConsent().catch(() => null),
        checkJiraConfig().catch(() => false),
        getContextWindow().catch(() => null),
        isEmbeddingDownloaded().catch(() => false),
        getDeploymentHealthSummary().catch(() => null),
        listIntegrations().catch(() => []),
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
      setDeploymentHealth(deployHealth);
      setIntegrations(integrationsList);

      // Check embedding model status
      await checkEmbeddingStatus();
    } catch (err) {
      console.error('Failed to load settings state:', err);
    }
  }

  async function handleVectorToggle() {
    const newValue = !vectorEnabled;
    try {
      await setVectorConsent(newValue);
      setVectorEnabled(newValue);
    } catch (err) {
      setError(`Failed to update vector consent: ${err}`);
    }
  }

  async function handleJiraConnect(e: React.FormEvent) {
    e.preventDefault();
    setError(null);
    try {
      await configureJira(jiraForm.baseUrl, jiraForm.email, jiraForm.apiToken);
      setJiraConfigured(true);
      setJiraForm({ baseUrl: '', email: '', apiToken: '' });
    } catch (err) {
      setError(`Failed to connect to Jira: ${err}`);
    }
  }

  async function handleJiraDisconnect() {
    setError(null);
    try {
      await disconnectJira();
      setJiraConfigured(false);
    } catch (err) {
      setError(`Failed to disconnect Jira: ${err}`);
    }
  }

  async function handleLoadModel(modelId: string) {
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
  }

  async function handleUnloadModel() {
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
  }

  async function handleDownloadModel(modelId: string) {
    setError(null);
    try {
      await downloadModel(modelId);
      setDownloadedModels(prev => [...prev, modelId]);
    } catch (err) {
      setError(`Failed to download model: ${err}`);
    }
  }

  async function handleLoadCustomModel() {
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
        // Validate the file first
        const validation = await validateGgufFile(selected);
        if (!validation.is_valid) {
          setError(`Invalid GGUF file: ${validation.file_name}. Please select a valid GGUF model file.`);
          return;
        }

        // Load the model
        await loadCustomModel(selected);
        setLoadedModel(validation.file_name);
        showSuccess(`Loaded custom model: ${validation.file_name}`);
      }
    } catch (err) {
      setError(`Failed to load custom model: ${err}`);
    } finally {
      setLoading(null);
    }
  }

  async function handleSelectKbFolder() {
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
  }

  async function handleRebuildIndex() {
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
  }

  async function handleContextWindowChange(value: string) {
    const newSize = value === '' ? null : parseInt(value, 10);
    setError(null);
    try {
      await setContextWindow(newSize);
      setContextWindowSize(newSize);
      showSuccess('Context window updated');
    } catch (err) {
      setError(`Failed to update context window: ${err}`);
    }
  }

  async function handleDownloadEmbeddingModel() {
    setError(null);
    try {
      await downloadModel('nomic-embed-text');
      setEmbeddingDownloaded(true);
      showSuccess('Embedding model downloaded');
    } catch (err) {
      setError(`Failed to download embedding model: ${err}`);
    }
  }

  async function handleLoadEmbeddingModel() {
    setError(null);
    try {
      // Engine is initialized at startup; this is idempotent
      await initEmbeddingEngine();
      // Get model path
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
  }

  async function handleUnloadEmbeddingModel() {
    setError(null);
    try {
      await unloadEmbeddingModel();
      showSuccess('Embedding model unloaded');
    } catch (err) {
      setError(`Failed to unload embedding model: ${err}`);
    }
  }

  async function handleGenerateEmbeddings() {
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
  }

  // Custom variable handlers
  const handleEditVariable = useCallback((variable: CustomVariable) => {
    setEditingVariable(variable);
    setVariableForm({ name: variable.name, value: variable.value });
    setShowVariableForm(true);
    setVariableFormError(null);
  }, []);

  const handleAddVariable = useCallback(() => {
    setEditingVariable(null);
    setVariableForm({ name: '', value: '' });
    setShowVariableForm(true);
    setVariableFormError(null);
  }, []);

  const handleCancelVariableForm = useCallback(() => {
    setShowVariableForm(false);
    setEditingVariable(null);
    setVariableForm({ name: '', value: '' });
    setVariableFormError(null);
  }, []);

  const handleSaveVariable = useCallback(async () => {
    const name = variableForm.name.trim();
    const value = variableForm.value.trim();

    // Validate name format (alphanumeric and underscores only)
    if (!/^[a-zA-Z_][a-zA-Z0-9_]*$/.test(name)) {
      setVariableFormError('Name must start with a letter or underscore and contain only letters, numbers, and underscores');
      return;
    }

    if (!value) {
      setVariableFormError('Value is required');
      return;
    }

    // Check for duplicate name (except when editing the same variable)
    const isDuplicate = customVariables.some(
      (v) => v.name.toLowerCase() === name.toLowerCase() && v.id !== editingVariable?.id
    );
    if (isDuplicate) {
      setVariableFormError('A variable with this name already exists');
      return;
    }

    const success = await saveVariable(name, value, editingVariable?.id);
    if (success) {
      showSuccess(editingVariable ? 'Variable updated' : 'Variable created');
      handleCancelVariableForm();
    } else {
      setVariableFormError('Failed to save variable');
    }
  }, [variableForm, editingVariable, customVariables, saveVariable, showSuccess, handleCancelVariableForm]);

  const handleDeleteVariable = useCallback(async (variableId: string) => {
    const success = await deleteVariable(variableId);
    if (success) {
      showSuccess('Variable deleted');
    } else {
      showError('Failed to delete variable');
    }
  }, [deleteVariable, showSuccess, showError]);

  // Backup handlers
  const handleExportBackup = useCallback(async () => {
    setBackupLoading('export');
    setError(null);
    try {
      const result = await invoke<{ drafts_count: number; templates_count: number; variables_count: number; trees_count: number; path: string }>('export_backup');
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
      // Reload data
      loadInitialState();
      loadVariables();
    } catch (err) {
      if (String(err) !== 'Import cancelled') {
        showError(`Import failed: ${err}`);
      }
    } finally {
      setBackupLoading(null);
    }
  }, [showSuccess, showError, loadVariables]);

  const handleExportAuditLog = useCallback(async () => {
    setAuditExporting(true);
    try {
      const { save } = await import('@tauri-apps/plugin-dialog');
      const path = await save({
        title: 'Export Audit Log',
        defaultPath: 'assist-support-audit.json',
        filters: [{ name: 'JSON', extensions: ['json'] }],
      });
      if (!path) {
        setAuditExporting(false);
        return;
      }
      const output = await invoke<string>('export_audit_log', { exportPath: path });
      showSuccess(`Audit log exported to ${output}`);
    } catch (err) {
      if (String(err) !== 'Export cancelled') {
        showError(`Audit export failed: ${err}`);
      }
    } finally {
      setAuditExporting(false);
    }
  }, [showSuccess, showError]);

  const handleRunDeploymentPreflight = useCallback(async () => {
    setDeployPreflightRunning(true);
    try {
      const result = await runDeploymentPreflight('stable');
      setDeployPreflightChecks(result.checks);
      const latest = await getDeploymentHealthSummary().catch(() => null);
      setDeploymentHealth(latest);
      if (result.ok) {
        showSuccess('Deployment preflight passed');
      } else {
        showError('Deployment preflight reported failures');
      }
    } catch (err) {
      showError(`Deployment preflight failed: ${err}`);
    } finally {
      setDeployPreflightRunning(false);
    }
  }, [runDeploymentPreflight, getDeploymentHealthSummary, showSuccess, showError]);

  const handleToggleIntegration = useCallback(async (integrationType: string, enabled: boolean) => {
    try {
      await configureIntegration(integrationType, enabled);
      const updated = await listIntegrations();
      setIntegrations(updated);
    } catch (err) {
      showError(`Failed to update ${integrationType}: ${err}`);
    }
  }, [configureIntegration, listIntegrations, showError]);

  return (
    <div className="settings-tab">
      {error && <div className="settings-error">{error}</div>}

      <section className="settings-section">
        <h2>Appearance</h2>
        <p className="settings-description">
          Choose your preferred color theme.
        </p>
        <div className="theme-selector">
          <label className="theme-option">
            <input
              type="radio"
              name="theme"
              value="light"
              checked={theme === 'light'}
              onChange={() => setTheme('light')}
            />
            <span>Light</span>
          </label>
          <label className="theme-option">
            <input
              type="radio"
              name="theme"
              value="dark"
              checked={theme === 'dark'}
              onChange={() => setTheme('dark')}
            />
            <span>Dark</span>
          </label>
          <label className="theme-option">
            <input
              type="radio"
              name="theme"
              value="system"
              checked={theme === 'system'}
              onChange={() => setTheme('system')}
            />
            <span>System</span>
          </label>
        </div>
      </section>

      <section className="settings-section">
        <h2>Language Model</h2>
        <p className="settings-description">
          Select and load a language model for generating responses.
        </p>

        {loadedModel && (
          <div className="loaded-model-banner">
            <span>Currently loaded: <strong>{loadedModel}</strong></span>
            <Button
              variant="secondary"
              size="small"
              onClick={handleUnloadModel}
              disabled={loading === 'unload'}
            >
              {loading === 'unload' ? 'Unloading...' : 'Unload'}
            </Button>
          </div>
        )}

        <div className="model-list">
          {AVAILABLE_MODELS.map(model => {
            const isDownloaded = downloadedModels.includes(model.id);
            const isLoaded = loadedModel === model.id;
            const isLoadingThis = loading === model.id;
            const isDownloadingThis = isDownloading && downloadProgress?.model_id === model.id;

            return (
              <div key={model.id} className={`model-card ${isLoaded ? 'loaded' : ''}`}>
                <div className="model-info">
                  <h3>{model.name}</h3>
                  <p>{model.description}</p>
                  <span className="model-size">{model.size}</span>
                </div>
                <div className="model-actions">
                  {isDownloadingThis ? (
                    <div className="download-progress-container">
                      <div className="download-progress">
                        <div
                          className="download-bar"
                          style={{ width: `${downloadProgress?.percent || 0}%` }}
                        />
                        <span className="download-percent">{Math.round(downloadProgress?.percent || 0)}%</span>
                      </div>
                      <div className="download-info">
                        <span className="download-size">
                          {formatBytes(downloadProgress?.downloaded_bytes || 0)}
                          {downloadProgress?.total_bytes ? ` / ${formatBytes(downloadProgress.total_bytes)}` : ''}
                        </span>
                        <span className="download-speed">{formatSpeed(downloadProgress?.speed_bps || 0)}</span>
                      </div>
                      <Button
                        variant="ghost"
                        size="small"
                        onClick={cancelDownload}
                        className="download-cancel-btn"
                      >
                        Cancel
                      </Button>
                    </div>
                  ) : isDownloaded ? (
                    <Button
                      variant={isLoaded ? 'secondary' : 'primary'}
                      size="small"
                      onClick={() => isLoaded ? handleUnloadModel() : handleLoadModel(model.id)}
                      disabled={!!loading}
                    >
                      {isLoadingThis ? 'Loading...' : isLoaded ? 'Unload' : 'Load'}
                    </Button>
                  ) : (
                    <Button
                      variant="secondary"
                      size="small"
                      onClick={() => handleDownloadModel(model.id)}
                      disabled={isDownloading}
                    >
                      Download
                    </Button>
                  )}
                </div>
              </div>
            );
          })}
        </div>

        <div className="custom-model-section">
          <h3>Custom Model</h3>
          <p className="settings-description">
            Load any GGUF-format model file from your computer.
          </p>
          <Button
            variant="secondary"
            onClick={handleLoadCustomModel}
            disabled={!!loading || isDownloading}
          >
            {loading === 'custom' ? 'Loading...' : 'Select GGUF File...'}
          </Button>
        </div>
      </section>

      <section className="settings-section">
        <h2>Context Window</h2>
        <p className="settings-description">
          Configure the maximum context length for LLM generation. Larger values allow more content but use more memory.
        </p>
        <div className="context-window-config">
          <select
            className="context-window-select"
            value={contextWindowSize ?? ''}
            onChange={(e) => handleContextWindowChange(e.target.value)}
            disabled={!loadedModel}
          >
            {CONTEXT_WINDOW_OPTIONS.map(opt => (
              <option key={opt.value ?? 'default'} value={opt.value ?? ''}>
                {opt.label}
              </option>
            ))}
          </select>
          {!loadedModel && (
            <p className="setting-note">Load a model to configure context window.</p>
          )}
          <p className="setting-note">
            Higher values require more RAM. The "Model Default" option uses the model's training context (capped at 8K).
          </p>
        </div>
      </section>

      <section className="settings-section">
        <h2>Embedding Model</h2>
        <p className="settings-description">
          Embedding model for semantic search. Uses nomic-embed-text (768-dim, ~550MB).
        </p>

        <div className="embedding-model-config">
          {isDownloading && downloadProgress?.model_id === 'nomic-embed-text' ? (
            <div className="download-progress-container">
              <div className="download-progress">
                <div
                  className="download-bar"
                  style={{ width: `${downloadProgress?.percent || 0}%` }}
                />
                <span className="download-percent">{Math.round(downloadProgress?.percent || 0)}%</span>
              </div>
              <div className="download-info">
                <span className="download-size">
                  {formatBytes(downloadProgress?.downloaded_bytes || 0)}
                  {downloadProgress?.total_bytes ? ` / ${formatBytes(downloadProgress.total_bytes)}` : ''}
                </span>
                <span className="download-speed">{formatSpeed(downloadProgress?.speed_bps || 0)}</span>
              </div>
              <Button
                variant="ghost"
                size="small"
                onClick={cancelDownload}
                className="download-cancel-btn"
              >
                Cancel
              </Button>
            </div>
          ) : !embeddingDownloaded ? (
            <div className="embedding-status">
              <span className="status-badge not-downloaded">Not Downloaded</span>
              <Button
                variant="primary"
                size="small"
                onClick={handleDownloadEmbeddingModel}
                disabled={isDownloading}
              >
                Download Model
              </Button>
            </div>
          ) : !isEmbeddingLoaded ? (
            <div className="embedding-status">
              <span className="status-badge downloaded">Downloaded</span>
              <Button
                variant="primary"
                size="small"
                onClick={handleLoadEmbeddingModel}
                disabled={embeddingLoading}
              >
                {embeddingLoading ? 'Loading...' : 'Load Model'}
              </Button>
            </div>
          ) : (
            <div className="embedding-status">
              <span className="status-badge loaded">Loaded</span>
              <div className="embedding-info">
                <span className="model-name">{embeddingModelInfo?.name || 'nomic-embed-text'}</span>
                <span className="model-dim">{embeddingModelInfo?.embedding_dim || 768} dimensions</span>
              </div>
              <Button
                variant="secondary"
                size="small"
                onClick={handleUnloadEmbeddingModel}
              >
                Unload
              </Button>
            </div>
          )}

          {vectorEnabled && isEmbeddingLoaded && (
            <div className="generate-embeddings-row">
              <Button
                variant="ghost"
                size="small"
                onClick={handleGenerateEmbeddings}
                disabled={generatingEmbeddings}
              >
                {generatingEmbeddings ? 'Generating...' : 'Generate Embeddings for KB'}
              </Button>
              <p className="setting-note">
                Creates vector embeddings for all indexed documents.
              </p>
            </div>
          )}
        </div>
      </section>

      <section className="settings-section">
        <h2>Knowledge Base</h2>
        <p className="settings-description">
          Configure the folder containing your knowledge base documents.
        </p>

        <div className="kb-config">
          <div className="kb-folder-row">
            <div className="kb-folder-display">
              {kbFolder ? (
                <code>{kbFolder}</code>
              ) : (
                <span className="kb-placeholder">No folder selected</span>
              )}
            </div>
            <Button variant="secondary" onClick={handleSelectKbFolder}>
              {kbFolder ? 'Change' : 'Select Folder'}
            </Button>
          </div>

          {kbFolder && (
            <div className="kb-stats">
              <div className="stat-item">
                <span className="stat-label">Files indexed</span>
                <span className="stat-value">{indexStats?.total_files ?? '—'}</span>
              </div>
              <div className="stat-item">
                <span className="stat-label">Total chunks</span>
                <span className="stat-value">{indexStats?.total_chunks ?? '—'}</span>
              </div>
              <Button
                variant="ghost"
                size="small"
                onClick={handleRebuildIndex}
                disabled={loading === 'rebuild'}
              >
                {loading === 'rebuild' ? 'Rebuilding...' : 'Rebuild Index'}
              </Button>
            </div>
          )}
        </div>
      </section>

      <section className="settings-section">
        <h2>Advanced Search</h2>
        <p className="settings-description">
          Enable AI-powered semantic search for better knowledge base results.
        </p>
        <div className="vector-consent">
          <label className="toggle-label">
            <input
              type="checkbox"
              checked={vectorEnabled}
              onChange={handleVectorToggle}
            />
            <span className="toggle-text">Enable vector embeddings</span>
          </label>
          <p className="setting-note">
            Creates embeddings of your documents for semantic search.
            All processing happens locally on your machine.
          </p>
        </div>
      </section>

      <section className="settings-section">
        <h2>Template Variables</h2>
        <p className="settings-description">
          Define custom variables to use in response templates. Use as <code>{`{{variable_name}}`}</code> in your prompts.
        </p>

        <div className="variables-container">
          {customVariables.length === 0 ? (
            <p className="variables-empty">No custom variables defined yet.</p>
          ) : (
            <div className="variables-list">
              {customVariables.map((variable) => (
                <div key={variable.id} className="variable-item">
                  <div className="variable-info">
                    <code className="variable-name">{`{{${variable.name}}}`}</code>
                    <span className="variable-value">{variable.value}</span>
                  </div>
                  <div className="variable-actions">
                    <Button
                      variant="ghost"
                      size="small"
                      onClick={() => handleEditVariable(variable)}
                    >
                      Edit
                    </Button>
                    <Button
                      variant="ghost"
                      size="small"
                      onClick={() => handleDeleteVariable(variable.id)}
                    >
                      Delete
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          )}

          <Button
            variant="secondary"
            size="small"
            onClick={handleAddVariable}
          >
            + Add Variable
          </Button>
        </div>

        {showVariableForm && (
          <div className="variable-form-overlay" onClick={handleCancelVariableForm}>
            <div className="variable-form-modal" onClick={(e) => e.stopPropagation()}>
              <h3>{editingVariable ? 'Edit Variable' : 'Add Variable'}</h3>
              {variableFormError && (
                <div className="variable-form-error">{variableFormError}</div>
              )}
              <div className="form-field">
                <label htmlFor="var-name">Name</label>
                <input
                  id="var-name"
                  type="text"
                  placeholder="my_variable"
                  value={variableForm.name}
                  onChange={(e) => setVariableForm((f) => ({ ...f, name: e.target.value }))}
                  autoFocus
                />
                <p className="field-hint">Letters, numbers, and underscores only</p>
              </div>
              <div className="form-field">
                <label htmlFor="var-value">Value</label>
                <textarea
                  id="var-value"
                  placeholder="The value to substitute..."
                  value={variableForm.value}
                  onChange={(e) => setVariableForm((f) => ({ ...f, value: e.target.value }))}
                  rows={3}
                />
              </div>
              <div className="form-actions">
                <Button variant="ghost" onClick={handleCancelVariableForm}>
                  Cancel
                </Button>
                <Button
                  variant="primary"
                  onClick={handleSaveVariable}
                  disabled={!variableForm.name.trim() || !variableForm.value.trim()}
                >
                  {editingVariable ? 'Save' : 'Add'}
                </Button>
              </div>
            </div>
          </div>
        )}
      </section>

      <section className="settings-section">
        <h2>Jira Integration</h2>
        <p className="settings-description">
          Connect to Jira Cloud to import tickets directly into your drafts.
        </p>

        {jiraConfigured ? (
          <div className="jira-connected">
            <div className="jira-status">
              <span className="status-icon">&#10003;</span>
              <span>Connected to {jiraConfig?.base_url || 'Jira'}</span>
            </div>
            <p className="jira-email">Account: {jiraConfig?.email}</p>
            <Button
              variant="secondary"
              size="small"
              onClick={handleJiraDisconnect}
              disabled={jiraLoading}
            >
              Disconnect
            </Button>
          </div>
        ) : (
          <form className="jira-form" onSubmit={handleJiraConnect}>
            <div className="form-field">
              <label htmlFor="jira-url">Jira URL</label>
              <input
                id="jira-url"
                type="url"
                placeholder="https://your-company.atlassian.net"
                value={jiraForm.baseUrl}
                onChange={e => setJiraForm(f => ({ ...f, baseUrl: e.target.value }))}
                required
              />
            </div>
            <div className="form-field">
              <label htmlFor="jira-email">Email</label>
              <input
                id="jira-email"
                type="email"
                placeholder="your.email@company.com"
                value={jiraForm.email}
                onChange={e => setJiraForm(f => ({ ...f, email: e.target.value }))}
                required
              />
            </div>
            <div className="form-field">
              <label htmlFor="jira-token">API Token</label>
              <input
                id="jira-token"
                type="password"
                placeholder="Your Jira API token"
                value={jiraForm.apiToken}
                onChange={e => setJiraForm(f => ({ ...f, apiToken: e.target.value }))}
                required
              />
              <p className="field-hint">
                Generate at <a href="https://id.atlassian.com/manage/api-tokens" target="_blank" rel="noopener noreferrer">id.atlassian.com/manage/api-tokens</a>
              </p>
            </div>
            <Button
              type="submit"
              variant="primary"
              disabled={jiraLoading || !jiraForm.baseUrl || !jiraForm.email || !jiraForm.apiToken}
            >
              {jiraLoading ? 'Connecting...' : 'Connect'}
            </Button>
          </form>
        )}
      </section>

      <section className="settings-section">
        <h2>Deployment &amp; Integrations</h2>
        <p className="settings-description">
          Deployment health, preflight validation, and integration toggles for ServiceNow/Slack/Teams.
        </p>
        <div className="settings-row">
          <Button
            variant="secondary"
            size="small"
            onClick={handleRunDeploymentPreflight}
            disabled={deployPreflightRunning}
          >
            {deployPreflightRunning ? 'Running preflight...' : 'Run Deployment Preflight'}
          </Button>
        </div>
        {deploymentHealth && (
          <div className="startup-metrics">
            <p className="text-sm text-secondary">
              Signed artifacts: {deploymentHealth.signed_artifacts}/{deploymentHealth.total_artifacts}
            </p>
            {deploymentHealth.last_run && (
              <p className="text-sm text-secondary">
                Last run: {deploymentHealth.last_run.status} ({deploymentHealth.last_run.target_channel})
              </p>
            )}
          </div>
        )}
        {deployPreflightChecks.length > 0 && (
          <ul className="audit-list">
            {deployPreflightChecks.map((check, idx) => (
              <li key={`${check}-${idx}`} className="audit-row">{check}</li>
            ))}
          </ul>
        )}
        <div className="settings-row">
          {['servicenow', 'slack', 'teams'].map(type => {
            const current = integrations.find(i => i.integration_type === type);
            const enabled = current?.enabled ?? false;
            return (
              <label key={type} className="toggle-option">
                <input
                  type="checkbox"
                  checked={enabled}
                  onChange={(e) => void handleToggleIntegration(type, e.target.checked)}
                />
                <span>{type.charAt(0).toUpperCase() + type.slice(1)}</span>
              </label>
            );
          })}
        </div>
      </section>

      <section className="settings-section">
        <h2>Data Backup</h2>
        <p className="settings-description">
          Export or import your drafts, templates, variables, and settings.
        </p>
        <div className="backup-actions">
          <div className="backup-row">
            <div className="backup-info">
              <strong>Export</strong>
              <span>Save all your data to a ZIP file for backup.</span>
            </div>
            <Button
              variant="secondary"
              size="small"
              onClick={handleExportBackup}
              disabled={backupLoading === 'export'}
            >
              {backupLoading === 'export' ? 'Exporting...' : 'Export Data'}
            </Button>
          </div>
          <div className="backup-row">
            <div className="backup-info">
              <strong>Import</strong>
              <span>Restore data from a backup ZIP file.</span>
            </div>
            <Button
              variant="secondary"
              size="small"
              onClick={handleImportBackup}
              disabled={backupLoading === 'import'}
            >
              {backupLoading === 'import' ? 'Importing...' : 'Import Data'}
            </Button>
          </div>
        </div>
      </section>

      <section className="settings-section">
        <h2>Audit Logs</h2>
        <p className="settings-description">
          Security and system events recorded locally. Export for review or compliance.
        </p>
        <div className="audit-actions">
          <Button
            variant="secondary"
            size="small"
            onClick={loadAuditEntries}
            disabled={auditLoading}
          >
            {auditLoading ? 'Refreshing...' : 'Refresh'}
          </Button>
          <Button
            variant="secondary"
            size="small"
            onClick={handleExportAuditLog}
            disabled={auditExporting}
          >
            {auditExporting ? 'Exporting...' : 'Export JSON'}
          </Button>
        </div>
        <div className="audit-list">
          {auditEntries.length === 0 ? (
            <p className="audit-empty">No audit entries yet.</p>
          ) : (
            auditEntries
              .slice()
              .reverse()
              .map((entry, index) => (
                <div className="audit-row" key={`${entry.timestamp}-${index}`}>
                  <span className={`audit-severity ${entry.severity}`}>{entry.severity}</span>
                  <span className="audit-event">{formatAuditEvent(entry.event)}</span>
                  <span className="audit-message">{entry.message}</span>
                  <span className="audit-time">{new Date(entry.timestamp).toLocaleString()}</span>
                </div>
              ))
          )}
        </div>
      </section>

      <section className="settings-section">
        <h2>Security &amp; Session</h2>
        <p className="settings-description">
          Your session auto-unlocks for 24 hours. Lock the app to require re-authentication.
        </p>
        <div className="settings-row">
          <Button
            variant="secondary"
            size="small"
            disabled={lockingApp}
            onClick={async () => {
              setLockingApp(true);
              try {
                await invoke('lock_app');
                localStorage.removeItem('assistsupport_session_token');
                showSuccess('App locked. You will need to re-authenticate on next launch.');
              } catch (e) {
                showError(`Failed to lock app: ${e}`);
              } finally {
                setLockingApp(false);
              }
            }}
          >
            {lockingApp ? 'Locking...' : 'Lock App (Require Password)'}
          </Button>
        </div>
        {startupMetrics && (
          <div className="startup-metrics">
            <p className="text-sm text-secondary">
              Last startup: {startupMetrics.init_app_ms}ms
            </p>
          </div>
        )}
      </section>

      <section className="settings-section">
        <h2>About</h2>
        <p className="settings-description">
          AssistSupport - Local AI-powered support ticket assistant
        </p>
        <div className="about-info">
          <p>Version 1.0.0</p>
          <p>All processing happens locally on your machine.</p>
        </div>
      </section>
    </div>
  );
}
