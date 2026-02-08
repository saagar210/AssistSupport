import { useCallback, useEffect, useMemo, useState } from 'react';
import { Button } from '../../components/shared/Button';
import { FollowUpsTab } from '../../components/FollowUps/FollowUpsTab';
import { useDrafts } from '../../hooks/useDrafts';
import type { SavedDraft } from '../../types';
import {
  buildQueueItems,
  filterQueueItems,
  formatQueueTimestamp,
  loadQueueMeta,
  persistQueueMeta,
  summarizeQueue,
  type QueueMetaMap,
  type QueuePriority,
  type QueueState,
  type QueueView,
} from './queueModel';
import './QueueFirstInboxPage.css';

interface QueueFirstInboxPageProps {
  onLoadDraft: (draft: SavedDraft) => void;
  initialQueueView?: QueueView | null;
  onQueueViewConsumed?: () => void;
}

const QUEUE_OPERATOR_STORAGE_KEY = 'assistsupport.queue.operator';

const QUEUE_VIEWS: Array<{ id: QueueView; label: string }> = [
  { id: 'all', label: 'All' },
  { id: 'unassigned', label: 'Unassigned' },
  { id: 'at_risk', label: 'At Risk' },
  { id: 'in_progress', label: 'In Progress' },
  { id: 'resolved', label: 'Resolved' },
];

function formatTicketLabel(draft: SavedDraft): string {
  return draft.ticket_id?.trim() || `Draft ${draft.id.slice(0, 8)}`;
}

function truncate(value: string, limit: number): string {
  if (value.length <= limit) {
    return value;
  }
  return `${value.slice(0, limit)}...`;
}

function loadOperatorName(): string {
  if (typeof window === 'undefined') {
    return 'current-operator';
  }

  try {
    return localStorage.getItem(QUEUE_OPERATOR_STORAGE_KEY) || 'current-operator';
  } catch {
    return 'current-operator';
  }
}

export function QueueFirstInboxPage({
  onLoadDraft,
  initialQueueView = null,
  onQueueViewConsumed,
}: QueueFirstInboxPageProps) {
  const { drafts, loading, loadDrafts } = useDrafts();
  const [queueMetaMap, setQueueMetaMap] = useState<QueueMetaMap>(() => loadQueueMeta());
  const [queueView, setQueueView] = useState<QueueView>('all');
  const [searchQuery, setSearchQuery] = useState('');
  const [operatorName, setOperatorName] = useState(loadOperatorName);

  useEffect(() => {
    loadDrafts(100);
  }, [loadDrafts]);

  useEffect(() => {
    if (!initialQueueView) {
      return;
    }

    setQueueView(initialQueueView);
    onQueueViewConsumed?.();
  }, [initialQueueView, onQueueViewConsumed]);

  useEffect(() => {
    try {
      localStorage.setItem(QUEUE_OPERATOR_STORAGE_KEY, operatorName);
    } catch {
      // Ignore local storage write failures to keep queue workflows usable.
    }
  }, [operatorName]);

  const queueItems = useMemo(() => buildQueueItems(drafts, queueMetaMap), [drafts, queueMetaMap]);
  const queueSummary = useMemo(() => summarizeQueue(queueItems), [queueItems]);

  const filteredItems = useMemo(() => {
    const scoped = filterQueueItems(queueItems, queueView);
    if (!searchQuery.trim()) {
      return scoped;
    }

    const normalized = searchQuery.toLowerCase();
    return scoped.filter((item) => {
      const ticket = item.draft.ticket_id?.toLowerCase() ?? '';
      const summary = item.draft.summary_text?.toLowerCase() ?? '';
      const input = item.draft.input_text.toLowerCase();
      return ticket.includes(normalized) || summary.includes(normalized) || input.includes(normalized);
    });
  }, [queueItems, queueView, searchQuery]);

  const updateQueueMeta = useCallback((draftId: string, updates: Partial<{ owner: string; state: QueueState; priority: QueuePriority }>) => {
    setQueueMetaMap((prev) => {
      const existing = prev[draftId] ?? {
        owner: 'unassigned',
        state: 'open' as QueueState,
        priority: 'normal' as QueuePriority,
        updatedAt: new Date().toISOString(),
      };

      const next: QueueMetaMap = {
        ...prev,
        [draftId]: {
          ...existing,
          ...updates,
          updatedAt: new Date().toISOString(),
        },
      };

      persistQueueMeta(next);
      return next;
    });
  }, []);

  const handleClaim = useCallback((draftId: string) => {
    updateQueueMeta(draftId, { owner: operatorName, state: 'in_progress' });
  }, [operatorName, updateQueueMeta]);

  const handleResolve = useCallback((draftId: string) => {
    updateQueueMeta(draftId, { state: 'resolved' });
  }, [updateQueueMeta]);

  const handleReopen = useCallback((draftId: string) => {
    updateQueueMeta(draftId, { state: 'open' });
  }, [updateQueueMeta]);

  const handlePriorityChange = useCallback((draftId: string, priority: QueuePriority) => {
    updateQueueMeta(draftId, { priority });
  }, [updateQueueMeta]);

  return (
    <div className="queue-inbox-page" data-testid="queue-first-inbox">
      <section className="queue-inbox-banner" aria-live="polite">
        <h2>Queue-first inbox mode</h2>
        <p>
          Follow-up history remains available below while queue workflows are staged under flags.
        </p>
      </section>

      <section className="queue-operator-controls" aria-label="Queue operator settings">
        <label htmlFor="queue-operator-name">Operator</label>
        <input
          id="queue-operator-name"
          value={operatorName}
          onChange={(event) => setOperatorName(event.target.value || 'current-operator')}
          maxLength={64}
        />
      </section>

      <section className="queue-summary" aria-label="Queue summary metrics">
        <div className="queue-metric"><span>Total</span><strong>{queueSummary.total}</strong></div>
        <div className="queue-metric"><span>Unassigned</span><strong>{queueSummary.unassigned}</strong></div>
        <div className="queue-metric"><span>In Progress</span><strong>{queueSummary.inProgress}</strong></div>
        <div className="queue-metric"><span>At Risk</span><strong>{queueSummary.atRisk}</strong></div>
      </section>

      <section className="queue-filters" aria-label="Queue filters">
        {QUEUE_VIEWS.map((view) => (
          <Button
            key={view.id}
            variant={queueView === view.id ? 'primary' : 'ghost'}
            size="small"
            onClick={() => setQueueView(view.id)}
          >
            {view.label}
          </Button>
        ))}
        <input
          className="queue-search"
          placeholder="Search queue..."
          value={searchQuery}
          onChange={(event) => setSearchQuery(event.target.value)}
        />
      </section>

      <section className="queue-list" aria-label="Queue work items">
        {loading && <p className="queue-loading">Loading queue...</p>}
        {!loading && filteredItems.length === 0 && (
          <p className="queue-empty">No queue items for this view.</p>
        )}

        {!loading && filteredItems.length > 0 && (
          <ul>
            {filteredItems.map((item) => (
              <li key={item.draft.id} className="queue-item" data-priority={item.meta.priority}>
                <div className="queue-item-head">
                  <div>
                    <strong>{formatTicketLabel(item.draft)}</strong>
                    <p>{truncate(item.draft.summary_text || item.draft.input_text, 120)}</p>
                  </div>
                  <div className="queue-badges">
                    <span className={`badge badge-priority-${item.meta.priority}`}>{item.meta.priority}</span>
                    <span className={`badge badge-state-${item.meta.state}`}>{item.meta.state.replace('_', ' ')}</span>
                    {item.isAtRisk && <span className="badge badge-risk">at risk</span>}
                  </div>
                </div>

                <div className="queue-item-meta">
                  <span>Owner: {item.meta.owner}</span>
                  <span>SLA due: {formatQueueTimestamp(item.slaDueAt)}</span>
                  <label>
                    Priority
                    <select
                      value={item.meta.priority}
                      onChange={(event) => handlePriorityChange(item.draft.id, event.target.value as QueuePriority)}
                    >
                      <option value="low">low</option>
                      <option value="normal">normal</option>
                      <option value="high">high</option>
                      <option value="urgent">urgent</option>
                    </select>
                  </label>
                </div>

                <div className="queue-item-actions">
                  <Button size="small" variant="secondary" onClick={() => onLoadDraft(item.draft)}>
                    Open Draft
                  </Button>
                  {item.meta.owner === 'unassigned' && item.meta.state !== 'resolved' && (
                    <Button size="small" variant="primary" onClick={() => handleClaim(item.draft.id)}>
                      Claim
                    </Button>
                  )}
                  {item.meta.state !== 'resolved' ? (
                    <Button size="small" variant="ghost" onClick={() => handleResolve(item.draft.id)}>
                      Resolve
                    </Button>
                  ) : (
                    <Button size="small" variant="ghost" onClick={() => handleReopen(item.draft.id)}>
                      Reopen
                    </Button>
                  )}
                </div>
              </li>
            ))}
          </ul>
        )}
      </section>

      <details className="queue-history">
        <summary>History and templates</summary>
        <FollowUpsTab onLoadDraft={onLoadDraft} />
      </details>
    </div>
  );
}
