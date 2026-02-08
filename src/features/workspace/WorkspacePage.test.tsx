import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { WorkspacePage } from './WorkspacePage';

vi.mock('../../components/Draft/DraftTab', () => ({
  DraftTab: ({ onNavigateToSource }: { onNavigateToSource: (query: string) => void }) => (
    <button type="button" data-testid="legacy-draft-tab" onClick={() => onNavigateToSource('vpn policy')}>
      Legacy Draft
    </button>
  ),
}));

describe('WorkspacePage', () => {
  it('renders legacy draft tab by default', () => {
    render(<WorkspacePage onNavigateToSource={() => undefined} />);
    expect(screen.getByTestId('legacy-draft-tab')).toBeInTheDocument();
  });

  it('renders revamp shell when workspace revamp mode is enabled', () => {
    render(<WorkspacePage onNavigateToSource={() => undefined} revampModeEnabled />);
    expect(screen.getByTestId('workspace-revamp-shell')).toBeInTheDocument();
    expect(screen.getByText(/Draft workflow/i)).toBeInTheDocument();
  });
});
