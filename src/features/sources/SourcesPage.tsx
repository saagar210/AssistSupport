import { SourcesTab } from '../../components/Sources/SourcesTab';

interface SourcesPageProps {
  initialSearchQuery: string | null;
  onSearchQueryConsumed: () => void;
}

export function SourcesPage({ initialSearchQuery, onSearchQueryConsumed }: SourcesPageProps) {
  return (
    <SourcesTab
      initialSearchQuery={initialSearchQuery}
      onSearchQueryConsumed={onSearchQueryConsumed}
    />
  );
}
