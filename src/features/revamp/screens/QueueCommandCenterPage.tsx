import { useCallback, useEffect, useMemo, useState } from 'react';
import { Icon } from '../../../components/shared/Icon';
import { useAnalytics } from '../../../hooks/useAnalytics';
import { useDrafts } from '../../../hooks/useDrafts';
import type { SavedDraft } from '../../../types';
import {
  buildQueueItems,
  filterQueueItems,
  formatQueueTimestamp,
  loadQueueMeta,
  persistQueueMeta,
  summarizeQueue,
  type QueueItem,
  type QueueMetaMap,
  type QueuePriority,
  type QueueState,
  type QueueView,
} from '../../inbox/queueModel';
import { AsButton, Badge, EmptyState, Panel, Skeleton } from '../ui';
import '../../../styles/revamp/index.css';
import './queueCommandCenter.css';

interface QueueCommandCenterPageProps {
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
  if (value.length <= limit) return value;
  return `${value.slice(0, limit)}...`;
}

function loadOperatorName(): string {
  if (typeof window === 'undefined') return 'current-operator';
  try {
    return localStorage.getItem(QUEUE_OPERATOR_STORAGE_KEY) || 'current-operator';
  } catch {
    return 'current-operator';
  }
}

function bandLabel(item: QueueItem): 'At Risk' | 'Unassigned' | 'In Progress' | 'Open' | 'Resolved' {
  if (item.meta.state === 'resolved') return 'Resolved';
  if (item.isAtRisk) return 'At Risk';
  if (item.meta.owner === 'unassigned') return 'Unassigned';
  if (item.meta.state === 'in_progress') return 'In Progress';
  return 'Open';
}

export function QueueCommandCenterPage({
  onLoadDraft,
  initialQueueView = null,
  onQueueViewConsumed,
}: QueueCommandCenterPageProps) {
  const { logEvent } = useAnalytics();
  const { drafts, loading, error: draftsError, loadDrafts } = useDrafts();
  const [queueMetaMap, setQueueMetaMap] = useState<QueueMetaMap>(() => loadQueueMeta());
  const [queueView, setQueueView] = useState<QueueView>('all');
  const [searchQuery, setSearchQuery] = useState('');
  const [operatorName, setOperatorName] = useState(loadOperatorName);
  const [selectedIndex, setSelectedIndex] = useState(0);

  useEffect(() => {
    loadDrafts(100);
  }, [loadDrafts]);

  useEffect(() => {
    if (!initialQueueView) return;
    setQueueView(initialQueueView);
    onQueueViewConsumed?.();
  }, [initialQueueView, onQueueViewConsumed]);

  useEffect(() => {
    try {
      localStorage.setItem(QUEUE_OPERATOR_STORAGE_KEY, operatorName);
    } catch {
      // Keep queue workflows usable even if storage is restricted.
    }
  }, [operatorName]);

  const queueItems = useMemo(() => buildQueueItems(drafts, queueMetaMap), [drafts, queueMetaMap]);
  const queueSummary = useMemo(() => summarizeQueue(queueItems), [queueItems]);

  const filteredItems = useMemo(() => {
    const scoped = filterQueueItems(queueItems, queueView);
    const q = searchQuery.trim().toLowerCase();
    if (!q) return scoped;
    return scoped.filter((item) => {
      const ticket = item.draft.ticket_id?.toLowerCase() ?? '';
      const summary = item.draft.summary_text?.toLowerCase() ?? '';
      const input = item.draft.input_text.toLowerCase();
      return ticket.includes(q) || summary.includes(q) || input.includes(q);
    });
  }, [queueItems, queueView, searchQuery]);

  useEffect(() => {
    setSelectedIndex((prev) => {
      if (filteredItems.length === 0) return 0;
      return Math.max(0, Math.min(prev, filteredItems.length - 1));
    });
  }, [filteredItems]);

  const updateQueueMeta = useCallback(
    (draftId: string, updates: Partial<{ owner: string; state: QueueState; priority: QueuePriority }>) => {
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
    },
    [],
  );

  const handleClaim = useCallback(
    (draftId: string) => {
      updateQueueMeta(draftId, { owner: operatorName, state: 'in_progress' });
      void logEvent('queue_item_claimed', { draft_id: draftId, operator: operatorName });
    },
    [logEvent, operatorName, updateQueueMeta],
  );

  const handleResolve = useCallback(
    (draftId: string) => {
      updateQueueMeta(draftId, { state: 'resolved' });
      void logEvent('queue_item_resolved', { draft_id: draftId, operator: operatorName });
    },
    [logEvent, operatorName, updateQueueMeta],
  );

  const handleReopen = useCallback(
    (draftId: string) => {
      updateQueueMeta(draftId, { state: 'open' });
      void logEvent('queue_item_reopened', { draft_id: draftId, operator: operatorName });
    },
    [logEvent, operatorName, updateQueueMeta],
  );

  const handlePriorityChange = useCallback(
    (draftId: string, priority: QueuePriority) => {
      updateQueueMeta(draftId, { priority });
      void logEvent('queue_item_priority_changed', {
        draft_id: draftId,
        operator: operatorName,
        priority,
      });
    },
    [logEvent, operatorName, updateQueueMeta],
  );

  const currentItem = filteredItems[selectedIndex] ?? null;

  const withCurrentItem = useCallback(
    (handler: (item: QueueItem) => void) => {
      if (!currentItem) return;
      handler(currentItem);
    },
    [currentItem],
  );

  const handleQueueKeyDown = useCallback(
    (event: React.KeyboardEvent<HTMLElement>) => {
      const target = event.target as HTMLElement;
      const isInputElement = target.tagName === 'INPUT' || target.tagName === 'SELECT' || target.tagName === 'TEXTAREA';
      if (isInputElement) return;

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
    },
    [filteredItems.length, handleClaim, handleReopen, handleResolve, logEvent, onLoadDraft, operatorName, withCurrentItem],
  );

  const listActions = (
    <div className="as-queue__count" aria-label="Visible work item count">
      {loading ? 'Loading…' : `${filteredItems.length} shown`}
    </div>
  );

  const listContent = (
    <Panel
      title="Work Items"
      subtitle="Keyboard triage: J/K move · C claim · X resolve · O reopen · Enter open"
      actions={listActions}
    >
      {loading && <Skeleton lines={6} />}
      {!loading && draftsError && (
        <div className="as-queue__errorState">
          <EmptyState
            title="Queue unavailable"
            description="Drafts could not be loaded. Retry, or switch back to Draft to continue working offline."
            icon={<Icon name="alert-triangle" size={18} />}
          />
          <div className="as-queue__errorActions">
            <AsButton
              tone="primary"
              size="small"
              onClick={() => {
                loadDrafts(100);
              }}
            >
              Retry Load
            </AsButton>
          </div>
        </div>
      )}
      {!loading && filteredItems.length === 0 && (
        <EmptyState
          title="No work items in this view"
          description="Try switching views, clearing search, or creating a new draft from Intake."
          icon={<Icon name="list" size={18} />}
        />
      )}
      {!loading && filteredItems.length > 0 && (
        <ul
          data-testid="queue-items-list"
          tabIndex={0}
          role="listbox"
          aria-label="Work items"
          aria-activedescendant={currentItem ? `as-queue-item-${currentItem.draft.id}` : undefined}
          className="as-queue__items"
        >
          {filteredItems.map((item, index) => {
            const prev = filteredItems[index - 1] ?? null;
            const label = bandLabel(item);
            const prevLabel = prev ? bandLabel(prev) : null;
            const showSection = index === 0 || label !== prevLabel;
            const selected = selectedIndex === index;

            return (
              <li key={item.draft.id}>
                {showSection && <div className="as-queue__sectionLabel">{label}</div>}
                <div
                  id={`as-queue-item-${item.draft.id}`}
                  role="option"
                  aria-selected={selected}
                  className={['as-queue__item', selected ? 'is-selected' : ''].filter(Boolean).join(' ')}
                  data-selected={selected ? 'true' : 'false'}
                  onClick={() => setSelectedIndex(index)}
                >
                  <div className="as-queue__itemTop">
                    <div>
                      <div className="as-queue__ticket">{formatTicketLabel(item.draft)}</div>
                      <p className="as-queue__summary">
                        {truncate(item.draft.summary_text || item.draft.input_text, 140)}
                      </p>
                    </div>
                    <div className="as-queue__badges">
                      <Badge tone={item.isAtRisk ? 'bad' : item.meta.priority === 'urgent' ? 'warn' : 'neutral'}>
                        {item.meta.priority}
                      </Badge>
                      <Badge tone={item.meta.state === 'resolved' ? 'info' : item.meta.state === 'in_progress' ? 'good' : 'neutral'}>
                        {item.meta.state.replace('_', ' ')}
                      </Badge>
                      {item.isAtRisk && <Badge tone="bad">at risk</Badge>}
                    </div>
                  </div>

                  <div className="as-queue__metaRow">
                    <span>Owner: {item.meta.owner}</span>
                    <span>SLA due: {formatQueueTimestamp(item.slaDueAt)}</span>
                    <span>Updated: {formatQueueTimestamp(item.meta.updatedAt)}</span>
                  </div>

                  <div className="as-queue__actions">
                    <AsButton
                      tone="primary"
                      size="small"
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
                    </AsButton>
                    {item.meta.owner === 'unassigned' && item.meta.state !== 'resolved' && (
                      <AsButton size="small" onClick={() => handleClaim(item.draft.id)}>
                        Claim
                      </AsButton>
                    )}
                    {item.meta.state !== 'resolved' ? (
                      <AsButton tone="ghost" size="small" onClick={() => handleResolve(item.draft.id)}>
                        Resolve
                      </AsButton>
                    ) : (
                      <AsButton tone="ghost" size="small" onClick={() => handleReopen(item.draft.id)}>
                        Reopen
                      </AsButton>
                    )}
                  </div>
                </div>
              </li>
            );
          })}
        </ul>
      )}
    </Panel>
  );

  const detailContent = (
    <div className="as-queue__detail">
      <Panel
        title="Selected Item"
        subtitle={currentItem ? 'Preview + quick edits' : 'Select a work item to preview'}
      >
        {!currentItem ? (
          <EmptyState
            title="No selection"
            description="Use J/K or click an item, then Enter to open it in Draft."
            icon={<Icon name="search" size={18} />}
          />
        ) : (
          <>
            <h3 className="as-queue__detailTitle">{formatTicketLabel(currentItem.draft)}</h3>
            <p className="as-queue__detailText">
              {currentItem.draft.summary_text || truncate(currentItem.draft.input_text, 260)}
            </p>

            <div className="as-queue__detailGrid">
              <div>
                <div className="as-queue__label">Owner</div>
                <div>{currentItem.meta.owner}</div>
              </div>
              <div>
                <div className="as-queue__label">SLA due</div>
                <div>{formatQueueTimestamp(currentItem.slaDueAt)}</div>
              </div>
              <div>
                <div className="as-queue__label">Priority</div>
                <select
                  className="as-queue__select"
                  value={currentItem.meta.priority}
                  onChange={(event) => handlePriorityChange(currentItem.draft.id, event.target.value as QueuePriority)}
                >
                  <option value="low">low</option>
                  <option value="normal">normal</option>
                  <option value="high">high</option>
                  <option value="urgent">urgent</option>
                </select>
              </div>
            </div>

            <div className="as-queue__actions">
              <AsButton
                tone="primary"
                onClick={() => {
                  onLoadDraft(currentItem.draft);
                  void logEvent('queue_item_opened', {
                    draft_id: currentItem.draft.id,
                    operator: operatorName,
                    entrypoint: 'preview',
                  });
                }}
              >
                Open In Draft
              </AsButton>
              {currentItem.meta.owner === 'unassigned' && currentItem.meta.state !== 'resolved' && (
                <AsButton onClick={() => handleClaim(currentItem.draft.id)}>Claim</AsButton>
              )}
              {currentItem.meta.state !== 'resolved' ? (
                <AsButton tone="ghost" onClick={() => handleResolve(currentItem.draft.id)}>Resolve</AsButton>
              ) : (
                <AsButton tone="ghost" onClick={() => handleReopen(currentItem.draft.id)}>Reopen</AsButton>
              )}
            </div>
          </>
        )}
      </Panel>
    </div>
  );

  return (
    <div className="as-queue" data-testid="queue-first-inbox" onKeyDown={handleQueueKeyDown}>
      <div className="as-queue__header">
        <div>
          <h2 className="as-queue__title">Queue Command Center</h2>
          <p className="as-queue__subtitle">
            Local-only queue triage. Open drafts, claim ownership, and resolve work without losing context.
          </p>
        </div>
        <div className="as-queue__operator" aria-label="Queue operator settings">
          <div className="as-queue__label">Operator</div>
          <input
            className="as-queue__input"
            value={operatorName}
            onChange={(event) => setOperatorName(event.target.value || 'current-operator')}
            maxLength={64}
            aria-label="Operator name"
          />
        </div>
      </div>

      <div className="as-queue__metrics" aria-label="Queue summary metrics">
        <Panel>
          <div className="as-queue__metricKey">Total</div>
          <div className="as-queue__metricVal">{queueSummary.total}</div>
        </Panel>
        <Panel>
          <div className="as-queue__metricKey">Unassigned</div>
          <div className="as-queue__metricVal">{queueSummary.unassigned}</div>
        </Panel>
        <Panel>
          <div className="as-queue__metricKey">In Progress</div>
          <div className="as-queue__metricVal">{queueSummary.inProgress}</div>
        </Panel>
        <Panel>
          <div className="as-queue__metricKey">At Risk</div>
          <div className="as-queue__metricVal">{queueSummary.atRisk}</div>
        </Panel>
      </div>

      <div className="as-queue__controls" aria-label="Queue controls">
        <div className="as-queue__viewButtons" role="group" aria-label="Queue views">
          {QUEUE_VIEWS.map((view) => (
            <AsButton
              key={view.id}
              tone={queueView === view.id ? 'primary' : 'ghost'}
              size="small"
              onClick={() => {
                setQueueView(view.id);
                void logEvent('queue_view_changed', { queue_view: view.id, operator: operatorName });
              }}
            >
              {view.label}
            </AsButton>
          ))}
        </div>

        <input
          className={['as-queue__input', 'as-queue__search'].join(' ')}
          placeholder="Search queue..."
          value={searchQuery}
          onChange={(event) => setSearchQuery(event.target.value)}
          aria-label="Search queue"
        />
      </div>

      <div className="as-queue__grid" aria-label="Queue workspace">
        <div className="as-queue__list">{listContent}</div>
        {detailContent}
      </div>
    </div>
  );
}
