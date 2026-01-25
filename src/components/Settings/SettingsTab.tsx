import { useState, useEffect } from 'react';
import { Button } from '../shared/Button';
import { useLlm } from '../../hooks/useLlm';
import { useKb } from '../../hooks/useKb';
import { useDownload } from '../../hooks/useDownload';
import { useJira } from '../../hooks/useJira';
import { useTheme } from '../../contexts/ThemeContext';
import type { ModelInfo } from '../../types';
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

const CONTEXT_WINDOW_OPTIONS = [
  { value: null, label: 'Model Default' },
  { value: 2048, label: '2K (2,048 tokens)' },
  { value: 4096, label: '4K (4,096 tokens)' },
  { value: 8192, label: '8K (8,192 tokens)' },
  { value: 16384, label: '16K (16,384 tokens)' },
  { value: 32768, label: '32K (32,768 tokens)' },
];

export function SettingsTab() {
  const { loadModel, unloadModel, getLoadedModel, listModels, getContextWindow, setContextWindow } = useLlm();
  const { setKbFolder, getKbFolder, rebuildIndex, getIndexStats, getVectorConsent, setVectorConsent } = useKb();
  const { downloadModel, downloadProgress, isDownloading } = useDownload();
  const { checkConfiguration: checkJiraConfig, configure: configureJira, disconnect: disconnectJira, config: jiraConfig, loading: jiraLoading } = useJira();
  const { theme, setTheme } = useTheme();

  const [loadedModel, setLoadedModel] = useState<string | null>(null);
  const [downloadedModels, setDownloadedModels] = useState<string[]>([]);
  const [kbFolder, setKbFolderState] = useState<string | null>(null);
  const [indexStats, setIndexStats] = useState<{ total_chunks: number; total_files: number } | null>(null);
  const [vectorEnabled, setVectorEnabled] = useState(false);
  const [jiraConfigured, setJiraConfigured] = useState(false);
  const [jiraForm, setJiraForm] = useState({ baseUrl: '', email: '', apiToken: '' });
  const [contextWindowSize, setContextWindowSize] = useState<number | null>(null);
  const [loading, setLoading] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadInitialState();
  }, []);

  async function loadInitialState() {
    try {
      const [loaded, downloaded, folder, stats, consent, jiraConfigResult, ctxWindow] = await Promise.all([
        getLoadedModel(),
        listModels(),
        getKbFolder(),
        getIndexStats().catch(() => null),
        getVectorConsent().catch(() => null),
        checkJiraConfig().catch(() => false),
        getContextWindow().catch(() => null),
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
    } catch (err) {
      setError(`Failed to update context window: ${err}`);
    }
  }

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
                    <div className="download-progress">
                      <div
                        className="download-bar"
                        style={{ width: `${downloadProgress?.percent || 0}%` }}
                      />
                      <span>{Math.round(downloadProgress?.percent || 0)}%</span>
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
        <h2>About</h2>
        <p className="settings-description">
          AssistSupport - Local AI-powered support ticket assistant
        </p>
        <div className="about-info">
          <p>Version 0.1.0</p>
          <p>All processing happens locally on your machine.</p>
        </div>
      </section>
    </div>
  );
}
