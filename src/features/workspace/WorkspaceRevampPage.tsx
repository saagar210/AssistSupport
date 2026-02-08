import { forwardRef } from 'react';
import { DraftTab, type DraftTabHandle } from '../../components/Draft/DraftTab';
import './WorkspaceRevampPage.css';

interface WorkspaceRevampPageProps {
  onNavigateToSource: (searchQuery: string) => void;
}

export const WorkspaceRevampPage = forwardRef<DraftTabHandle, WorkspaceRevampPageProps>(
  function WorkspaceRevampPage({ onNavigateToSource }, ref) {
    return (
      <div className="workspace-revamp" data-testid="workspace-revamp-shell">
        <section className="workspace-revamp__rail" aria-label="Draft workflow guidance">
          <h2>Draft workflow</h2>
          <ol>
            <li>Capture the customer issue in plain language.</li>
            <li>Review policy and approval requirements before responding.</li>
            <li>Generate and edit response with supporting context.</li>
            <li>Save to follow-ups for handoff continuity.</li>
          </ol>
        </section>
        <section className="workspace-revamp__main">
          <DraftTab ref={ref} onNavigateToSource={onNavigateToSource} />
        </section>
      </div>
    );
  },
);
