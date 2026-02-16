import { useState, useCallback } from 'react';
import { Button } from '../shared/Button';
import './SaveAsTemplateModal.css';

interface SaveAsTemplateModalProps {
  content: string;
  sourceDraftId?: string;
  sourceRating?: number;
  onSave: (name: string, category: string | null, content: string, variablesJson: string | null) => Promise<boolean>;
  onClose: () => void;
}

const CATEGORIES = ['General', 'Technical', 'Account', 'Billing', 'Troubleshooting', 'Escalation'];

export function SaveAsTemplateModal({ content, sourceDraftId, sourceRating, onSave, onClose }: SaveAsTemplateModalProps) {
  const [name, setName] = useState('');
  const [category, setCategory] = useState('');
  const [editedContent, setEditedContent] = useState(content);
  const [saving, setSaving] = useState(false);

  // Extract variables like {{variable_name}} from content
  const detectedVariables = Array.from(
    new Set(editedContent.match(/\{\{(\w+)\}\}/g)?.map(v => v.slice(2, -2)) ?? [])
  );

  const handleSave = useCallback(async () => {
    if (!name.trim()) return;
    setSaving(true);
    try {
      const variablesJson = detectedVariables.length > 0
        ? JSON.stringify(detectedVariables)
        : null;
      const success = await onSave(name.trim(), category || null, editedContent, variablesJson);
      if (success) {
        onClose();
      }
    } finally {
      setSaving(false);
    }
  }, [name, category, editedContent, detectedVariables, onSave, onClose]);

  return (
    <div className="modal-overlay" onClick={(e) => e.target === e.currentTarget && onClose()}>
      <div className="modal modal-md">
        <div className="modal-header">
          <h3 className="modal-title">Save as Template</h3>
          <button className="modal-close" onClick={onClose}>&times;</button>
        </div>
        <div className="modal-body">
          <div className="template-save-field">
            <label htmlFor="template-name">Template Name</label>
            <input
              id="template-name"
              className="input input-md"
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="e.g., Password Reset Response"
              autoFocus
            />
          </div>

          <div className="template-save-field">
            <label htmlFor="template-category">Category</label>
            <select
              id="template-category"
              className="select select-md"
              value={category}
              onChange={(e) => setCategory(e.target.value)}
            >
              <option value="">Select category...</option>
              {CATEGORIES.map(cat => (
                <option key={cat} value={cat}>{cat}</option>
              ))}
            </select>
          </div>

          <div className="template-save-field">
            <label htmlFor="template-content">Content</label>
            <textarea
              id="template-content"
              className="textarea"
              value={editedContent}
              onChange={(e) => setEditedContent(e.target.value)}
              rows={8}
              placeholder="Template content..."
            />
            <span className="template-save-hint">
              Use {'{{variable_name}}'} for dynamic content. Detected: {detectedVariables.length > 0 ? detectedVariables.join(', ') : 'none'}
            </span>
          </div>

          {sourceDraftId && (
            <div className="template-save-source">
              Source: Draft {sourceDraftId.slice(0, 8)}...
              {sourceRating && ` (rated ${sourceRating}/5)`}
            </div>
          )}
        </div>
        <div className="modal-footer">
          <Button variant="secondary" onClick={onClose}>Cancel</Button>
          <Button
            variant="primary"
            onClick={handleSave}
            disabled={!name.trim() || saving}
            loading={saving}
          >
            Save Template
          </Button>
        </div>
      </div>
    </div>
  );
}
