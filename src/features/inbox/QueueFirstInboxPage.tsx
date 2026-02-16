import { useCallback, useEffect, useMemo, useState } from 'react';
import { Button } from '../../components/shared/Button';
import { FollowUpsTab } from '../../components/FollowUps/FollowUpsTab';
import { useAnalytics } from '../../hooks/useAnalytics';
import { useDrafts } from '../../hooks/useDrafts';
import type { SavedDraft } from '../../types';
import {
  buildQueueHandoffDelta,
  buildQueueHandoffSnapshot,
  buildQueueItems,
  filterQueueItems,
  formatQueueTimestamp,
  loadQueueHandoffSnapshot,
  loadQueueMeta,
  persistQueueHandoffSnapshot,
  persistQueueMeta,
  summarizeQueue,
  summarizeQueueByOwner,
  summarizeQueueByPriority,
  type QueueHandoffSnapshot,
  type QueueMetaMap,
  type QueueItem,
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

function formatSignedDelta(value: number): string {
  if (value === 0) {
    return '0';
  }
  return value > 0 ? `+${value}` : `${value}`;
}

export function QueueFirstInboxPage({
  onLoadDraft,
  initialQueueView = null,
  onQueueViewConsumed,
}: QueueFirstInboxPageProps) {
  const { logEvent } = useAnalytics();
  const { drafts, loading, loadDrafts } = useDrafts();
  const [queueMetaMap, setQueueMetaMap] = useState<QueueMetaMap>(() => loadQueueMeta());
  const [previousSnapshot, setPreviousSnapshot] = useState<QueueHandoffSnapshot | null>(() => loadQueueHandoffSnapshot());
  const [queueView, setQueueView] = useState<QueueView>('all');
  const [searchQuery, setSearchQuery] = useState('');
  const [operatorName, setOperatorName] = useState(loadOperatorName);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [handoffStatus, setHandoffStatus] = useState<string | null>(null);

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
  const queuePrioritySummary = useMemo(() => summarizeQueueByPriority(queueItems), [queueItems]);
  const queueOwnerSummary = useMemo(() => summarizeQueueByOwner(queueItems), [queueItems]);
  const queueHandoffSnapshot = useMemo(() => buildQueueHandoffSnapshot(queueItems), [queueItems]);
  const queueHandoffDelta = useMemo(
    () => buildQueueHandoffDelta(queueHandoffSnapshot, previousSnapshot),
    [queueHandoffSnapshot, previousSnapshot],
  );

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

  useEffect(() => {
    setSelectedIndex((prev) => {
      if (filteredItems.length === 0) {
        return 0;
      }
      return Math.max(0, Math.min(prev, filteredItems.length - 1));
    });
  }, [filteredItems]);

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
    void logEvent('queue_item_claimed', { draft_id: draftId, operator: operatorName });
  }, [logEvent, operatorName, updateQueueMeta]);

  const handleResolve = useCallback((draftId: string) => {
    updateQueueMeta(draftId, { state: 'resolved' });
    void logEvent('queue_item_resolved', { draft_id: draftId, operator: operatorName });
  }, [logEvent, operatorName, updateQueueMeta]);

  const handleReopen = useCallback((draftId: string) => {
    updateQueueMeta(draftId, { state: 'open' });
    void logEvent('queue_item_reopened', { draft_id: draftId, operator: operatorName });
  }, [logEvent, operatorName, updateQueueMeta]);

  const handlePriorityChange = useCallback((draftId: string, priority: QueuePriority) => {
    updateQueueMeta(draftId, { priority });
    void logEvent('queue_item_priority_changed', {
      draft_id: draftId,
      operator: operatorName,
      priority,
    });
  }, [logEvent, operatorName, updateQueueMeta]);

  const currentItem = filteredItems[selectedIndex] ?? null;

  const withCurrentItem = useCallback((handler: (item: QueueItem) => void) => {
    if (!currentItem) {
      return;
    }
    handler(currentItem);
  }, [currentItem]);

  const handleCopyHandoffSnapshot = useCallback(async () => {
    const payload = JSON.stringify(queueHandoffSnapshot, null, 2);
    if (typeof navigator === 'undefined' || !navigator.clipboard?.writeText) {
      setHandoffStatus('Clipboard not available in this environment.');
      return;
    }

    try {
      await navigator.clipboard.writeText(payload);
      setHandoffStatus('Shift handoff snapshot copied.');
      void logEvent('queue_handoff_snapshot_copied', {
        operator: operatorName,
        queue_view: queueView,
        at_risk: queueHandoffSnapshot.summary.atRisk,
      });
      persistQueueHandoffSnapshot(queueHandoffSnapshot);
      setPreviousSnapshot(queueHandoffSnapshot);
    } catch {
      setHandoffStatus('Failed to copy handoff snapshot.');
    }
  }, [logEvent, operatorName, queueHandoffSnapshot, queueView]);

  const handleQueueKeyDown = useCallback((event: React.KeyboardEvent<HTMLElement>) => {
    const target = event.target as HTMLElement;
    const isInputElement = target.tagName === 'INPUT' || target.tagName === 'SELECT' || target.tagName === 'TEXTAREA';
    if (isInputElement) {
      return;
    }

    switch (event.key.toLowerCase()) {
      case 'arrowdown':
      case 'j':
        event.preventDefault();
        setSelectedIndex((prev) => Math.min(prev + 1, Math.max(filteredItems.length - 1, 0)));
        break;
      case 'arrowup':
      case 'k':
        event.preventDefault();
        setSelectedIndex((prev) => Math.max(prev - 1, 0));
        break;
      case 'enter':
        event.preventDefault();
        withCurrentItem((item) => {
          onLoadDraft(item.draft);
          void logEvent('queue_item_opened', {
            draft_id: item.draft.id,
            operator: operatorName,
            entrypoint: 'keyboard',
          });
        });
        break;
      case 'c':
        event.preventDefault();
        withCurrentItem((item) => {
          if (item.meta.owner === 'unassigned' && item.meta.state !== 'resolved') {
            handleClaim(item.draft.id);
          }
        });
        break;
      case 'x':
        event.preventDefault();
        withCurrentItem((item) => {
          if (item.meta.state !== 'resolved') {
            handleResolve(item.draft.id);
          }
        });
        break;
      case 'o':
        event.preventDefault();
        withCurrentItem((item) => {
          if (item.meta.state === 'resolved') {
            handleReopen(item.draft.id);
          }
        });
        break;
      default:
        break;
    }
  }, [filteredItems.length, handleClaim, handleReopen, handleResolve, logEvent, onLoadDraft, operatorName, withCurrentItem]);

  return (
    <div className="queue-inbox-page" data-testid="queue-first-inbox" onKeyDown={handleQueueKeyDown}>
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

      <section className="queue-analytics" aria-label="Shift handoff analytics">
        <div className="queue-analytics-card">
          <h3>Priority Mix</h3>
          <div className="queue-priority-grid">
            <span>Urgent: <strong>{queuePrioritySummary.urgent}</strong></span>
            <span>High: <strong>{queuePrioritySummary.high}</strong></span>
            <span>Normal: <strong>{queuePrioritySummary.normal}</strong></span>
            <span>Low: <strong>{queuePrioritySummary.low}</strong></span>
          </div>
        </div>
        <div className="queue-analytics-card">
          <h3>Owner Workload</h3>
          {queueOwnerSummary.length === 0 ? (
            <p>No active queue ownership yet.</p>
          ) : (
            <ul>
              {queueOwnerSummary.map((owner) => (
                <li key={owner.owner}>
                  <strong>{owner.owner}</strong> · open {owner.openCount} · in progress {owner.inProgressCount}
                  {owner.atRiskCount > 0 ? ` · at risk ${owner.atRiskCount}` : ''}
                </li>
              ))}
            </ul>
          )}
        </div>
        <div className="queue-analytics-card">
          <h3>Shift Handoff</h3>
          <p>Generated: {formatQueueTimestamp(queueHandoffSnapshot.generatedAt)}</p>
          <p>
            Trend vs previous snapshot
            {queueHandoffDelta.previousGeneratedAt
              ? ` (${formatQueueTimestamp(queueHandoffDelta.previousGeneratedAt)})`
              : ' (no baseline yet)'}
          </p>
          <div className="queue-priority-grid queue-trend-grid">
            <span>Open: <strong>{formatSignedDelta(queueHandoffDelta.summaryDelta.open)}</strong></span>
            <span>In Progress: <strong>{formatSignedDelta(queueHandoffDelta.summaryDelta.inProgress)}</strong></span>
            <span>At Risk: <strong>{formatSignedDelta(queueHandoffDelta.summaryDelta.atRisk)}</strong></span>
            <span>Unassigned: <strong>{formatSignedDelta(queueHandoffDelta.summaryDelta.unassigned)}</strong></span>
          </div>
          <Button size="small" variant="secondary" onClick={handleCopyHandoffSnapshot}>
            Copy Handoff Snapshot
          </Button>
          {handoffStatus && <p className="queue-handoff-status">{handoffStatus}</p>}
          {queueHandoffDelta.ownerDelta.length > 0 && (
            <ul>
              {queueHandoffDelta.ownerDelta.map((entry) => (
                <li key={entry.owner}>
                  {entry.owner}: workload {formatSignedDelta(entry.workloadDelta)}
                  {entry.atRiskDelta !== 0 ? ` · at-risk ${formatSignedDelta(entry.atRiskDelta)}` : ''}
                </li>
              ))}
            </ul>
          )}
          {queueHandoffSnapshot.topAtRisk.length > 0 && (
            <ul>
              {queueHandoffSnapshot.topAtRisk.slice(0, 3).map((riskItem) => (
                <li key={riskItem.draftId}>
                  {riskItem.ticketLabel} · {riskItem.priority} · due {formatQueueTimestamp(riskItem.slaDueAt)}
                </li>
              ))}
            </ul>
          )}
        </div>
      </section>

      <section className="queue-filters" aria-label="Queue filters">
        {QUEUE_VIEWS.map((view) => (
          <Button
            key={view.id}
            variant={queueView === view.id ? 'primary' : 'ghost'}
            size="small"
            onClick={() => {
              setQueueView(view.id);
              void logEvent('queue_view_changed', { queue_view: view.id, operator: operatorName });
            }}
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
        <p className="queue-shortcuts">
          Shortcuts: <kbd>J</kbd>/<kbd>K</kbd> move · <kbd>C</kbd> claim · <kbd>X</kbd> resolve · <kbd>O</kbd> reopen · <kbd>Enter</kbd> open
        </p>
        {loading && <p className="queue-loading">Loading queue...</p>}
        {!loading && filteredItems.length === 0 && (
          <p className="queue-empty">No queue items for this view.</p>
        )}

        {!loading && filteredItems.length > 0 && (
          <ul data-testid="queue-items-list" tabIndex={0}>
            {filteredItems.map((item, index) => (
              <li
                key={item.draft.id}
                className={`queue-item ${selectedIndex === index ? 'queue-item--selected' : ''}`}
                data-priority={item.meta.priority}
                data-selected={selectedIndex === index ? 'true' : 'false'}
              >
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
                  <Button
                    size="small"
                    variant="secondary"
                    onClick={() => {
                      onLoadDraft(item.draft);
                      void logEvent('queue_item_opened', {
                        draft_id: item.draft.id,
                        operator: operatorName,
                        entrypoint: 'button',
                      });
                    }}
                  >
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
