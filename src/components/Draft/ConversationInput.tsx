import { useState, useRef, useCallback, KeyboardEvent } from 'react';
import { Button } from '../shared/Button';
import type { ResponseLength } from '../../types';
import './ConversationInput.css';

interface ConversationInputProps {
  onSubmit: (text: string) => void;
  generating: boolean;
  modelLoaded: boolean;
  responseLength: ResponseLength;
  onResponseLengthChange: (length: ResponseLength) => void;
  onCancel?: () => void;
}

export function ConversationInput({
  onSubmit,
  generating,
  modelLoaded,
  responseLength,
  onResponseLengthChange,
  onCancel,
}: ConversationInputProps) {
  const [text, setText] = useState('');
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const handleSubmit = useCallback(() => {
    if (!text.trim() || generating || !modelLoaded) return;
    onSubmit(text.trim());
    setText('');
    // Reset textarea height
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto';
    }
  }, [text, generating, modelLoaded, onSubmit]);

  const handleKeyDown = useCallback((e: KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSubmit();
    }
  }, [handleSubmit]);

  // Auto-resize textarea
  const handleChange = useCallback((value: string) => {
    setText(value);
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto';
      textareaRef.current.style.height = Math.min(textareaRef.current.scrollHeight, 120) + 'px';
    }
  }, []);

  return (
    <div className="conversation-input-bar">
      <div className="conversation-input-controls">
        <select
          className="conversation-length-select"
          value={responseLength}
          onChange={e => onResponseLengthChange(e.target.value as ResponseLength)}
        >
          <option value="Short">Short</option>
          <option value="Medium">Medium</option>
          <option value="Long">Long</option>
        </select>
      </div>
      <div className="conversation-input-row">
        <textarea
          ref={textareaRef}
          className="conversation-textarea"
          placeholder={modelLoaded ? 'Type your support query...' : 'Load a model first...'}
          value={text}
          onChange={e => handleChange(e.target.value)}
          onKeyDown={handleKeyDown}
          disabled={!modelLoaded}
          rows={1}
        />
        {generating ? (
          <Button variant="ghost" size="small" onClick={onCancel}>
            Cancel
          </Button>
        ) : (
          <Button
            variant="primary"
            size="small"
            onClick={handleSubmit}
            disabled={!text.trim() || !modelLoaded}
          >
            Send
          </Button>
        )}
      </div>
    </div>
  );
}
