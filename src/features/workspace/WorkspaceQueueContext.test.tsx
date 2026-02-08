import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { WorkspaceQueueContext } from './WorkspaceQueueContext';

vi.mock('../../hooks/useDrafts', () => ({
  useDrafts: () => ({
    drafts: [
      {
        id: 'draft-1',
        input_text: 'sev1 outage',
        summary_text: 'VPN outage',
        diagnosis_json: null,
        response_text: null,
        ticket_id: 'INC-9001',
        kb_sources_json: null,
        created_at: '2020-01-01T00:00:00Z',
        updated_at: '2020-01-01T00:00:00Z',
        is_autosave: false,
      },
      {
        id: 'draft-2',
        input_text: 'blocked laptop setup',
        summary_text: 'Setup blocked',
        diagnosis_json: null,
        response_text: null,
        ticket_id: 'INC-9002',
        kb_sources_json: null,
        created_at: '2020-01-01T00:00:00Z',
        updated_at: '2020-01-01T00:00:00Z',
        is_autosave: false,
      },
    ],
    loading: false,
    loadDrafts: vi.fn(),
  }),
}));

vi.mock('../../hooks/useAnalytics', () => ({
  useAnalytics: () => ({
    logEvent: vi.fn(),
  }),
}));

describe('WorkspaceQueueContext', () => {
  it('renders live queue metrics and at-risk tickets', () => {
    const onNavigateToQueue = vi.fn();
    render(<WorkspaceQueueContext onNavigateToQueue={onNavigateToQueue} />);

    expect(screen.getByText(/Live queue context/i)).toBeInTheDocument();
    expect(screen.getByText(/Open Queue/i)).toBeInTheDocument();
    expect(screen.getByText(/At Risk/i)).toBeInTheDocument();
    expect(screen.getByText(/INC-9001/i)).toBeInTheDocument();

    fireEvent.click(screen.getByRole('button', { name: /Open At-Risk Queue/i }));
    expect(onNavigateToQueue).toHaveBeenCalledWith('at_risk');
  });
});
