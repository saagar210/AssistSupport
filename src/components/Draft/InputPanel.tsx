import { useState, useCallback, useRef, useEffect, KeyboardEvent } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Button } from '../shared/Button';
import { useJira } from '../../hooks/useJira';
import type { ResponseLength, OcrResult } from '../../types';
import './InputPanel.css';

interface InputPanelProps {
  value: string;
  onChange: (value: string) => void;
  ocrText: string | null;
  onOcrTextChange: (text: string | null) => void;
  onGenerate: () => void;
  onClear: () => void;
  generating: boolean;
  modelLoaded: boolean;
  responseLength: ResponseLength;
  onResponseLengthChange: (length: ResponseLength) => void;
}

export function InputPanel({
  value,
  onChange,
  ocrText,
  onOcrTextChange,
  onGenerate,
  onClear,
  generating,
  modelLoaded,
  responseLength,
  onResponseLengthChange,
}: InputPanelProps) {
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const { checkConfiguration, getTicket, configured, loading: jiraLoading } = useJira();
  const [showJiraImport, setShowJiraImport] = useState(false);
  const [jiraTicketKey, setJiraTicketKey] = useState('');
  const [jiraError, setJiraError] = useState<string | null>(null);

  useEffect(() => {
    checkConfiguration();
  }, [checkConfiguration]);

  const handleKeyDown = useCallback((e: KeyboardEvent) => {
    // Cmd+G to generate
    if (e.metaKey && e.key === 'g') {
      e.preventDefault();
      if (modelLoaded && !generating && value.trim()) {
        onGenerate();
      }
    }
    // Cmd+N to clear
    if (e.metaKey && e.key === 'n') {
      e.preventDefault();
      onClear();
    }
  }, [modelLoaded, generating, value, onGenerate, onClear]);

  const handlePaste = useCallback(async (e: React.ClipboardEvent) => {
    const items = e.clipboardData.items;
    for (const item of items) {
      if (item.type.startsWith('image/')) {
        e.preventDefault();
        const blob = item.getAsFile();
        if (blob) {
          try {
            onOcrTextChange('[Processing image...]');

            // Convert blob to base64
            const arrayBuffer = await blob.arrayBuffer();
            const bytes = new Uint8Array(arrayBuffer);
            let binary = '';
            for (let i = 0; i < bytes.byteLength; i++) {
              binary += String.fromCharCode(bytes[i]);
            }
            const base64 = btoa(binary);

            // Process OCR via backend
            const result = await invoke<OcrResult>('process_ocr_bytes', { imageBase64: base64 });
            if (result.text.trim()) {
              onOcrTextChange(result.text);
            } else {
              onOcrTextChange('[No text detected in image]');
            }
          } catch (err) {
            console.error('OCR failed:', err);
            onOcrTextChange(`[OCR failed: ${err}]`);
          }
        }
        return;
      }
    }
  }, [onOcrTextChange]);

  const handleJiraImport = useCallback(async () => {
    if (!jiraTicketKey.trim()) return;

    setJiraError(null);
    try {
      const ticket = await getTicket(jiraTicketKey.trim().toUpperCase());

      // Format ticket content
      let content = `[${ticket.key}] ${ticket.summary}\n\n`;
      if (ticket.description) {
        content += ticket.description;
      }
      content += `\n\nStatus: ${ticket.status}`;
      if (ticket.priority) {
        content += ` | Priority: ${ticket.priority}`;
      }
      content += ` | Type: ${ticket.issue_type}`;

      onChange(content);
      setShowJiraImport(false);
      setJiraTicketKey('');
    } catch (err) {
      setJiraError(err instanceof Error ? err.message : String(err));
    }
  }, [jiraTicketKey, getTicket, onChange]);

  const wordCount = value.trim().split(/\s+/).filter(Boolean).length;

  return (
    <>
      <div className="panel-header">
        <h3>Input</h3>
        <div className="input-actions">
          {configured && (
            <Button
              variant="ghost"
              size="small"
              onClick={() => setShowJiraImport(true)}
              disabled={jiraLoading}
            >
              Import Jira
            </Button>
          )}
          <select
            className="response-length-select"
            value={responseLength}
            onChange={e => onResponseLengthChange(e.target.value as ResponseLength)}
          >
            <option value="Short">Short (~80 words)</option>
            <option value="Medium">Medium (~160 words)</option>
            <option value="Long">Long (~300 words)</option>
          </select>
          <Button
            variant="ghost"
            size="small"
            onClick={onClear}
            disabled={!value && !ocrText}
          >
            Clear
          </Button>
          <Button
            variant="primary"
            size="small"
            onClick={onGenerate}
            loading={generating}
            disabled={!modelLoaded || !value.trim()}
          >
            Generate
          </Button>
        </div>
      </div>

      <div className="panel-content input-content">
        <textarea
          ref={textareaRef}
          className="input-textarea"
          placeholder="Paste ticket content or describe the issue..."
          value={value}
          onChange={e => onChange(e.target.value)}
          onKeyDown={handleKeyDown}
          onPaste={handlePaste}
        />

        {ocrText && (
          <div className="ocr-preview">
            <div className="ocr-header">
              <span>Screenshot Text (OCR)</span>
              <button
                className="ocr-remove"
                onClick={() => onOcrTextChange(null)}
                aria-label="Remove OCR text"
              >
                &times;
              </button>
            </div>
            <pre className="ocr-text">{ocrText}</pre>
          </div>
        )}

        <div className="input-footer">
          <span className="word-count">{wordCount} words</span>
          <span className={`model-status ${modelLoaded ? 'model-ready' : 'model-warning'}`}>
            {modelLoaded ? '● Model ready' : '○ No model loaded'}
          </span>
        </div>
      </div>

      {/* Jira Import Modal */}
      {showJiraImport && (
        <div className="jira-import-overlay" onClick={() => setShowJiraImport(false)}>
          <div className="jira-import-modal" onClick={e => e.stopPropagation()}>
            <h4>Import from Jira</h4>
            <div className="jira-import-form">
              <input
                type="text"
                placeholder="Ticket key (e.g., HELP-123)"
                value={jiraTicketKey}
                onChange={e => setJiraTicketKey(e.target.value.toUpperCase())}
                onKeyDown={e => {
                  if (e.key === 'Enter') {
                    handleJiraImport();
                  }
                }}
                autoFocus
              />
              {jiraError && <p className="jira-import-error">{jiraError}</p>}
              <div className="jira-import-actions">
                <Button
                  variant="ghost"
                  size="small"
                  onClick={() => setShowJiraImport(false)}
                >
                  Cancel
                </Button>
                <Button
                  variant="primary"
                  size="small"
                  onClick={handleJiraImport}
                  loading={jiraLoading}
                  disabled={!jiraTicketKey.trim()}
                >
                  Import
                </Button>
              </div>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
