import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Button } from '../shared/Button';
import { useToastContext } from '../../contexts/ToastContext';
import type { JiraTransition } from '../../types';
import './JiraPostPanel.css';

interface JiraPostPanelProps {
  ticketKey: string | null;
  responseText: string;
  draftId: string | null;
}

export function JiraPostPanel({ ticketKey, responseText, draftId }: JiraPostPanelProps) {
  const { success: showSuccess, error: showError } = useToastContext();
  const [transitions, setTransitions] = useState<JiraTransition[]>([]);
  const [selectedTransition, setSelectedTransition] = useState<string>('');
  const [posting, setPosting] = useState(false);
  const [loadingTransitions, setLoadingTransitions] = useState(false);

  const loadTransitions = useCallback(async () => {
    if (!ticketKey) return;
    setLoadingTransitions(true);
    try {
      const data = await invoke<JiraTransition[]>('get_jira_transitions', { ticketKey });
      setTransitions(data);
    } catch (err) {
      console.error('Failed to load transitions:', err);
      setTransitions([]);
    } finally {
      setLoadingTransitions(false);
    }
  }, [ticketKey]);

  useEffect(() => {
    loadTransitions();
  }, [loadTransitions]);

  const handlePostAndTransition = useCallback(async () => {
    if (!ticketKey || !responseText.trim()) return;
    setPosting(true);
    try {
      await invoke<string>('post_and_transition', {
        ticketKey,
        comment: responseText,
        transitionId: selectedTransition || null,
        draftId,
      });
      const action = selectedTransition
        ? 'Posted comment and updated status'
        : 'Posted comment';
      showSuccess(`${action} on ${ticketKey}`);
    } catch (err) {
      showError(`Jira operation failed: ${err}`);
    } finally {
      setPosting(false);
    }
  }, [ticketKey, responseText, selectedTransition, draftId, showSuccess, showError]);

  if (!ticketKey) return null;

  return (
    <div className="jira-post-panel">
      <div className="jira-post-header">
        <span className="jira-post-label">Post to Jira</span>
        <span className="jira-post-ticket">{ticketKey}</span>
      </div>

      <div className="jira-post-controls">
        <div className="jira-transition-select">
          <label htmlFor="jira-transition">Update status (optional)</label>
          <select
            id="jira-transition"
            className="select select-sm"
            value={selectedTransition}
            onChange={(e) => setSelectedTransition(e.target.value)}
            disabled={loadingTransitions || transitions.length === 0}
          >
            <option value="">Keep current status</option>
            {transitions.map((t) => (
              <option key={t.id} value={t.id}>
                {t.name} → {t.to_status}
              </option>
            ))}
          </select>
          {loadingTransitions && (
            <span className="jira-transition-loading">Loading...</span>
          )}
        </div>

        <Button
          variant="primary"
          size="small"
          onClick={handlePostAndTransition}
          disabled={!responseText.trim() || posting}
          loading={posting}
          className="btn-hover-scale"
        >
          {selectedTransition ? 'Post & Update Status' : 'Post Comment'}
        </Button>
      </div>
    </div>
  );
}
