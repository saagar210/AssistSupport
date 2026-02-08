import { forwardRef } from 'react';
import { DraftTab, type DraftTabHandle } from '../../components/Draft/DraftTab';

interface WorkspacePageProps {
  onNavigateToSource: (searchQuery: string) => void;
}

export const WorkspacePage = forwardRef<DraftTabHandle, WorkspacePageProps>(
  function WorkspacePage({ onNavigateToSource }, ref) {
    return <DraftTab ref={ref} onNavigateToSource={onNavigateToSource} />;
  }
);
