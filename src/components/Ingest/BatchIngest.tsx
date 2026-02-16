import { useState } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import { useIngest } from '../../hooks/useIngest';
import { Button } from '../shared/Button';

interface BatchIngestProps {
  onSuccess: (message: string) => void;
  onError: (message: string) => void;
}

export function BatchIngest({ onSuccess, onError }: BatchIngestProps) {
  const { processSourceFile, ingesting } = useIngest();
  const [filePath, setFilePath] = useState('');
  const [lastResult, setLastResult] = useState<{
    successful: number;
    failed: number;
  } | null>(null);

  const handleBrowse = async () => {
    try {
      const selected = await open({
        multiple: false,
        title: 'Select Source Definition File',
        filters: [{
          name: 'YAML Files',
          extensions: ['yaml', 'yml'],
        }],
      });
      if (selected && typeof selected === 'string') {
        setFilePath(selected);
      }
    } catch (e) {
      onError(`Failed to open file picker: ${e}`);
    }
  };

  const handleIngest = async () => {
    if (!filePath.trim()) return;

    try {
      const result = await processSourceFile(filePath.trim());
      setLastResult({
        successful: result.successful.length,
        failed: result.failed.length,
      });

      if (result.cancelled) {
        onError('Batch ingestion was cancelled');
      } else if (result.failed.length > 0) {
        const totalChunks = result.successful.reduce((sum, r) => sum + r.chunk_count, 0);
        onSuccess(
          `Partially completed: ${result.successful.length} sources ingested (${totalChunks} chunks), ${result.failed.length} failed`
        );
      } else {
        const totalChunks = result.successful.reduce((sum, r) => sum + r.chunk_count, 0);
        onSuccess(`Ingested ${result.successful.length} sources (${totalChunks} chunks)`);
      }
    } catch (e) {
      onError(`Failed to process source file: ${e}`);
    }
  };

  return (
    <div className="ingest-form">
      <div className="ingest-form-header">
        <h3>Batch Import</h3>
        <p>Import multiple sources from a YAML definition file. This allows you to define and manage your knowledge sources declaratively.</p>
      </div>

      <div className="ingest-form-field">
        <label htmlFor="batch-file">Source Definition File (YAML)</label>
        <div className="path-input-row">
          <input
            id="batch-file"
            type="text"
            placeholder="/path/to/sources.yaml"
            value={filePath}
            onChange={(e) => setFilePath(e.target.value)}
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
        <details>
          <summary>Example YAML format</summary>
          <pre className="yaml-example">{`namespace: it-support
sources:
  - name: microsoft-docs
    type: url
    uri: https://learn.microsoft.com/...
    enabled: true
  - name: tutorial-video
    type: youtube
    uri: https://www.youtube.com/watch?v=...
    enabled: true
  - name: local-repo
    type: github
    uri: /path/to/repo
    enabled: false`}</pre>
        </details>
      </div>

      {lastResult && (
        <div className="ingest-result">
          <span className="result-success">{lastResult.successful} succeeded</span>
          {lastResult.failed > 0 && (
            <span className="result-failed">{lastResult.failed} failed</span>
          )}
        </div>
      )}

      <div className="ingest-form-actions">
        <Button
          variant="primary"
          onClick={handleIngest}
          disabled={!filePath.trim() || ingesting}
        >
          {ingesting ? 'Processing...' : 'Process Source File'}
        </Button>
      </div>
    </div>
  );
}
