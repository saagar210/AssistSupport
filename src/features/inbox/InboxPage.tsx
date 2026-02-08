import { FollowUpsTab } from '../../components/FollowUps/FollowUpsTab';
import type { SavedDraft } from '../../types';
import { QueueFirstInboxPage } from './QueueFirstInboxPage';

interface InboxPageProps {
  onLoadDraft: (draft: SavedDraft) => void;
  queueFirstModeEnabled?: boolean;
}

export function InboxPage({ onLoadDraft, queueFirstModeEnabled = false }: InboxPageProps) {
  if (queueFirstModeEnabled) {
    return <QueueFirstInboxPage onLoadDraft={onLoadDraft} />;
  }

  return <FollowUpsTab onLoadDraft={onLoadDraft} />;
}
