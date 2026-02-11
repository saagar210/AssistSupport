import { describe, expect, it } from 'vitest';
import type { SavedDraft } from '../../types';
import {
  buildQueueHandoffDelta,
  buildQueueHandoffSnapshot,
  buildQueueItems,
  filterQueueItems,
  inferPriorityFromDraft,
  loadQueueHandoffSnapshot,
  loadQueueMeta,
  persistQueueHandoffSnapshot,
  persistQueueMeta,
  summarizeQueue,
  summarizeQueueByOwner,
  summarizeQueueByPriority,
  type QueueMetaMap,
} from './queueModel';

function makeDraft(overrides: Partial<SavedDraft> = {}): SavedDraft {
  return {
    id: overrides.id ?? 'draft-1',
    input_text: overrides.input_text ?? 'User cannot sign in',
    summary_text: overrides.summary_text ?? 'Account login failed',
    diagnosis_json: null,
    response_text: null,
    ticket_id: overrides.ticket_id ?? 'INC-1001',
    kb_sources_json: null,
    created_at: overrides.created_at ?? '2026-02-08T00:00:00Z',
    updated_at: overrides.updated_at ?? '2026-02-08T00:00:00Z',
    is_autosave: false,
  };
}

describe('queueModel', () => {
  it('infers priority from escalation keywords', () => {
    const urgent = makeDraft({ input_text: 'sev1 outage in production, urgent' });
    const high = makeDraft({ input_text: 'customer blocked after deployment failure' });
    const normal = makeDraft({ input_text: 'routine follow-up question' });

    expect(inferPriorityFromDraft(urgent)).toBe('urgent');
    expect(inferPriorityFromDraft(high)).toBe('high');
    expect(inferPriorityFromDraft(normal)).toBe('normal');
  });

  it('builds and sorts queue items with SLA/at-risk signals', () => {
    const drafts = [
      makeDraft({ id: 'a', input_text: 'sev1 outage', updated_at: '2026-02-08T00:00:00Z' }),
      makeDraft({ id: 'b', input_text: 'routine issue', updated_at: '2026-02-08T01:00:00Z' }),
      makeDraft({ id: 'c', input_text: 'blocked deployment', updated_at: '2026-02-08T02:00:00Z' }),
    ];

    const meta: QueueMetaMap = {
      b: {
        owner: 'agent-a',
        priority: 'low',
        state: 'resolved',
        updatedAt: '2026-02-08T01:00:00Z',
      },
    };

    const nowMs = Date.parse('2026-02-08T10:00:00Z');
    const items = buildQueueItems(drafts, meta, nowMs);

    expect(items[0].draft.id).toBe('a');
    expect(items[0].isAtRisk).toBe(true);
    expect(items[items.length - 1].draft.id).toBe('b');
  });

  it('filters and summarizes queue states', () => {
    const drafts = [
      makeDraft({ id: 'a', input_text: 'sev1 outage', updated_at: '2026-02-08T00:00:00Z' }),
      makeDraft({ id: 'b', input_text: 'routine issue', updated_at: '2026-02-08T01:00:00Z' }),
    ];

    const meta: QueueMetaMap = {
      a: { owner: 'unassigned', priority: 'urgent', state: 'open', updatedAt: '2026-02-08T00:00:00Z' },
      b: { owner: 'agent-a', priority: 'normal', state: 'in_progress', updatedAt: '2026-02-08T01:00:00Z' },
    };

    const items = buildQueueItems(drafts, meta, Date.parse('2026-02-08T12:00:00Z'));
    const unassigned = filterQueueItems(items, 'unassigned');
    const inProgress = filterQueueItems(items, 'in_progress');
    const summary = summarizeQueue(items);

    expect(unassigned).toHaveLength(1);
    expect(unassigned[0].draft.id).toBe('a');
    expect(inProgress).toHaveLength(1);
    expect(summary.total).toBe(2);
    expect(summary.unassigned).toBe(1);
    expect(summary.inProgress).toBe(1);
  });

  it('persists and safely loads queue metadata', () => {
    const store = new Map<string, string>();
    const storage = {
      getItem: (key: string) => store.get(key) ?? null,
      setItem: (key: string, value: string) => {
        store.set(key, value);
      },
    };

    const meta: QueueMetaMap = {
      x: { owner: 'agent-a', priority: 'high', state: 'in_progress', updatedAt: '2026-02-08T00:00:00Z' },
    };

    persistQueueMeta(meta, storage);
    expect(loadQueueMeta(storage)).toEqual(meta);

    storage.setItem('assistsupport.queue.meta.v1', '{broken-json');
    expect(loadQueueMeta(storage)).toEqual({});
  });

  it('normalizes malformed metadata entries loaded from storage', () => {
    const store = new Map<string, string>();
    const storage = {
      getItem: (key: string) => store.get(key) ?? null,
      setItem: (key: string, value: string) => {
        store.set(key, value);
      },
    };

    storage.setItem(
      'assistsupport.queue.meta.v1',
      JSON.stringify({
        valid: { owner: 'agent-a', priority: 'high', state: 'in_progress', updatedAt: '2026-02-08T00:00:00Z' },
        blankOwner: { owner: '  ', priority: 'low', state: 'open', updatedAt: 'not-a-date' },
        invalidValues: { owner: 'agent-b', priority: 'critical', state: 'waiting', updatedAt: 'also-bad' },
        primitive: 42,
      }),
    );

    expect(loadQueueMeta(storage)).toEqual({
      valid: { owner: 'agent-a', priority: 'high', state: 'in_progress', updatedAt: '2026-02-08T00:00:00Z' },
      blankOwner: { owner: 'unassigned', priority: 'low', state: 'open', updatedAt: '' },
      invalidValues: { owner: 'agent-b', priority: 'normal', state: 'open', updatedAt: '' },
    });
  });

  it('builds queue analytics and handoff snapshot', () => {
    const drafts = [
      makeDraft({ id: 'a', input_text: 'sev1 outage', ticket_id: 'INC-1', updated_at: '2026-02-08T00:00:00Z' }),
      makeDraft({ id: 'b', input_text: 'routine issue', ticket_id: 'INC-2', updated_at: '2026-02-08T01:00:00Z' }),
      makeDraft({ id: 'c', input_text: 'blocked deployment', ticket_id: 'INC-3', updated_at: '2026-02-08T02:00:00Z' }),
    ];

    const meta: QueueMetaMap = {
      a: { owner: 'unassigned', priority: 'urgent', state: 'open', updatedAt: '2026-02-08T00:00:00Z' },
      b: { owner: 'agent-a', priority: 'normal', state: 'in_progress', updatedAt: '2026-02-08T01:00:00Z' },
      c: { owner: 'agent-a', priority: 'high', state: 'resolved', updatedAt: '2026-02-08T02:00:00Z' },
    };

    const items = buildQueueItems(drafts, meta, Date.parse('2026-02-08T12:00:00Z'));
    const byPriority = summarizeQueueByPriority(items);
    const byOwner = summarizeQueueByOwner(items);
    const snapshot = buildQueueHandoffSnapshot(items, '2026-02-08T13:00:00Z');

    expect(byPriority.urgent).toBe(1);
    expect(byPriority.high).toBe(0);
    expect(byOwner[0]).toEqual({
      owner: 'agent-a',
      openCount: 0,
      inProgressCount: 1,
      atRiskCount: 1,
    });
    expect(snapshot.generatedAt).toBe('2026-02-08T13:00:00Z');
    expect(snapshot.summary.total).toBe(3);
    expect(snapshot.topAtRisk[0]?.ticketLabel).toBe('INC-1');
  });

  it('persists handoff snapshots and calculates trend deltas', () => {
    const store = new Map<string, string>();
    const storage = {
      getItem: (key: string) => store.get(key) ?? null,
      setItem: (key: string, value: string) => {
        store.set(key, value);
      },
    };

    const previous = {
      generatedAt: '2026-02-08T10:00:00Z',
      summary: {
        total: 5,
        unassigned: 2,
        inProgress: 1,
        resolved: 1,
        atRisk: 3,
      },
      prioritySummary: { low: 0, normal: 2, high: 2, urgent: 0 },
      ownerWorkload: [
        { owner: 'agent-a', openCount: 1, inProgressCount: 1, atRiskCount: 1 },
        { owner: 'unassigned', openCount: 2, inProgressCount: 0, atRiskCount: 2 },
      ],
      topAtRisk: [],
    } as const;

    const current = {
      generatedAt: '2026-02-08T11:00:00Z',
      summary: {
        total: 6,
        unassigned: 1,
        inProgress: 3,
        resolved: 1,
        atRisk: 2,
      },
      prioritySummary: { low: 0, normal: 1, high: 2, urgent: 2 },
      ownerWorkload: [
        { owner: 'agent-a', openCount: 1, inProgressCount: 2, atRiskCount: 0 },
        { owner: 'unassigned', openCount: 1, inProgressCount: 0, atRiskCount: 1 },
      ],
      topAtRisk: [],
    } as const;

    persistQueueHandoffSnapshot(previous, storage);
    expect(loadQueueHandoffSnapshot(storage)).toEqual(previous);

    const delta = buildQueueHandoffDelta(current, previous);
    expect(delta.previousGeneratedAt).toBe('2026-02-08T10:00:00Z');
    expect(delta.summaryDelta).toEqual({
      open: 1,
      inProgress: 2,
      resolved: 0,
      atRisk: -1,
      unassigned: -1,
    });
    expect(delta.ownerDelta[0]).toEqual({
      owner: 'agent-a',
      workloadDelta: 1,
      atRiskDelta: -1,
    });

    storage.setItem('assistsupport.queue.handoff.latest.v1', '{broken-json');
    expect(loadQueueHandoffSnapshot(storage)).toBeNull();
  });
});
