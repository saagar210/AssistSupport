import { useEffect, useMemo, useState } from 'react';
import { Button } from '../../components/shared/Button';
import { useAnalytics } from '../../hooks/useAnalytics';
import { useDrafts } from '../../hooks/useDrafts';
import {
  buildQueueHandoffSnapshot,
  buildQueueItems,
  formatQueueTimestamp,
  loadQueueMeta,
  summarizeQueue,
  type QueueMetaMap,
  type QueueView,
} from '../inbox/queueModel';

interface WorkspaceQueueContextProps {
  onNavigateToQueue?: (queueView: QueueView) => void;
}

export function WorkspaceQueueContext({ onNavigateToQueue }: WorkspaceQueueContextProps) {
  const { logEvent } = useAnalytics();
  const { drafts, loading, loadDrafts } = useDrafts();
  const [queueMetaMap, setQueueMetaMap] = useState<QueueMetaMap>(() => loadQueueMeta());

  useEffect(() => {
    loadDrafts(100);
  }, [loadDrafts]);

  useEffect(() => {
    const timer = window.setInterval(() => {
      setQueueMetaMap(loadQueueMeta());
    }, 15000);

    return () => window.clearInterval(timer);
  }, []);

  const queueItems = useMemo(() => buildQueueItems(drafts, queueMetaMap), [drafts, queueMetaMap]);
  const queueSummary = useMemo(() => summarizeQueue(queueItems), [queueItems]);
  const handoffSnapshot = useMemo(() => buildQueueHandoffSnapshot(queueItems), [queueItems]);

  return (
    <section className="workspace-queue-context" aria-label="Queue context">
      <div className="workspace-queue-context__header">
        <h2>Live queue context</h2>
        <Button
          size="small"
          variant="ghost"
          onClick={() => {
            loadDrafts(100);
            void logEvent('workspace_queue_context_refreshed');
          }}
        >
          Refresh
        </Button>
      </div>

      <div className="workspace-queue-context__stats">
        <div>
          <span>Open Queue</span>
          <strong>{Math.max(queueSummary.total - queueSummary.resolved, 0)}</strong>
        </div>
        <div>
          <span>At Risk</span>
          <strong>{queueSummary.atRisk}</strong>
        </div>
        <div>
          <span>Unassigned</span>
          <strong>{queueSummary.unassigned}</strong>
        </div>
      </div>

      {loading ? (
        <p className="workspace-queue-context__hint">Loading queue context...</p>
      ) : handoffSnapshot.topAtRisk.length > 0 ? (
        <div className="workspace-queue-context__risk">
          <h3>Top At-Risk Tickets</h3>
          <ul>
            {handoffSnapshot.topAtRisk.slice(0, 3).map((riskItem) => (
              <li key={riskItem.draftId}>
                <strong>{riskItem.ticketLabel}</strong> · {riskItem.priority} · due {formatQueueTimestamp(riskItem.slaDueAt)}
              </li>
            ))}
          </ul>
        </div>
      ) : (
        <p className="workspace-queue-context__hint">No at-risk tickets right now.</p>
      )}

      <div className="workspace-queue-context__footer">
        Snapshot updated {formatQueueTimestamp(handoffSnapshot.generatedAt)}
      </div>
      {onNavigateToQueue && (
        <div className="workspace-queue-context__quick-actions">
          <Button
            size="small"
            variant="secondary"
            onClick={() => {
              onNavigateToQueue('at_risk');
              void logEvent('workspace_queue_quick_action_used', { queue_view: 'at_risk' });
            }}
          >
            Open At-Risk Queue
          </Button>
          <Button
            size="small"
            variant="ghost"
            onClick={() => {
              onNavigateToQueue('unassigned');
              void logEvent('workspace_queue_quick_action_used', { queue_view: 'unassigned' });
            }}
          >
            Open Unassigned Queue
          </Button>
          <Button
            size="small"
            variant="ghost"
            onClick={() => {
              onNavigateToQueue('in_progress');
              void logEvent('workspace_queue_quick_action_used', { queue_view: 'in_progress' });
            }}
          >
            Open In-Progress Queue
          </Button>
        </div>
      )}
    </section>
  );
}
