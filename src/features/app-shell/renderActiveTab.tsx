import { ErrorBoundary } from '../../components/shared/ErrorBoundary';
import { type DraftTabHandle } from '../../components/Draft/DraftTab';
import { lazy, Suspense } from 'react';
import { WorkspacePage } from '../workspace';
import type { SavedDraft } from '../../types';
import type { TabId } from './types';
import type { RefObject } from 'react';
import type { RevampFlags } from '../revamp';
import type { QueueView } from '../inbox/queueModel';

const InboxPage = lazy(() =>
  import('../inbox').then((module) => ({ default: module.InboxPage })),
);
const SourcesPage = lazy(() =>
  import('../sources').then((module) => ({ default: module.SourcesPage })),
);
const IngestPage = lazy(() =>
  import('../ingest').then((module) => ({ default: module.IngestPage })),
);
const KnowledgePage = lazy(() =>
  import('../knowledge').then((module) => ({ default: module.KnowledgePage })),
);
const AnalyticsPage = lazy(() =>
  import('../analytics').then((module) => ({ default: module.AnalyticsPage })),
);
const PilotPage = lazy(() =>
  import('../pilot').then((module) => ({ default: module.PilotPage })),
);
const SearchPage = lazy(() =>
  import('../search').then((module) => ({ default: module.SearchPage })),
);
const SettingsPage = lazy(() =>
  import('../settings').then((module) => ({ default: module.SettingsPage })),
);
const OpsPage = lazy(() => import('../ops').then((module) => ({ default: module.OpsPage })));

function TabLoadFallback({ tabLabel }: { tabLabel: string }) {
  return (
    <section aria-label={`${tabLabel} loading`} className="tab-loading-fallback">
      Loading {tabLabel}...
    </section>
  );
}

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
          <Suspense fallback={<TabLoadFallback tabLabel="Follow-ups" />}>
            <InboxPage
              onLoadDraft={onLoadDraft}
              queueFirstModeEnabled={revampFlags.ASSISTSUPPORT_REVAMP_INBOX}
              initialQueueView={pendingQueueView}
              onQueueViewConsumed={onQueueViewConsumed}
            />
          </Suspense>
        </ErrorBoundary>
      );
    case 'sources':
      return (
        <ErrorBoundary fallbackTitle="Sources tab encountered an error">
          <Suspense fallback={<TabLoadFallback tabLabel="Sources" />}>
            <SourcesPage
              initialSearchQuery={sourceSearchQuery}
              onSearchQueryConsumed={onSearchQueryConsumed}
            />
          </Suspense>
        </ErrorBoundary>
      );
    case 'ingest':
      return (
        <ErrorBoundary fallbackTitle="Ingest tab encountered an error">
          <Suspense fallback={<TabLoadFallback tabLabel="Ingest" />}>
            <IngestPage />
          </Suspense>
        </ErrorBoundary>
      );
    case 'knowledge':
      return (
        <ErrorBoundary fallbackTitle="Knowledge tab encountered an error">
          <Suspense fallback={<TabLoadFallback tabLabel="Knowledge" />}>
            <KnowledgePage />
          </Suspense>
        </ErrorBoundary>
      );
    case 'analytics':
      return (
        <ErrorBoundary fallbackTitle="Analytics tab encountered an error">
          <Suspense fallback={<TabLoadFallback tabLabel="Analytics" />}>
            <AnalyticsPage />
          </Suspense>
        </ErrorBoundary>
      );
    case 'pilot':
      return (
        <ErrorBoundary fallbackTitle="Pilot tab encountered an error">
          <Suspense fallback={<TabLoadFallback tabLabel="Pilot" />}>
            <PilotPage />
          </Suspense>
        </ErrorBoundary>
      );
    case 'search':
      return (
        <ErrorBoundary fallbackTitle="Search tab encountered an error">
          <Suspense fallback={<TabLoadFallback tabLabel="Search" />}>
            <SearchPage />
          </Suspense>
        </ErrorBoundary>
      );
    case 'settings':
      return (
        <ErrorBoundary fallbackTitle="Settings tab encountered an error">
          <Suspense fallback={<TabLoadFallback tabLabel="Settings" />}>
            <SettingsPage />
          </Suspense>
        </ErrorBoundary>
      );
    case 'ops':
      return (
        <ErrorBoundary fallbackTitle="Operations tab encountered an error">
          <Suspense fallback={<TabLoadFallback tabLabel="Operations" />}>
            <OpsPage />
          </Suspense>
        </ErrorBoundary>
      );
  }
}
