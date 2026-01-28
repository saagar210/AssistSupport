import { useState, useCallback } from 'react';
import { useBatch } from '../../hooks/useBatch';
import { Button } from '../shared/Button';
import type { ResponseLength } from '../../types';
import './BatchPanel.css';

interface BatchPanelProps {
  responseLength: ResponseLength;
  modelLoaded: boolean;
}

export function BatchPanel({ responseLength, modelLoaded }: BatchPanelProps) {
  const { status, loading, error, startBatch, exportResults, cancelBatch, reset } = useBatch();
  const [inputText, setInputText] = useState('');

  const handleStart = useCallback(async () => {
    const inputs = inputText
      .split('\n')
      .map(line => line.trim())
      .filter(line => line.length > 0);

    if (inputs.length === 0) return;

    try {
      await startBatch(inputs, responseLength);
    } catch (err) {
      console.error('Batch start failed:', err);
    }
  }, [inputText, responseLength, startBatch]);

  const handleExport = useCallback(async () => {
    if (!status?.job_id) return;
    await exportResults(status.job_id, 'markdown');
  }, [status, exportResults]);

  const progress = status ? (status.total > 0 ? (status.completed / status.total) * 100 : 0) : 0;

  return (
    <div className="batch-panel">
      {!status ? (
        // Input phase
        <div className="batch-input-section">
          <div className="batch-instructions">
            <p>Enter one support query per line. Each will be processed with KB search and response generation.</p>
          </div>
          <textarea
            className="batch-textarea"
            placeholder="How do I reset my password?\nVPN is not connecting on macOS\nCannot access shared drive..."
            value={inputText}
            onChange={e => setInputText(e.target.value)}
            rows={8}
            disabled={loading}
          />
          <div className="batch-actions">
            <span className="batch-count">
              {inputText.split('\n').filter(l => l.trim()).length} queries
            </span>
            <Button
              variant="primary"
              onClick={handleStart}
              disabled={!modelLoaded || loading || !inputText.trim()}
              loading={loading}
            >
              Start Batch
            </Button>
          </div>
        </div>
      ) : (
        // Results phase
        <div className="batch-results-section">
          <div className="batch-progress-header">
            <div className="batch-progress-info">
              <span className="batch-status-label">
                {status.status === 'running' ? 'Processing...' :
                 status.status === 'succeeded' ? 'Complete' :
                 status.status === 'failed' ? 'Failed' :
                 status.status === 'cancelled' ? 'Cancelled' : 'Queued'}
              </span>
              <span className="batch-progress-count">
                {status.completed} / {status.total}
              </span>
            </div>
            <div className="batch-progress-bar">
              <div className="batch-progress-fill" style={{ width: `${progress}%` }} />
            </div>
          </div>

          {error && <div className="batch-error">{error}</div>}

          <div className="batch-results-list">
            {status.results.map((result, index) => (
              <div key={index} className="batch-result-card">
                <div className="batch-result-input">
                  <span className="batch-result-label">Query {index + 1}:</span>
                  <span className="batch-result-text">{result.input}</span>
                </div>
                <div className="batch-result-response">
                  <span className="batch-result-label">Response:</span>
                  <div className="batch-result-text">{result.response}</div>
                </div>
                <div className="batch-result-meta">
                  <span>{result.sources.length} sources</span>
                  <span>{(result.duration_ms / 1000).toFixed(1)}s</span>
                </div>
              </div>
            ))}
          </div>

          <div className="batch-actions">
            {status.status === 'running' && (
              <Button variant="ghost" onClick={cancelBatch}>Cancel</Button>
            )}
            {(status.status === 'succeeded' || status.status === 'failed' || status.status === 'cancelled') && (
              <>
                <Button variant="ghost" onClick={reset}>New Batch</Button>
                {status.results.length > 0 && (
                  <Button variant="primary" onClick={handleExport}>Export Results</Button>
                )}
              </>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
