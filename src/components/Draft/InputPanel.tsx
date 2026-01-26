import { useState, useCallback, useRef, useEffect, KeyboardEvent } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Button } from '../shared/Button';
import { useJira, JiraTicket } from '../../hooks/useJira';
import type { ResponseLength, OcrResult, FirstResponseTone } from '../../types';
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
  ticketId: string | null;
  onTicketIdChange: (id: string | null) => void;
  ticket: JiraTicket | null;
  onTicketChange: (ticket: JiraTicket | null) => void;
  firstResponse: string;
  onFirstResponseChange: (text: string) => void;
  firstResponseTone: FirstResponseTone;
  onFirstResponseToneChange: (tone: FirstResponseTone) => void;
  onGenerateFirstResponse: () => void;
  onCopyFirstResponse: () => void;
  onClearFirstResponse: () => void;
  firstResponseGenerating: boolean;
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
  ticketId,
  onTicketIdChange,
  ticket,
  onTicketChange,
  firstResponse,
  onFirstResponseChange,
  firstResponseTone,
  onFirstResponseToneChange,
  onGenerateFirstResponse,
  onCopyFirstResponse,
  onClearFirstResponse,
  firstResponseGenerating,
}: InputPanelProps) {
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const { checkConfiguration, getTicket, configured } = useJira();
  const [ticketInputValue, setTicketInputValue] = useState(ticketId || '');
  const [ticketFetching, setTicketFetching] = useState(false);
  const [ticketError, setTicketError] = useState<string | null>(null);
  const [showDescription, setShowDescription] = useState(false);
  const fetchTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    checkConfiguration();
  }, [checkConfiguration]);

  // Sync external ticketId changes to local input
  useEffect(() => {
    setTicketInputValue(ticketId || '');
  }, [ticketId]);

  // Debounced ticket fetch
  const handleTicketInputChange = useCallback((value: string) => {
    const upperValue = value.toUpperCase();
    setTicketInputValue(upperValue);
    setTicketError(null);

    // Clear existing timeout
    if (fetchTimeoutRef.current) {
      clearTimeout(fetchTimeoutRef.current);
    }

    // If empty, clear ticket
    if (!upperValue.trim()) {
      onTicketIdChange(null);
      onTicketChange(null);
      return;
    }

    // Debounce fetch by 500ms
    fetchTimeoutRef.current = setTimeout(async () => {
      // Only fetch if it looks like a ticket key (e.g., PROJ-123)
      if (!/^[A-Z]+-\d+$/.test(upperValue.trim())) {
        return;
      }

      setTicketFetching(true);
      try {
        const fetchedTicket = await getTicket(upperValue.trim());
        onTicketIdChange(upperValue.trim());
        onTicketChange(fetchedTicket);
        setTicketError(null);
      } catch (err) {
        setTicketError(err instanceof Error ? err.message : String(err));
        onTicketChange(null);
      } finally {
        setTicketFetching(false);
      }
    }, 500);
  }, [getTicket, onTicketIdChange, onTicketChange]);

  // Cleanup timeout on unmount
  useEffect(() => {
    return () => {
      if (fetchTimeoutRef.current) {
        clearTimeout(fetchTimeoutRef.current);
      }
    };
  }, []);

  const handleClearTicket = useCallback(() => {
    setTicketInputValue('');
    onTicketIdChange(null);
    onTicketChange(null);
    setTicketError(null);
  }, [onTicketIdChange, onTicketChange]);

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

  const wordCount = value.trim().split(/\s+/).filter(Boolean).length;
  const hasFirstResponseInput = Boolean(value.trim() || ticket || ocrText);

  return (
    <>
      <div className="panel-header">
        <h3>Input</h3>
        <div className="input-actions">
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
            disabled={!value && !ocrText && !ticket && !firstResponse}
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

        {/* Jira Ticket Section */}
        {configured && (
          <div className="ticket-section">
            <div className="ticket-input-row">
              <label htmlFor="ticket-id">Ticket:</label>
              <input
                id="ticket-id"
                type="text"
                className="ticket-input"
                placeholder="PROJ-123"
                value={ticketInputValue}
                onChange={e => handleTicketInputChange(e.target.value)}
              />
              {ticketInputValue && (
                <button
                  className="ticket-clear"
                  onClick={handleClearTicket}
                  aria-label="Clear ticket"
                >
                  &times;
                </button>
              )}
              {ticketFetching && <span className="ticket-loading">Loading...</span>}
            </div>

            {ticketError && (
              <div className="ticket-error">{ticketError}</div>
            )}

            {ticket && !ticketError && (
              <div className="ticket-preview">
                <div className="ticket-header">
                  <span className="ticket-key">{ticket.key}</span>
                  <span className={`ticket-status status-${ticket.status.toLowerCase().replace(/\s+/g, '-')}`}>
                    {ticket.status}
                  </span>
                  {ticket.priority && (
                    <span className={`ticket-priority priority-${ticket.priority.toLowerCase()}`}>
                      {ticket.priority}
                    </span>
                  )}
                </div>
                <div className="ticket-summary">{ticket.summary}</div>
                {ticket.description && (
                  <button
                    className="ticket-description-toggle"
                    onClick={() => setShowDescription(!showDescription)}
                  >
                    {showDescription ? '▼ Hide description' : '▶ Show description'}
                  </button>
                )}
                {showDescription && ticket.description && (
                  <div className="ticket-description">{ticket.description}</div>
                )}
              </div>
            )}
          </div>
        )}

        <div className="first-response-section">
          <div className="first-response-header">
            <h4>First Response</h4>
            <div className="first-response-actions">
              <select
                className="first-response-tone"
                value={firstResponseTone}
                onChange={e => onFirstResponseToneChange(e.target.value as FirstResponseTone)}
              >
                <option value="slack">Slack (friendly)</option>
                <option value="jira">Jira (concise)</option>
              </select>
              <Button
                variant="ghost"
                size="small"
                onClick={onGenerateFirstResponse}
                loading={firstResponseGenerating}
                disabled={!modelLoaded || !hasFirstResponseInput}
              >
                Draft Reply
              </Button>
              <Button
                variant="secondary"
                size="small"
                onClick={onCopyFirstResponse}
                disabled={!firstResponse.trim()}
              >
                Copy
              </Button>
              <Button
                variant="ghost"
                size="small"
                onClick={onClearFirstResponse}
                disabled={!firstResponse.trim()}
              >
                Clear Reply
              </Button>
            </div>
          </div>
          <textarea
            className="first-response-textarea"
            placeholder="Drafted first response will appear here..."
            value={firstResponse}
            onChange={e => onFirstResponseChange(e.target.value)}
          />
        </div>

        <div className="input-footer">
          <span className="word-count">{wordCount} words</span>
          <span className={`model-status ${modelLoaded ? 'model-ready' : 'model-warning'}`}>
            {modelLoaded ? '● Model ready' : '○ No model loaded'}
          </span>
        </div>
      </div>

    </>
  );
}
