import { forwardRef } from 'react';
import { DraftTab, type DraftTabHandle } from '../../components/Draft/DraftTab';
import { WorkspaceQueueContext } from './WorkspaceQueueContext';
import './WorkspaceRevampPage.css';

interface WorkspaceRevampPageProps {
  onNavigateToSource: (searchQuery: string) => void;
}

export const WorkspaceRevampPage = forwardRef<DraftTabHandle, WorkspaceRevampPageProps>(
  function WorkspaceRevampPage({ onNavigateToSource }, ref) {
    return (
      <div className="workspace-revamp" data-testid="workspace-revamp-shell">
        <section className="workspace-revamp__rail" aria-label="Draft workflow guidance">
          <WorkspaceQueueContext />
          <div className="workspace-revamp__playbook">
            <h3>Response playbook</h3>
            <ol>
              <li>Capture the customer issue in plain language.</li>
              <li>Validate policy and approval requirements.</li>
              <li>Generate and edit response with cited context.</li>
              <li>Save to follow-ups for handoff continuity.</li>
            </ol>
          </div>
        </section>
        <section className="workspace-revamp__main">
          <DraftTab ref={ref} onNavigateToSource={onNavigateToSource} />
        </section>
      </div>
    );
  },
);
