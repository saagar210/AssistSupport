import { forwardRef } from 'react';
import { DraftTab, type DraftTabHandle } from '../../components/Draft/DraftTab';
import { WorkspaceRevampPage } from './WorkspaceRevampPage';

interface WorkspacePageProps {
  onNavigateToSource: (searchQuery: string) => void;
  revampModeEnabled?: boolean;
}

export const WorkspacePage = forwardRef<DraftTabHandle, WorkspacePageProps>(
  function WorkspacePage({ onNavigateToSource, revampModeEnabled = false }, ref) {
    if (revampModeEnabled) {
      return <WorkspaceRevampPage ref={ref} onNavigateToSource={onNavigateToSource} />;
    }

    return <DraftTab ref={ref} onNavigateToSource={onNavigateToSource} />;
  }
);
