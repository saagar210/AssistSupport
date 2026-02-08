import type { SavedDraft } from '../../types';
import { FollowUpsTab } from '../../components/FollowUps/FollowUpsTab';
import './QueueFirstInboxPage.css';

interface QueueFirstInboxPageProps {
  onLoadDraft: (draft: SavedDraft) => void;
}

export function QueueFirstInboxPage({ onLoadDraft }: QueueFirstInboxPageProps) {
  return (
    <div className="queue-inbox-page" data-testid="queue-first-inbox">
      <section className="queue-inbox-banner" aria-live="polite">
        <h2>Queue-first inbox mode</h2>
        <p>
          Revamp inbox mode is enabled. Follow-up history remains fully available while
          queue-centric workflows are introduced incrementally.
        </p>
      </section>
      <FollowUpsTab onLoadDraft={onLoadDraft} />
    </div>
  );
}
