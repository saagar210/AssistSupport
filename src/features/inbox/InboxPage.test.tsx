import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import type { SavedDraft } from '../../types';
import { InboxPage } from './InboxPage';

vi.mock('../../components/FollowUps/FollowUpsTab', () => ({
  FollowUpsTab: () => <div data-testid="legacy-followups-tab">Legacy FollowUps</div>,
}));

const sampleDraft = {
  id: 'draft-1',
  input_text: 'test',
  summary_text: null,
  diagnosis_json: null,
  response_text: null,
  ticket_id: null,
  kb_sources_json: null,
  created_at: '2026-02-08T00:00:00Z',
  updated_at: '2026-02-08T00:00:00Z',
  is_autosave: false,
} satisfies SavedDraft;

describe('InboxPage', () => {
  it('renders legacy follow-ups tab by default', () => {
    render(<InboxPage onLoadDraft={() => undefined} />);
    expect(screen.getByTestId('legacy-followups-tab')).toBeInTheDocument();
  });

  it('renders queue-first shell when queue-first mode is enabled', () => {
    render(
      <InboxPage
        onLoadDraft={(draft) => {
          expect(draft).toEqual(sampleDraft);
        }}
        queueFirstModeEnabled
      />,
    );

    expect(screen.getByTestId('queue-first-inbox')).toBeInTheDocument();
    expect(screen.getByText(/Queue-first inbox mode/i)).toBeInTheDocument();
  });
});
