import { FollowUpsTab } from '../../components/FollowUps/FollowUpsTab';
import type { SavedDraft } from '../../types';
import { QueueFirstInboxPage } from './QueueFirstInboxPage';
import type { QueueView } from './queueModel';

interface InboxPageProps {
  onLoadDraft: (draft: SavedDraft) => void;
  queueFirstModeEnabled?: boolean;
  initialQueueView?: QueueView | null;
  onQueueViewConsumed?: () => void;
}

export function InboxPage({
  onLoadDraft,
  queueFirstModeEnabled = false,
  initialQueueView = null,
  onQueueViewConsumed,
}: InboxPageProps) {
  if (queueFirstModeEnabled) {
    return (
      <QueueFirstInboxPage
        onLoadDraft={onLoadDraft}
        initialQueueView={initialQueueView}
        onQueueViewConsumed={onQueueViewConsumed}
      />
    );
  }

  return <FollowUpsTab onLoadDraft={onLoadDraft} />;
}
