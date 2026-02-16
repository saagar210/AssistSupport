import { FollowUpsTab } from '../../components/FollowUps/FollowUpsTab';
import type { SavedDraft } from '../../types';
import type { QueueView } from './queueModel';
import { QueueCommandCenterPage } from '../revamp';

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
      <QueueCommandCenterPage
        onLoadDraft={onLoadDraft}
        initialQueueView={initialQueueView}
        onQueueViewConsumed={onQueueViewConsumed}
      />
    );
  }

  return <FollowUpsTab onLoadDraft={onLoadDraft} />;
}
