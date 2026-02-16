import { useState, useEffect } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import { useIngest } from '../../hooks/useIngest';
import { Button } from '../shared/Button';

interface GitHubIngestProps {
  namespaceId: string;
  onSuccess: (message: string) => void;
  onError: (message: string) => void;
}

export function GitHubIngest({ namespaceId, onSuccess, onError }: GitHubIngestProps) {
  const {
    ingestGithub,
    ingestGithubRemote,
    hasGithubToken,
    setGithubToken,
    clearGithubToken,
    ingesting,
  } = useIngest();
  const [mode, setMode] = useState<'local' | 'remote'>('local');
  const [repoPath, setRepoPath] = useState('');
  const [repoUrl, setRepoUrl] = useState('');
  const [host, setHost] = useState<string | null>(null);
  const [token, setToken] = useState('');
  const [tokenStored, setTokenStored] = useState(false);
  const [tokenLoading, setTokenLoading] = useState(false);

  useEffect(() => {
    const value = repoUrl.trim();
    if (!value) {
      setHost(null);
      setTokenStored(false);
      return;
    }
    try {
      const parsed = new URL(value);
      setHost(parsed.host);
    } catch {
      setHost(null);
      setTokenStored(false);
    }
  }, [repoUrl]);

  useEffect(() => {
    let active = true;
    if (!host) {
      setTokenStored(false);
      return;
    }
    setTokenLoading(true);
    hasGithubToken(host)
      .then((stored) => {
        if (active) {
          setTokenStored(stored);
        }
      })
      .catch(() => {
        if (active) {
          setTokenStored(false);
        }
      })
      .finally(() => {
        if (active) {
          setTokenLoading(false);
        }
      });
    return () => {
      active = false;
    };
  }, [host, hasGithubToken]);

  const handleBrowse = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Select Git Repository',
      });
      if (selected && typeof selected === 'string') {
        setRepoPath(selected);
      }
    } catch (e) {
      onError(`Failed to open folder picker: ${e}`);
    }
  };

  const handleIngest = async () => {
    if (!repoPath.trim()) return;

    try {
      const results = await ingestGithub(repoPath.trim(), namespaceId);
      const totalChunks = results.reduce((sum, r) => sum + r.chunk_count, 0);
      onSuccess(`Ingested ${results.length} files (${totalChunks} chunks)`);
      setRepoPath('');
    } catch (e) {
      onError(`Failed to ingest repository: ${e}`);
    }
  };

  const handleSaveToken = async () => {
    if (!host || !token.trim()) return;
    setTokenLoading(true);
    try {
      await setGithubToken(host, token.trim());
      setTokenStored(true);
      setToken('');
      onSuccess(`Token saved for ${host}`);
    } catch (e) {
      onError(`Failed to save token: ${e}`);
    } finally {
      setTokenLoading(false);
    }
  };

  const handleClearToken = async () => {
    if (!host) return;
    setTokenLoading(true);
    try {
      await clearGithubToken(host);
      setTokenStored(false);
      onSuccess(`Token cleared for ${host}`);
    } catch (e) {
      onError(`Failed to clear token: ${e}`);
    } finally {
      setTokenLoading(false);
    }
  };

  const handleIngestRemote = async () => {
    if (!repoUrl.trim()) return;

    try {
      if (token.trim() && host) {
        await setGithubToken(host, token.trim());
        setTokenStored(true);
        setToken('');
      }
      const results = await ingestGithubRemote(repoUrl.trim(), namespaceId);
      const totalChunks = results.reduce((sum, r) => sum + r.chunk_count, 0);
      onSuccess(`Ingested ${results.length} files (${totalChunks} chunks)`);
      setRepoUrl('');
    } catch (e) {
      onError(`Failed to ingest repository: ${e}`);
    }
  };

  return (
    <div className="ingest-form">
      <div className="ingest-form-header">
        <h3>GitHub Repository</h3>
        <p>Ingest documentation and code from local or remote GitHub repositories. README files, docs folder, and source code will be processed.</p>
      </div>

      <div className="ingest-form-field">
        <label>Repository Type</label>
        <div className="mode-buttons">
          <button
            className={`mode-btn ${mode === 'local' ? 'active' : ''}`}
            onClick={() => setMode('local')}
            disabled={ingesting}
          >
            Local
          </button>
          <button
            className={`mode-btn ${mode === 'remote' ? 'active' : ''}`}
            onClick={() => setMode('remote')}
            disabled={ingesting}
          >
            Remote (HTTPS)
          </button>
        </div>
      </div>

      {mode === 'local' && (
        <div className="ingest-form-field">
          <label htmlFor="repo-path">Repository Path</label>
          <div className="path-input-row">
            <input
              id="repo-path"
              type="text"
              placeholder="/path/to/repository"
              value={repoPath}
              onChange={(e) => setRepoPath(e.target.value)}
              disabled={ingesting}
            />
            <Button
              variant="secondary"
              size="small"
              onClick={handleBrowse}
              disabled={ingesting}
            >
              Browse...
            </Button>
          </div>
        </div>
      )}

      {mode === 'remote' && (
        <>
          <div className="ingest-form-field">
            <label htmlFor="repo-url">Repository HTTPS URL</label>
            <input
              id="repo-url"
              type="text"
              placeholder="https://github.com/owner/repo"
              value={repoUrl}
              onChange={(e) => setRepoUrl(e.target.value)}
              disabled={ingesting}
            />
          </div>

          <div className="ingest-form-field">
            <label htmlFor="repo-token">Access Token (optional)</label>
            <input
              id="repo-token"
              type="password"
              placeholder="Personal access token for private repos"
              value={token}
              onChange={(e) => setToken(e.target.value)}
              disabled={ingesting || tokenLoading}
            />
            <div className="token-actions">
              <Button
                variant="secondary"
                size="small"
                onClick={handleSaveToken}
                disabled={!host || !token.trim() || ingesting || tokenLoading}
              >
                Save Token
              </Button>
              {tokenStored && (
                <Button
                  variant="secondary"
                  size="small"
                  onClick={handleClearToken}
                  disabled={!host || ingesting || tokenLoading}
                >
                  Clear Token
                </Button>
              )}
              {host && (
                <span className="token-status">
                  {tokenStored ? `Token stored for ${host}` : `No token stored for ${host}`}
                </span>
              )}
            </div>
          </div>
        </>
      )}

      <div className="ingest-form-info">
        {mode === 'local' && (
          <ul>
            <li>Must be a valid Git repository (contains .git folder)</li>
            <li>Indexes: .md, .mdx, .rst, .txt, .adoc files</li>
            <li>Also indexes common code files (.py, .js, .ts, .rs, etc.)</li>
            <li>Skips: node_modules, .git, build artifacts, large files</li>
          </ul>
        )}
        {mode === 'remote' && (
          <ul>
            <li>HTTPS only (GitHub.com or GitHub Enterprise)</li>
            <li>Private repos require a token saved per host</li>
            <li>Repos are cached locally and cleaned via LRU limits</li>
            <li>Indexes: docs, configs, and common code files</li>
          </ul>
        )}
      </div>

      <div className="ingest-form-actions">
        {mode === 'local' && (
          <Button
            variant="primary"
            onClick={handleIngest}
            disabled={!repoPath.trim() || ingesting}
          >
            {ingesting ? 'Ingesting...' : 'Ingest Repository'}
          </Button>
        )}
        {mode === 'remote' && (
          <Button
            variant="primary"
            onClick={handleIngestRemote}
            disabled={!repoUrl.trim() || ingesting}
          >
            {ingesting ? 'Ingesting...' : 'Ingest Repository'}
          </Button>
        )}
      </div>
    </div>
  );
}
