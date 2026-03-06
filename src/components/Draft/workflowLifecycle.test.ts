import { readFileSync } from 'node:fs';
import path from 'node:path';
import { describe, expect, it } from 'vitest';
import {
  createInitialDraftLifecycleState,
  DraftLifecycleEvent,
  reduceDraftLifecycle,
  replayDraftLifecycle,
} from './workflowLifecycle';

function loadFixture(name: string): DraftLifecycleEvent[] {
  const fixturePath = path.resolve(
    process.cwd(),
    'tests',
    'fixtures',
    'draft-lifecycle',
    name,
  );
  return JSON.parse(readFileSync(fixturePath, 'utf8')) as DraftLifecycleEvent[];
}

describe('workflowLifecycle', () => {
  it('tracks generate-save-handoff journey via replay fixtures', () => {
    const events = loadFixture('generate-save-handoff.json');
    const final = replayDraftLifecycle(events);
    expect(final.stage).toBe('handoff');
    expect(final.currentDraftId).toBe('draft-123');
    expect(final.handoffTouched).toBe(true);
    expect(final.hasResponse).toBe(true);
  });

  it('tracks clear + reopen journey via replay fixtures', () => {
    const events = loadFixture('reopen-after-clear.json');
    const final = replayDraftLifecycle(events);
    expect(final.stage).toBe('saved');
    expect(final.currentDraftId).toBe('draft-abc');
    expect(final.handoffTouched).toBe(false);
    expect(final.hasResponse).toBe(true);
  });

  it('drops handoff when response is cleared', () => {
    const initial = createInitialDraftLifecycleState();
    const withResponse = reduceDraftLifecycle(initial, {
      type: 'generation_succeeded',
      at: '2026-03-06T00:02:00.000Z',
      payload: { hasResponse: true },
    });
    const withHandoff = reduceDraftLifecycle(withResponse, {
      type: 'handoff_completed',
      at: '2026-03-06T00:02:10.000Z',
    });
    const cleared = reduceDraftLifecycle(withHandoff, {
      type: 'response_updated',
      at: '2026-03-06T00:02:20.000Z',
      payload: { hasResponse: false },
    });
    expect(cleared.stage).toBe('input');
    expect(cleared.handoffTouched).toBe(false);
  });
});
