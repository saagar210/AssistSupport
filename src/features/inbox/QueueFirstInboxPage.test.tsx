import { fireEvent, render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { SavedDraft } from '../../types';
import { QueueFirstInboxPage } from './QueueFirstInboxPage';

const drafts: SavedDraft[] = [
  {
    id: 'draft-1',
    input_text: 'sev1 outage in production',
    summary_text: 'VPN outage for multiple users',
    diagnosis_json: null,
    response_text: null,
    ticket_id: 'INC-2001',
    kb_sources_json: null,
    created_at: '2026-02-08T00:00:00Z',
    updated_at: '2026-02-08T00:00:00Z',
    is_autosave: false,
  },
  {
    id: 'draft-2',
    input_text: 'blocked laptop setup',
    summary_text: 'Setup blocked by security policy',
    diagnosis_json: null,
    response_text: null,
    ticket_id: 'INC-2002',
    kb_sources_json: null,
    created_at: '2026-02-08T00:00:00Z',
    updated_at: '2026-02-08T01:00:00Z',
    is_autosave: false,
  },
];

vi.mock('../../hooks/useDrafts', () => ({
  useDrafts: () => ({
    drafts,
    loading: false,
    loadDrafts: vi.fn(),
  }),
}));

vi.mock('../../hooks/useAnalytics', () => ({
  useAnalytics: () => ({
    logEvent: vi.fn(),
  }),
}));

vi.mock('../../components/FollowUps/FollowUpsTab', () => ({
  FollowUpsTab: () => <div data-testid="queue-history-tab">History</div>,
}));

describe('QueueFirstInboxPage', () => {
  beforeEach(() => {
    window.localStorage.clear();
  });

  it('supports keyboard triage and draft open actions', () => {
    const onLoadDraft = vi.fn();
    render(<QueueFirstInboxPage onLoadDraft={onLoadDraft} />);

    const list = screen.getByTestId('queue-items-list');
    list.focus();

    fireEvent.keyDown(list, { key: 'c' });
    expect(screen.getByText(/Owner: current-operator/i)).toBeInTheDocument();

    fireEvent.keyDown(list, { key: 'x' });
    expect(screen.getByRole('button', { name: /Reopen/i })).toBeInTheDocument();

    fireEvent.keyDown(list, { key: 'o' });
    expect(screen.getByRole('button', { name: /^Resolve$/i })).toBeInTheDocument();

    fireEvent.keyDown(list, { key: 'Enter' });
    expect(onLoadDraft).toHaveBeenCalledWith(expect.objectContaining({ id: 'draft-1' }));
  });

  it('consumes initial queue view deep-link', () => {
    const onQueueViewConsumed = vi.fn();
    render(
      <QueueFirstInboxPage
        onLoadDraft={() => undefined}
        initialQueueView="in_progress"
        onQueueViewConsumed={onQueueViewConsumed}
      />,
    );

    expect(onQueueViewConsumed).toHaveBeenCalledTimes(1);
    expect(screen.getByRole('button', { name: /In Progress/i })).toHaveClass('btn-primary');
  });
});
