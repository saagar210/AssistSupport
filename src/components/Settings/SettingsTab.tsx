import { useState, useEffect, useCallback, useMemo } from 'react';
import { Button } from '../shared/Button';
import { useLlm } from '../../hooks/useLlm';
import { useKb } from '../../hooks/useKb';
import { useDownload } from '../../hooks/useDownload';
import { useJira } from '../../hooks/useJira';
import { useEmbedding } from '../../hooks/useEmbedding';
import { useCustomVariables } from '../../hooks/useCustomVariables';
import { useFeatureOps } from '../../hooks/useFeatureOps';
import {
  getResponseQualityThresholds,
  resetResponseQualityThresholds,
  ResponseQualityThresholds,
  saveResponseQualityThresholds,
} from '../../features/analytics/qualityThresholds';
import { useTheme } from '../../contexts/ThemeContext';
import { useToastContext } from '../../contexts/ToastContext';
import { resolveRevampFlags } from '../../features/revamp/flags';
import appPackage from '../../../package.json';
import { formatAppVersion } from './versionLabel';
import {
  AboutSection,
  AppearanceSection,
  DeploymentIntegrationsSection,
  MaintenanceDiagnosticsSection,
  MemoryKernelSection,
  PolicyGatesSection,
  SettingsHeroSection,
} from './internal/SettingsSections';
import { SettingsModelSection } from './internal/SettingsModelSection';
import { useSettingsOperationalState } from './internal/useSettingsOperationalState';
import { useSettingsAuditLogs } from './internal/useSettingsAuditLogs';
import { useSettingsWorkspaceState } from './internal/useSettingsWorkspaceState';
import type {
  CustomVariable,
  ModelInfo,
} from '../../types';
import './SettingsTab.css';

const RECOMMENDED_MODELS: ModelInfo[] = [
  {
    id: 'llama-3.1-8b-instruct',
    name: 'Llama 3.1 8B Instruct',
    size: '4.9 GB',
    description: 'Recommended: higher quality and more reliable grounding',
  },
];

// Still supported, but intentionally hidden behind progressive disclosure to keep
// operators focused on a single default model path.
const OTHER_SUPPORTED_MODELS: ModelInfo[] = [
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

const APP_VERSION = appPackage.version;

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

function validateQualityThresholds(
  thresholds: ResponseQualityThresholds,
): string | null {
  if (thresholds.editRatioWatch >= thresholds.editRatioAction) {
    return 'Edit ratio watch threshold must be lower than action threshold.';
  }
  if (thresholds.timeToDraftWatchMs >= thresholds.timeToDraftActionMs) {
    return 'Time-to-draft watch threshold must be lower than action threshold.';
  }
  if (thresholds.copyPerSaveWatch <= thresholds.copyPerSaveAction) {
    return 'Copy-per-save watch threshold must be higher than action threshold.';
  }
  if (thresholds.editedSaveRateWatch >= thresholds.editedSaveRateAction) {
    return 'Edited save rate watch threshold must be lower than action threshold.';
  }
  return null;
}

function SettingsTabOrchestrator() {
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

  const [qualityThresholds, setQualityThresholds] = useState<ResponseQualityThresholds>(() =>
    getResponseQualityThresholds(),
  );
  const [qualityThresholdError, setQualityThresholdError] = useState<string | null>(null);
  const revampFlags = useMemo(() => resolveRevampFlags(), []);

  // Custom variables state
  const [editingVariable, setEditingVariable] = useState<CustomVariable | null>(null);
  const [variableForm, setVariableForm] = useState({ name: '', value: '' });
  const [showVariableForm, setShowVariableForm] = useState(false);
  const [variableFormError, setVariableFormError] = useState<string | null>(null);
  const {
    deploymentHealth,
    deployPreflightChecks,
    deployPreflightRunning,
    integrations,
    memoryKernelPreflight,
    memoryKernelLoading,
    databaseStats,
    databaseStatsLoading,
    databaseMaintenanceRunning,
    databaseMaintenanceResult,
    refreshMemoryKernelStatus,
    refreshDatabaseStats,
    handleRunDatabaseMaintenance,
    refreshDeploymentAndIntegrations,
    handleRunDeploymentPreflight,
    handleToggleIntegration,
  } = useSettingsOperationalState({
    getDeploymentHealthSummary,
    runDeploymentPreflight,
    listIntegrations,
    configureIntegration,
    showSuccess,
    showError,
  });
  const {
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
  } = useSettingsWorkspaceState({
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
  });
  const {
    auditLoading,
    auditExporting,
    auditSeverityFilter,
    auditSearchQuery,
    auditPage,
    auditTotalPages,
    filteredAuditEntries,
    pagedAuditEntries,
    setAuditPage,
    setSeverityFilter,
    setSearchQuery,
    loadAuditEntries,
    handleExportAuditLog,
    formatAuditEvent,
  } = useSettingsAuditLogs({
    showSuccess,
    showError,
  });

  useEffect(() => {
    Promise.resolve(initializeWorkspaceState()).catch(err => console.error('Settings init failed:', err));
    setQualityThresholds(getResponseQualityThresholds());
    Promise.resolve(loadVariables()).catch(err => console.error('Variables load failed:', err));
    Promise.resolve(loadAuditEntries()).catch(err => console.error('Audit load failed:', err));
  }, [initializeWorkspaceState, loadVariables, loadAuditEntries]);

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

  const updateQualityThreshold = useCallback(
    <K extends keyof ResponseQualityThresholds>(
      key: K,
      value: number,
    ) => {
      setQualityThresholds(prev => ({ ...prev, [key]: value }));
      setQualityThresholdError(null);
    },
    [],
  );

  const handleSaveQualityThresholds = useCallback(() => {
    const validationError = validateQualityThresholds(qualityThresholds);
    if (validationError) {
      setQualityThresholdError(validationError);
      return;
    }
    const saved = saveResponseQualityThresholds(qualityThresholds);
    setQualityThresholds(saved);
    setQualityThresholdError(null);
    showSuccess('Response quality coaching thresholds updated');
  }, [qualityThresholds, showSuccess]);

  const handleResetQualityThresholds = useCallback(() => {
    const defaults = resetResponseQualityThresholds();
    setQualityThresholds(defaults);
    setQualityThresholdError(null);
    showSuccess('Response quality coaching thresholds reset to defaults');
  }, [showSuccess]);

  return (
    <div className="settings-tab">
      {error && <div className="settings-error">{error}</div>}

      <SettingsHeroSection
        loadedModel={loadedModel}
        kbFolder={kbFolder}
        isEmbeddingLoaded={isEmbeddingLoaded}
        embeddingDownloaded={embeddingDownloaded}
        memoryKernelPreflight={memoryKernelPreflight}
      />

      <PolicyGatesSection revampFlags={revampFlags} />

      <MemoryKernelSection
        memoryKernelPreflight={memoryKernelPreflight}
        memoryKernelLoading={memoryKernelLoading}
        onRefresh={refreshMemoryKernelStatus}
      />

      <AppearanceSection theme={theme} setTheme={setTheme} />

      <SettingsModelSection
        loadedModel={loadedModel}
        downloadedModels={downloadedModels}
        recommendedModels={RECOMMENDED_MODELS}
        otherSupportedModels={OTHER_SUPPORTED_MODELS}
        showOtherModels={showOtherModels}
        loading={loading}
        isDownloading={isDownloading}
        downloadProgress={downloadProgress ?? null}
        isEmbeddingLoaded={isEmbeddingLoaded}
        kbFolder={kbFolder}
        memoryKernelPreflight={memoryKernelPreflight}
        memoryKernelLoading={memoryKernelLoading}
        onUnloadModel={() => {
          void handleUnloadModel();
        }}
        onLoadModel={(modelId) => {
          void handleLoadModel(modelId);
        }}
        onDownloadModel={(modelId) => {
          void handleDownloadModel(modelId);
        }}
        onCancelDownload={() => {
          void cancelDownload();
        }}
        onToggleOtherModels={() => setShowOtherModels(v => !v)}
        onLoadCustomModel={() => {
          void handleLoadCustomModel();
        }}
        onRefreshMemoryKernelStatus={() => {
          void refreshMemoryKernelStatus();
        }}
      />

      <section className="settings-section">
        <h2>Context Window</h2>
        <p className="settings-description">
          Configure the maximum context length for LLM generation. Larger values allow more content but use more memory.
        </p>
        <div className="context-window-config">
          <select
            className="context-window-select"
            aria-label="Context window size"
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

      <DeploymentIntegrationsSection
        deploymentHealth={deploymentHealth}
        deployPreflightChecks={deployPreflightChecks}
        deployPreflightRunning={deployPreflightRunning}
        integrations={integrations}
        onRunPreflight={() => {
          void handleRunDeploymentPreflight();
        }}
        onToggleIntegration={(integrationType, enabled) => {
          void handleToggleIntegration(integrationType, enabled);
        }}
      />

      <MaintenanceDiagnosticsSection
        databaseStats={databaseStats}
        databaseStatsLoading={databaseStatsLoading}
        databaseMaintenanceRunning={databaseMaintenanceRunning}
        databaseMaintenanceResult={databaseMaintenanceResult}
        onRefreshDatabaseStats={() => {
          void refreshDatabaseStats();
        }}
        onRunDatabaseMaintenance={() => {
          void handleRunDatabaseMaintenance();
        }}
      />

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
        <h2>Response Quality Coaching</h2>
        <p className="settings-description">
          Tune coaching severity bands used in Analytics. These thresholds are local to this workspace and can be calibrated for your support team.
        </p>
        <div className="quality-threshold-grid">
          <div className="quality-threshold-card">
            <h3>Edit Ratio (%)</h3>
            <div className="quality-threshold-fields">
              <label>
                Watch
                <input
                  type="number"
                  min={0}
                  max={100}
                  step={1}
                  value={(qualityThresholds.editRatioWatch * 100).toFixed(0)}
                  onChange={(e) =>
                    updateQualityThreshold(
                      'editRatioWatch',
                      Number(e.target.value || 0) / 100,
                    )
                  }
                />
              </label>
              <label>
                Action
                <input
                  type="number"
                  min={0}
                  max={100}
                  step={1}
                  value={(qualityThresholds.editRatioAction * 100).toFixed(0)}
                  onChange={(e) =>
                    updateQualityThreshold(
                      'editRatioAction',
                      Number(e.target.value || 0) / 100,
                    )
                  }
                />
              </label>
            </div>
          </div>
          <div className="quality-threshold-card">
            <h3>Time to Draft (seconds)</h3>
            <div className="quality-threshold-fields">
              <label>
                Watch
                <input
                  type="number"
                  min={1}
                  step={5}
                  value={Math.round(qualityThresholds.timeToDraftWatchMs / 1000)}
                  onChange={(e) =>
                    updateQualityThreshold(
                      'timeToDraftWatchMs',
                      Math.max(1, Number(e.target.value || 1)) * 1000,
                    )
                  }
                />
              </label>
              <label>
                Action
                <input
                  type="number"
                  min={1}
                  step={5}
                  value={Math.round(qualityThresholds.timeToDraftActionMs / 1000)}
                  onChange={(e) =>
                    updateQualityThreshold(
                      'timeToDraftActionMs',
                      Math.max(1, Number(e.target.value || 1)) * 1000,
                    )
                  }
                />
              </label>
            </div>
          </div>
          <div className="quality-threshold-card">
            <h3>Copy per Save (%)</h3>
            <div className="quality-threshold-fields">
              <label>
                Watch
                <input
                  type="number"
                  min={0}
                  max={100}
                  step={1}
                  value={(qualityThresholds.copyPerSaveWatch * 100).toFixed(0)}
                  onChange={(e) =>
                    updateQualityThreshold(
                      'copyPerSaveWatch',
                      Number(e.target.value || 0) / 100,
                    )
                  }
                />
              </label>
              <label>
                Action
                <input
                  type="number"
                  min={0}
                  max={100}
                  step={1}
                  value={(qualityThresholds.copyPerSaveAction * 100).toFixed(0)}
                  onChange={(e) =>
                    updateQualityThreshold(
                      'copyPerSaveAction',
                      Number(e.target.value || 0) / 100,
                    )
                  }
                />
              </label>
            </div>
          </div>
          <div className="quality-threshold-card">
            <h3>Edited Save Rate (%)</h3>
            <div className="quality-threshold-fields">
              <label>
                Watch
                <input
                  type="number"
                  min={0}
                  max={100}
                  step={1}
                  value={(qualityThresholds.editedSaveRateWatch * 100).toFixed(0)}
                  onChange={(e) =>
                    updateQualityThreshold(
                      'editedSaveRateWatch',
                      Number(e.target.value || 0) / 100,
                    )
                  }
                />
              </label>
              <label>
                Action
                <input
                  type="number"
                  min={0}
                  max={100}
                  step={1}
                  value={(qualityThresholds.editedSaveRateAction * 100).toFixed(0)}
                  onChange={(e) =>
                    updateQualityThreshold(
                      'editedSaveRateAction',
                      Number(e.target.value || 0) / 100,
                    )
                  }
                />
              </label>
            </div>
          </div>
        </div>
        {qualityThresholdError && (
          <div className="settings-error">{qualityThresholdError}</div>
        )}
        <div className="quality-threshold-actions">
          <Button variant="secondary" size="small" onClick={handleSaveQualityThresholds}>
            Save Thresholds
          </Button>
          <Button variant="ghost" size="small" onClick={handleResetQualityThresholds}>
            Reset Defaults
          </Button>
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
        <div className="audit-filters">
          <label className="audit-filter-label">
            Severity
            <select
              aria-label="Audit severity filter"
              value={auditSeverityFilter}
              onChange={(e) => {
                setSeverityFilter(e.target.value as typeof auditSeverityFilter);
              }}
            >
              <option value="all">All</option>
              <option value="info">Info</option>
              <option value="warning">Warning</option>
              <option value="error">Error</option>
              <option value="critical">Critical</option>
            </select>
          </label>
          <input
            className="input"
            value={auditSearchQuery}
            onChange={(e) => {
              setSearchQuery(e.target.value);
            }}
            placeholder="Search event/message"
          />
        </div>
        <div className="audit-list">
          {pagedAuditEntries.length === 0 ? (
            <p className="audit-empty">No audit entries yet.</p>
          ) : (
            pagedAuditEntries.map((entry, index) => (
              <div className="audit-row" key={`${entry.timestamp}-${index}`}>
                <span className={`audit-severity ${entry.severity}`}>{entry.severity}</span>
                <span className="audit-event">{formatAuditEvent(entry.event)}</span>
                <span className="audit-message">{entry.message}</span>
                <span className="audit-time">{new Date(entry.timestamp).toLocaleString()}</span>
              </div>
            ))
          )}
        </div>
        <div className="audit-pagination">
          <span className="text-sm text-secondary">
            {filteredAuditEntries.length} entries • Page {auditPage} of {auditTotalPages}
          </span>
          <div className="audit-pagination-actions">
            <Button
              variant="secondary"
              size="small"
              onClick={() => setAuditPage((p) => Math.max(1, p - 1))}
              disabled={auditPage <= 1}
            >
              Previous
            </Button>
            <Button
              variant="secondary"
              size="small"
              onClick={() => setAuditPage((p) => Math.min(auditTotalPages, p + 1))}
              disabled={auditPage >= auditTotalPages}
            >
              Next
            </Button>
          </div>
        </div>
      </section>

      <AboutSection appVersionLabel={formatAppVersion(APP_VERSION)} />
    </div>
  );
}

export function SettingsTab() {
  return <SettingsTabOrchestrator />;
}
