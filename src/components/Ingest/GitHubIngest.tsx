import { useState } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import { useIngest } from '../../hooks/useIngest';
import { Button } from '../shared/Button';

interface GitHubIngestProps {
  namespaceId: string;
  onSuccess: (message: string) => void;
  onError: (message: string) => void;
}

export function GitHubIngest({ namespaceId, onSuccess, onError }: GitHubIngestProps) {
  const { ingestGithub, ingesting } = useIngest();
  const [repoPath, setRepoPath] = useState('');

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

  return (
    <div className="ingest-form">
      <div className="ingest-form-header">
        <h3>GitHub Repository</h3>
        <p>Ingest documentation and code from a local Git repository. README files, docs folder, and source code will be processed.</p>
      </div>

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

      <div className="ingest-form-info">
        <ul>
          <li>Must be a valid Git repository (contains .git folder)</li>
          <li>Indexes: .md, .mdx, .rst, .txt, .adoc files</li>
          <li>Also indexes common code files (.py, .js, .ts, .rs, etc.)</li>
          <li>Skips: node_modules, .git, build artifacts, large files</li>
        </ul>
      </div>

      <div className="ingest-form-actions">
        <Button
          variant="primary"
          onClick={handleIngest}
          disabled={!repoPath.trim() || ingesting}
        >
          {ingesting ? 'Ingesting...' : 'Ingest Repository'}
        </Button>
      </div>
    </div>
  );
}
