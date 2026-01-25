import { useState, useEffect } from 'react';
import { useIngest } from '../../hooks/useIngest';
import { useToastContext } from '../../contexts/ToastContext';
import { Button } from '../shared/Button';
import { UrlIngest } from './UrlIngest';
import { YouTubeIngest } from './YouTubeIngest';
import { GitHubIngest } from './GitHubIngest';
import { BatchIngest } from './BatchIngest';
import './IngestTab.css';

type IngestMode = 'url' | 'youtube' | 'github' | 'batch';

export function IngestTab() {
  const {
    namespaces,
    ingesting,
    ytdlpAvailable,
    error,
    loadNamespaces,
    createNamespace,
    checkYtdlp,
  } = useIngest();

  const { success: showSuccess, error: showError } = useToastContext();

  const [mode, setMode] = useState<IngestMode>('url');
  const [selectedNamespace, setSelectedNamespace] = useState<string>('default');
  const [showNewNamespace, setShowNewNamespace] = useState(false);
  const [newNamespaceName, setNewNamespaceName] = useState('');

  useEffect(() => {
    loadNamespaces();
    checkYtdlp();
  }, [loadNamespaces, checkYtdlp]);

  useEffect(() => {
    if (error) {
      showError(error);
    }
  }, [error, showError]);

  const handleCreateNamespace = async () => {
    if (!newNamespaceName.trim()) return;

    try {
      await createNamespace(newNamespaceName.trim());
      setSelectedNamespace(newNamespaceName.trim().toLowerCase().replace(' ', '-'));
      setNewNamespaceName('');
      setShowNewNamespace(false);
      showSuccess(`Namespace "${newNamespaceName}" created`);
    } catch (e) {
      showError(`Failed to create namespace: ${e}`);
    }
  };

  const handleIngestSuccess = (message: string) => {
    showSuccess(message);
  };

  const handleIngestError = (message: string) => {
    showError(message);
  };

  return (
    <div className="ingest-tab">
      <div className="ingest-header">
        <h2>Content Ingestion</h2>
        <p className="ingest-description">
          Add content to your knowledge base from various sources. All data is stored locally on your device.
        </p>
      </div>

      <div className="ingest-config">
        <div className="namespace-selector">
          <label htmlFor="namespace">Namespace</label>
          <div className="namespace-row">
            <select
              id="namespace"
              value={selectedNamespace}
              onChange={(e) => setSelectedNamespace(e.target.value)}
              disabled={ingesting}
            >
              {namespaces.map((ns) => (
                <option key={ns.id} value={ns.id}>
                  {ns.name}
                </option>
              ))}
            </select>
            <Button
              variant="secondary"
              size="small"
              onClick={() => setShowNewNamespace(!showNewNamespace)}
              disabled={ingesting}
            >
              + New
            </Button>
          </div>

          {showNewNamespace && (
            <div className="new-namespace-form">
              <input
                type="text"
                placeholder="Namespace name"
                value={newNamespaceName}
                onChange={(e) => setNewNamespaceName(e.target.value)}
                onKeyDown={(e) => e.key === 'Enter' && handleCreateNamespace()}
              />
              <Button
                variant="primary"
                size="small"
                onClick={handleCreateNamespace}
                disabled={!newNamespaceName.trim()}
              >
                Create
              </Button>
            </div>
          )}
        </div>

        <div className="mode-selector">
          <label>Source Type</label>
          <div className="mode-buttons">
            <button
              className={`mode-btn ${mode === 'url' ? 'active' : ''}`}
              onClick={() => setMode('url')}
              disabled={ingesting}
            >
              Web Page
            </button>
            <button
              className={`mode-btn ${mode === 'youtube' ? 'active' : ''}`}
              onClick={() => setMode('youtube')}
              disabled={ingesting}
            >
              YouTube
              {ytdlpAvailable === false && <span className="mode-warning">!</span>}
            </button>
            <button
              className={`mode-btn ${mode === 'github' ? 'active' : ''}`}
              onClick={() => setMode('github')}
              disabled={ingesting}
            >
              GitHub
            </button>
            <button
              className={`mode-btn ${mode === 'batch' ? 'active' : ''}`}
              onClick={() => setMode('batch')}
              disabled={ingesting}
            >
              Batch
            </button>
          </div>
        </div>
      </div>

      <div className="ingest-content">
        {mode === 'url' && (
          <UrlIngest
            namespaceId={selectedNamespace}
            onSuccess={handleIngestSuccess}
            onError={handleIngestError}
          />
        )}
        {mode === 'youtube' && (
          <YouTubeIngest
            namespaceId={selectedNamespace}
            ytdlpAvailable={ytdlpAvailable}
            onSuccess={handleIngestSuccess}
            onError={handleIngestError}
          />
        )}
        {mode === 'github' && (
          <GitHubIngest
            namespaceId={selectedNamespace}
            onSuccess={handleIngestSuccess}
            onError={handleIngestError}
          />
        )}
        {mode === 'batch' && (
          <BatchIngest
            onSuccess={handleIngestSuccess}
            onError={handleIngestError}
          />
        )}
      </div>

      <div className="ingest-privacy-notice">
        <strong>Privacy Notice:</strong> All ingested content is stored locally on your device.
        Be mindful of ingesting sensitive or confidential information.
      </div>
    </div>
  );
}
