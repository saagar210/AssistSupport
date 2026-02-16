import { forwardRef } from 'react';
import { DraftTab, type DraftTabHandle } from '../../components/Draft/DraftTab';
import { WorkspaceRevampPage } from './WorkspaceRevampPage';
import type { QueueView } from '../inbox/queueModel';

interface WorkspacePageProps {
  onNavigateToSource: (searchQuery: string) => void;
  onNavigateToQueue?: (queueView: QueueView) => void;
  revampModeEnabled?: boolean;
  appShellRevampEnabled?: boolean;
}

export const WorkspacePage = forwardRef<DraftTabHandle, WorkspacePageProps>(
  function WorkspacePage(
    { onNavigateToSource, onNavigateToQueue, revampModeEnabled = false, appShellRevampEnabled = false },
    ref
  ) {
    if (revampModeEnabled) {
      return (
        <WorkspaceRevampPage
          ref={ref}
          onNavigateToSource={onNavigateToSource}
          onNavigateToQueue={onNavigateToQueue}
          appShellRevampEnabled={appShellRevampEnabled}
        />
      );
    }

    return <DraftTab ref={ref} onNavigateToSource={onNavigateToSource} />;
  }
);
