import { useState, useCallback, useRef, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Button } from '../shared/Button';
import type { ContextSource } from '../../types';
import './ResponsePanel.css';

type ExportFormat = 'Markdown' | 'PlainText' | 'Html';

interface ResponsePanelProps {
  response: string;
  streamingText: string;
  isStreaming: boolean;
  sources: ContextSource[];
  generating: boolean;
  onSaveDraft?: () => void;
  onCancel?: () => void;
  hasInput?: boolean;
}

export function ResponsePanel({ response, streamingText, isStreaming, sources, generating, onSaveDraft, onCancel, hasInput }: ResponsePanelProps) {
  const [copied, setCopied] = useState(false);
  const [showSources, setShowSources] = useState(false);
  const [showExportMenu, setShowExportMenu] = useState(false);
  const exportMenuRef = useRef<HTMLDivElement>(null);

  // Close export menu when clicking outside
  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (exportMenuRef.current && !exportMenuRef.current.contains(event.target as Node)) {
        setShowExportMenu(false);
      }
    }
    if (showExportMenu) {
      document.addEventListener('mousedown', handleClickOutside);
      return () => document.removeEventListener('mousedown', handleClickOutside);
    }
  }, [showExportMenu]);

  const handleExport = useCallback(async (format: ExportFormat) => {
    if (!response) return;
    try {
      const saved = await invoke<boolean>('export_draft', {
        responseText: response,
        format,
      });
      if (saved) {
        // Could show a toast here
        console.log('Response exported successfully');
      }
    } catch (err) {
      console.error('Export failed:', err);
    }
    setShowExportMenu(false);
  }, [response]);

  const handleCopy = useCallback(async () => {
    if (!response) return;
    try {
      await navigator.clipboard.writeText(response);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error('Copy failed:', err);
    }
  }, [response]);

  const wordCount = response.trim().split(/\s+/).filter(Boolean).length;

  return (
    <>
      <div className="panel-header">
        <h3>Response</h3>
        <div className="response-actions">
          {sources.length > 0 && (
            <Button
              variant="ghost"
              size="small"
              onClick={() => setShowSources(!showSources)}
            >
              {showSources ? 'Hide Sources' : `Sources (${sources.length})`}
            </Button>
          )}
          {onSaveDraft && (
            <Button
              variant="ghost"
              size="small"
              onClick={onSaveDraft}
              disabled={!hasInput}
            >
              Save Draft
            </Button>
          )}
          <Button
            variant="secondary"
            size="small"
            onClick={handleCopy}
            disabled={!response}
          >
            {copied ? 'Copied!' : 'Copy'}
          </Button>
          <div className="export-dropdown-wrapper" ref={exportMenuRef}>
            <Button
              variant="secondary"
              size="small"
              onClick={() => setShowExportMenu(!showExportMenu)}
              disabled={!response}
            >
              Export ▾
            </Button>
            {showExportMenu && (
              <div className="export-dropdown-menu">
                <button className="export-option" onClick={() => handleExport('Markdown')}>
                  Markdown (.md)
                </button>
                <button className="export-option" onClick={() => handleExport('PlainText')}>
                  Plain Text (.txt)
                </button>
                <button className="export-option" onClick={() => handleExport('Html')}>
                  HTML (.html)
                </button>
              </div>
            )}
          </div>
        </div>
      </div>

      <div className="panel-content response-content">
        {generating && !streamingText ? (
          <div className="generating-indicator">
            <div className="generating-spinner" />
            <span>Generating response...</span>
            {onCancel && (
              <Button variant="ghost" size="small" onClick={onCancel}>
                Cancel
              </Button>
            )}
          </div>
        ) : isStreaming || streamingText ? (
          <>
            <div className="response-text">
              {streamingText}
              {isStreaming && <span className="streaming-cursor">|</span>}
            </div>
            {isStreaming && onCancel && (
              <div className="streaming-actions">
                <Button variant="ghost" size="small" onClick={onCancel}>
                  Cancel
                </Button>
              </div>
            )}
          </>
        ) : response ? (
          <>
            <div className="response-text">{response}</div>

            {showSources && sources.length > 0 && (
              <div className="sources-panel">
                <h4>Knowledge Base Sources</h4>
                <ul className="sources-list">
                  {sources.map((source, i) => (
                    <li key={source.chunk_id} className="source-item">
                      <span className="source-number">[{i + 1}]</span>
                      <div className="source-info">
                        <span className="source-title">
                          {source.title || source.file_path}
                        </span>
                        {source.heading_path && (
                          <span className="source-heading">
                            &rsaquo; {source.heading_path}
                          </span>
                        )}
                        <span className="source-score">
                          Score: {(source.score * 100).toFixed(0)}%
                        </span>
                      </div>
                    </li>
                  ))}
                </ul>
              </div>
            )}
          </>
        ) : (
          <div className="response-placeholder">
            Generated response will appear here...
          </div>
        )}

        {response && (
          <div className="response-footer">
            <span className="word-count">{wordCount} words</span>
          </div>
        )}
      </div>
    </>
  );
}
