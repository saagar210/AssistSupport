import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Button } from '../shared/Button';
import './ChunkEditor.css';

interface ChunkEditorProps {
  chunkId: string;
  initialContent: string;
  onSave: (newContent: string) => void;
  onCancel: () => void;
}

export function ChunkEditor({ chunkId, initialContent, onSave, onCancel }: ChunkEditorProps) {
  const [content, setContent] = useState(initialContent);
  const [saving, setSaving] = useState(false);
  const [showPreview, setShowPreview] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSave = useCallback(async () => {
    if (content === initialContent) {
      onCancel();
      return;
    }
    setSaving(true);
    setError(null);
    try {
      await invoke('update_chunk_content', { chunkId, content });
      onSave(content);
    } catch (err) {
      setError(String(err));
    } finally {
      setSaving(false);
    }
  }, [chunkId, content, initialContent, onSave, onCancel]);

  const hasChanges = content !== initialContent;

  return (
    <div className="chunk-editor">
      <div className="chunk-editor-header">
        <div className="chunk-editor-tabs">
          <button
            className={`chunk-editor-tab ${!showPreview ? 'active' : ''}`}
            onClick={() => setShowPreview(false)}
          >
            Edit
          </button>
          <button
            className={`chunk-editor-tab ${showPreview ? 'active' : ''}`}
            onClick={() => setShowPreview(true)}
          >
            Preview
          </button>
        </div>
      </div>

      {showPreview ? (
        <div className="chunk-editor-preview">
          {content || <span className="chunk-editor-empty">No content</span>}
        </div>
      ) : (
        <textarea
          className="chunk-editor-textarea"
          value={content}
          onChange={e => setContent(e.target.value)}
          rows={10}
          autoFocus
        />
      )}

      {error && <div className="chunk-editor-error">{error}</div>}

      <div className="chunk-editor-actions">
        <Button variant="ghost" size="small" onClick={onCancel} disabled={saving}>
          Cancel
        </Button>
        <Button
          variant="primary"
          size="small"
          onClick={handleSave}
          disabled={!hasChanges || saving}
          loading={saving}
        >
          Save
        </Button>
      </div>
    </div>
  );
}
