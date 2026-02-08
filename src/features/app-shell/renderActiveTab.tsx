import { ErrorBoundary } from '../../components/shared/ErrorBoundary';
import { type DraftTabHandle } from '../../components/Draft/DraftTab';
import { SourcesTab } from '../../components/Sources/SourcesTab';
import { IngestTab } from '../../components/Ingest/IngestTab';
import { KnowledgeBrowser } from '../../components/Knowledge';
import { AnalyticsTab } from '../../components/Analytics/AnalyticsTab';
import { PilotTab } from '../../components/Pilot';
import { HybridSearchTab } from '../../components/Search';
import { SettingsTab } from '../../components/Settings/SettingsTab';
import { OpsTab } from '../../components/Ops';
import { WorkspacePage } from '../workspace';
import { InboxPage } from '../inbox';
import type { SavedDraft } from '../../types';
import type { TabId } from './types';
import type { RefObject } from 'react';
import type { RevampFlags } from '../revamp';

export interface RenderActiveTabProps {
  activeTab: TabId;
  draftRef: RefObject<DraftTabHandle | null>;
  sourceSearchQuery: string | null;
  onSearchQueryConsumed: () => void;
  onNavigateToSource: (searchQuery: string) => void;
  onLoadDraft: (draft: SavedDraft) => void;
  revampFlags: RevampFlags;
}

export function renderActiveTab({
  activeTab,
  draftRef,
  sourceSearchQuery,
  onSearchQueryConsumed,
  onNavigateToSource,
  onLoadDraft,
  revampFlags,
}: RenderActiveTabProps) {
  switch (activeTab) {
    case 'draft':
      return (
        <ErrorBoundary fallbackTitle="Draft tab encountered an error">
          <WorkspacePage ref={draftRef} onNavigateToSource={onNavigateToSource} />
        </ErrorBoundary>
      );
    case 'followups':
      return (
        <ErrorBoundary fallbackTitle="Follow-ups tab encountered an error">
          <InboxPage
            onLoadDraft={onLoadDraft}
            queueFirstModeEnabled={revampFlags.ASSISTSUPPORT_REVAMP_INBOX}
          />
        </ErrorBoundary>
      );
    case 'sources':
      return (
        <ErrorBoundary fallbackTitle="Sources tab encountered an error">
          <SourcesTab
            initialSearchQuery={sourceSearchQuery}
            onSearchQueryConsumed={onSearchQueryConsumed}
          />
        </ErrorBoundary>
      );
    case 'ingest':
      return (
        <ErrorBoundary fallbackTitle="Ingest tab encountered an error">
          <IngestTab />
        </ErrorBoundary>
      );
    case 'knowledge':
      return (
        <ErrorBoundary fallbackTitle="Knowledge tab encountered an error">
          <KnowledgeBrowser />
        </ErrorBoundary>
      );
    case 'analytics':
      return (
        <ErrorBoundary fallbackTitle="Analytics tab encountered an error">
          <AnalyticsTab />
        </ErrorBoundary>
      );
    case 'pilot':
      return (
        <ErrorBoundary fallbackTitle="Pilot tab encountered an error">
          <PilotTab />
        </ErrorBoundary>
      );
    case 'search':
      return (
        <ErrorBoundary fallbackTitle="Search tab encountered an error">
          <HybridSearchTab />
        </ErrorBoundary>
      );
    case 'settings':
      return (
        <ErrorBoundary fallbackTitle="Settings tab encountered an error">
          <SettingsTab />
        </ErrorBoundary>
      );
    case 'ops':
      return (
        <ErrorBoundary fallbackTitle="Operations tab encountered an error">
          <OpsTab />
        </ErrorBoundary>
      );
  }
}
