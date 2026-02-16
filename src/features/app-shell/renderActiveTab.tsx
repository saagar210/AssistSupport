import { ErrorBoundary } from '../../components/shared/ErrorBoundary';
import { type DraftTabHandle } from '../../components/Draft/DraftTab';
import { WorkspacePage } from '../workspace';
import { InboxPage } from '../inbox';
import { SourcesPage } from '../sources';
import { IngestPage } from '../ingest';
import { KnowledgePage } from '../knowledge';
import { AnalyticsPage } from '../analytics';
import { PilotPage } from '../pilot';
import { SearchPage } from '../search';
import { SettingsPage } from '../settings';
import { OpsPage } from '../ops';
import type { SavedDraft } from '../../types';
import type { TabId } from './types';
import type { RefObject } from 'react';
import type { RevampFlags } from '../revamp';
import type { QueueView } from '../inbox/queueModel';

export interface RenderActiveTabProps {
  activeTab: TabId;
  draftRef: RefObject<DraftTabHandle | null>;
  sourceSearchQuery: string | null;
  pendingQueueView: QueueView | null;
  onSearchQueryConsumed: () => void;
  onQueueViewConsumed: () => void;
  onNavigateToSource: (searchQuery: string) => void;
  onNavigateToQueue: (queueView: QueueView) => void;
  onLoadDraft: (draft: SavedDraft) => void;
  revampFlags: RevampFlags;
}

export function renderActiveTab({
  activeTab,
  draftRef,
  sourceSearchQuery,
  pendingQueueView,
  onSearchQueryConsumed,
  onQueueViewConsumed,
  onNavigateToSource,
  onNavigateToQueue,
  onLoadDraft,
  revampFlags,
}: RenderActiveTabProps) {
  switch (activeTab) {
    case 'draft':
      return (
        <ErrorBoundary fallbackTitle="Draft tab encountered an error">
          <WorkspacePage
            ref={draftRef}
            onNavigateToSource={onNavigateToSource}
            onNavigateToQueue={onNavigateToQueue}
            revampModeEnabled={revampFlags.ASSISTSUPPORT_REVAMP_WORKSPACE}
            appShellRevampEnabled={revampFlags.ASSISTSUPPORT_REVAMP_APP_SHELL}
          />
        </ErrorBoundary>
      );
    case 'followups':
      return (
        <ErrorBoundary fallbackTitle="Follow-ups tab encountered an error">
          <InboxPage
            onLoadDraft={onLoadDraft}
            queueFirstModeEnabled={revampFlags.ASSISTSUPPORT_REVAMP_INBOX}
            initialQueueView={pendingQueueView}
            onQueueViewConsumed={onQueueViewConsumed}
          />
        </ErrorBoundary>
      );
    case 'sources':
      return (
        <ErrorBoundary fallbackTitle="Sources tab encountered an error">
          <SourcesPage
            initialSearchQuery={sourceSearchQuery}
            onSearchQueryConsumed={onSearchQueryConsumed}
          />
        </ErrorBoundary>
      );
    case 'ingest':
      return (
        <ErrorBoundary fallbackTitle="Ingest tab encountered an error">
          <IngestPage />
        </ErrorBoundary>
      );
    case 'knowledge':
      return (
        <ErrorBoundary fallbackTitle="Knowledge tab encountered an error">
          <KnowledgePage />
        </ErrorBoundary>
      );
    case 'analytics':
      return (
        <ErrorBoundary fallbackTitle="Analytics tab encountered an error">
          <AnalyticsPage />
        </ErrorBoundary>
      );
    case 'pilot':
      return (
        <ErrorBoundary fallbackTitle="Pilot tab encountered an error">
          <PilotPage />
        </ErrorBoundary>
      );
    case 'search':
      return (
        <ErrorBoundary fallbackTitle="Search tab encountered an error">
          <SearchPage />
        </ErrorBoundary>
      );
    case 'settings':
      return (
        <ErrorBoundary fallbackTitle="Settings tab encountered an error">
          <SettingsPage />
        </ErrorBoundary>
      );
    case 'ops':
      return (
        <ErrorBoundary fallbackTitle="Operations tab encountered an error">
          <OpsPage />
        </ErrorBoundary>
      );
  }
}
