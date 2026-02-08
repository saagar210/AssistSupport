import { FollowUpsTab } from '../../components/FollowUps/FollowUpsTab';
import type { SavedDraft } from '../../types';

interface InboxPageProps {
  onLoadDraft: (draft: SavedDraft) => void;
}

export function InboxPage({ onLoadDraft }: InboxPageProps) {
  return <FollowUpsTab onLoadDraft={onLoadDraft} />;
}
