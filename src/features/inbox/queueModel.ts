import type { SavedDraft } from '../../types';

export type QueueState = 'open' | 'in_progress' | 'resolved';
export type QueuePriority = 'low' | 'normal' | 'high' | 'urgent';
export type QueueView = 'all' | 'unassigned' | 'at_risk' | 'in_progress' | 'resolved';

export interface QueueMeta {
  state: QueueState;
  owner: string;
  priority: QueuePriority;
  updatedAt: string;
}

export type QueueMetaMap = Record<string, QueueMeta>;

export interface QueueItem {
  draft: SavedDraft;
  meta: QueueMeta;
  slaDueAt: string;
  isAtRisk: boolean;
}

export interface QueueSummary {
  total: number;
  unassigned: number;
  inProgress: number;
  resolved: number;
  atRisk: number;
}

export interface QueuePrioritySummary {
  low: number;
  normal: number;
  high: number;
  urgent: number;
}

export interface QueueOwnerWorkload {
  owner: string;
  openCount: number;
  inProgressCount: number;
  atRiskCount: number;
}

export interface QueueHandoffSnapshot {
  generatedAt: string;
  summary: QueueSummary;
  prioritySummary: QueuePrioritySummary;
  ownerWorkload: QueueOwnerWorkload[];
  topAtRisk: Array<{
    draftId: string;
    ticketLabel: string;
    owner: string;
    priority: QueuePriority;
    slaDueAt: string;
  }>;
}

export interface QueueHandoffDelta {
  previousGeneratedAt: string | null;
  summaryDelta: {
    open: number;
    inProgress: number;
    resolved: number;
    atRisk: number;
    unassigned: number;
  };
  ownerDelta: Array<{
    owner: string;
    workloadDelta: number;
    atRiskDelta: number;
  }>;
}

const QUEUE_META_STORAGE_KEY = 'assistsupport.queue.meta.v1';
const QUEUE_HANDOFF_SNAPSHOT_STORAGE_KEY = 'assistsupport.queue.handoff.latest.v1';

const PRIORITY_RANK: Record<QueuePriority, number> = {
  urgent: 4,
  high: 3,
  normal: 2,
  low: 1,
};

const SLA_HOURS_BY_PRIORITY: Record<QueuePriority, number> = {
  urgent: 1,
  high: 4,
  normal: 8,
  low: 24,
};

const ESCALATION_REGEX = /\b(sev1|p1|critical|outage|urgent|production down)\b/i;
const HIGH_PRIORITY_REGEX = /\b(sev2|p2|blocked|escalat|cannot access|failure)\b/i;

export function inferPriorityFromDraft(draft: SavedDraft): QueuePriority {
  const ticketText = [draft.ticket_id, draft.summary_text, draft.input_text].filter(Boolean).join(' ');
  if (ESCALATION_REGEX.test(ticketText)) {
    return 'urgent';
  }

  if (HIGH_PRIORITY_REGEX.test(ticketText)) {
    return 'high';
  }

  return 'normal';
}

function createDefaultMeta(draft: SavedDraft): QueueMeta {
  return {
    state: 'open',
    owner: 'unassigned',
    priority: inferPriorityFromDraft(draft),
    updatedAt: draft.updated_at,
  };
}

function safeParseQueueMeta(raw: string | null): QueueMetaMap {
  if (!raw) {
    return {};
  }

  try {
    const parsed = JSON.parse(raw) as QueueMetaMap;
    if (typeof parsed !== 'object' || parsed === null) {
      return {};
    }
    return parsed;
  } catch {
    return {};
  }
}

export function loadQueueMeta(storage: Pick<Storage, 'getItem'> | null = typeof window !== 'undefined' ? window.localStorage : null): QueueMetaMap {
  if (!storage) {
    return {};
  }

  try {
    return safeParseQueueMeta(storage.getItem(QUEUE_META_STORAGE_KEY));
  } catch {
    return {};
  }
}

export function persistQueueMeta(metaMap: QueueMetaMap, storage: Pick<Storage, 'setItem'> | null = typeof window !== 'undefined' ? window.localStorage : null): void {
  if (!storage) {
    return;
  }

  try {
    storage.setItem(QUEUE_META_STORAGE_KEY, JSON.stringify(metaMap));
  } catch {
    // Ignore persistence failures to keep queue workflows functional in restricted environments.
  }
}

function getReferenceTimestamp(isoTimestamp: string | null | undefined, fallbackMs: number): number {
  if (!isoTimestamp) {
    return fallbackMs;
  }

  const ms = Date.parse(isoTimestamp);
  return Number.isNaN(ms) ? fallbackMs : ms;
}

export function buildQueueItems(drafts: SavedDraft[], metaMap: QueueMetaMap, nowMs = Date.now()): QueueItem[] {
  return drafts
    .map((draft) => {
      const meta = metaMap[draft.id] ?? createDefaultMeta(draft);
      const referenceMs = getReferenceTimestamp(meta.updatedAt || draft.updated_at, nowMs);
      const slaHours = SLA_HOURS_BY_PRIORITY[meta.priority] ?? SLA_HOURS_BY_PRIORITY.normal;
      const slaDueAtMs = referenceMs + slaHours * 60 * 60 * 1000;
      const isAtRisk = meta.state !== 'resolved' && nowMs > slaDueAtMs;

      return {
        draft,
        meta,
        slaDueAt: new Date(slaDueAtMs).toISOString(),
        isAtRisk,
      };
    })
    .sort((a, b) => {
      if (a.meta.state === 'resolved' && b.meta.state !== 'resolved') {
        return 1;
      }
      if (b.meta.state === 'resolved' && a.meta.state !== 'resolved') {
        return -1;
      }
      if (a.isAtRisk && !b.isAtRisk) {
        return -1;
      }
      if (b.isAtRisk && !a.isAtRisk) {
        return 1;
      }

      const priorityDiff = PRIORITY_RANK[b.meta.priority] - PRIORITY_RANK[a.meta.priority];
      if (priorityDiff !== 0) {
        return priorityDiff;
      }

      return Date.parse(b.draft.updated_at) - Date.parse(a.draft.updated_at);
    });
}

export function filterQueueItems(items: QueueItem[], view: QueueView): QueueItem[] {
  switch (view) {
    case 'unassigned':
      return items.filter((item) => item.meta.owner === 'unassigned' && item.meta.state !== 'resolved');
    case 'at_risk':
      return items.filter((item) => item.isAtRisk && item.meta.state !== 'resolved');
    case 'in_progress':
      return items.filter((item) => item.meta.state === 'in_progress');
    case 'resolved':
      return items.filter((item) => item.meta.state === 'resolved');
    case 'all':
    default:
      return items;
  }
}

export function summarizeQueue(items: QueueItem[]): QueueSummary {
  return {
    total: items.length,
    unassigned: items.filter((item) => item.meta.owner === 'unassigned' && item.meta.state !== 'resolved').length,
    inProgress: items.filter((item) => item.meta.state === 'in_progress').length,
    resolved: items.filter((item) => item.meta.state === 'resolved').length,
    atRisk: items.filter((item) => item.isAtRisk && item.meta.state !== 'resolved').length,
  };
}

export function summarizeQueueByPriority(items: QueueItem[]): QueuePrioritySummary {
  return items.reduce<QueuePrioritySummary>(
    (acc, item) => {
      if (item.meta.state === 'resolved') {
        return acc;
      }

      acc[item.meta.priority] += 1;
      return acc;
    },
    { low: 0, normal: 0, high: 0, urgent: 0 },
  );
}

export function summarizeQueueByOwner(items: QueueItem[]): QueueOwnerWorkload[] {
  const buckets = new Map<string, QueueOwnerWorkload>();

  for (const item of items) {
    if (item.meta.state === 'resolved') {
      continue;
    }

    const key = item.meta.owner || 'unassigned';
    const current = buckets.get(key) ?? {
      owner: key,
      openCount: 0,
      inProgressCount: 0,
      atRiskCount: 0,
    };

    if (item.meta.state === 'in_progress') {
      current.inProgressCount += 1;
    } else {
      current.openCount += 1;
    }

    if (item.isAtRisk) {
      current.atRiskCount += 1;
    }

    buckets.set(key, current);
  }

  return Array.from(buckets.values()).sort((a, b) => {
    const aTotal = a.openCount + a.inProgressCount;
    const bTotal = b.openCount + b.inProgressCount;
    if (bTotal !== aTotal) {
      return bTotal - aTotal;
    }
    return a.owner.localeCompare(b.owner);
  });
}

function getQueueTicketLabel(draft: SavedDraft): string {
  return draft.ticket_id?.trim() || `Draft ${draft.id.slice(0, 8)}`;
}

export function buildQueueHandoffSnapshot(items: QueueItem[], generatedAt = new Date().toISOString()): QueueHandoffSnapshot {
  const atRisk = items
    .filter((item) => item.isAtRisk && item.meta.state !== 'resolved')
    .slice(0, 10)
    .map((item) => ({
      draftId: item.draft.id,
      ticketLabel: getQueueTicketLabel(item.draft),
      owner: item.meta.owner,
      priority: item.meta.priority,
      slaDueAt: item.slaDueAt,
    }));

  return {
    generatedAt,
    summary: summarizeQueue(items),
    prioritySummary: summarizeQueueByPriority(items),
    ownerWorkload: summarizeQueueByOwner(items),
    topAtRisk: atRisk,
  };
}

export function formatQueueTimestamp(value: string): string {
  const parsed = Date.parse(value);
  if (Number.isNaN(parsed)) {
    return 'Unknown';
  }

  return new Date(parsed).toLocaleString('en-US', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}

function calculateOpen(summary: QueueSummary): number {
  return Math.max(summary.total - summary.resolved, 0);
}

function parseHandoffSnapshot(raw: string | null): QueueHandoffSnapshot | null {
  if (!raw) {
    return null;
  }

  try {
    const parsed = JSON.parse(raw) as QueueHandoffSnapshot;
    if (!parsed || typeof parsed !== 'object') {
      return null;
    }

    if (!parsed.generatedAt || !parsed.summary || !parsed.prioritySummary || !Array.isArray(parsed.ownerWorkload)) {
      return null;
    }

    return parsed;
  } catch {
    return null;
  }
}

export function loadQueueHandoffSnapshot(
  storage: Pick<Storage, 'getItem'> | null = typeof window !== 'undefined' ? window.localStorage : null,
): QueueHandoffSnapshot | null {
  if (!storage) {
    return null;
  }

  try {
    return parseHandoffSnapshot(storage.getItem(QUEUE_HANDOFF_SNAPSHOT_STORAGE_KEY));
  } catch {
    return null;
  }
}

export function persistQueueHandoffSnapshot(
  snapshot: QueueHandoffSnapshot,
  storage: Pick<Storage, 'setItem'> | null = typeof window !== 'undefined' ? window.localStorage : null,
): void {
  if (!storage) {
    return;
  }

  try {
    storage.setItem(QUEUE_HANDOFF_SNAPSHOT_STORAGE_KEY, JSON.stringify(snapshot));
  } catch {
    // Ignore storage failures so handoff workflows remain usable.
  }
}

export function buildQueueHandoffDelta(
  current: QueueHandoffSnapshot,
  previous: QueueHandoffSnapshot | null,
): QueueHandoffDelta {
  if (!previous) {
    return {
      previousGeneratedAt: null,
      summaryDelta: {
        open: 0,
        inProgress: 0,
        resolved: 0,
        atRisk: 0,
        unassigned: 0,
      },
      ownerDelta: [],
    };
  }

  const previousOwners = new Map(previous.ownerWorkload.map((owner) => [owner.owner, owner]));
  const currentOwners = new Map(current.ownerWorkload.map((owner) => [owner.owner, owner]));
  const ownerKeys = new Set([...previousOwners.keys(), ...currentOwners.keys()]);

  const ownerDelta = Array.from(ownerKeys)
    .map((owner) => {
      const previousOwner = previousOwners.get(owner);
      const currentOwner = currentOwners.get(owner);
      const previousWorkload = (previousOwner?.openCount ?? 0) + (previousOwner?.inProgressCount ?? 0);
      const currentWorkload = (currentOwner?.openCount ?? 0) + (currentOwner?.inProgressCount ?? 0);
      return {
        owner,
        workloadDelta: currentWorkload - previousWorkload,
        atRiskDelta: (currentOwner?.atRiskCount ?? 0) - (previousOwner?.atRiskCount ?? 0),
      };
    })
    .filter((entry) => entry.workloadDelta !== 0 || entry.atRiskDelta !== 0)
    .sort((a, b) => {
      const aWeight = Math.abs(a.atRiskDelta) * 10 + Math.abs(a.workloadDelta);
      const bWeight = Math.abs(b.atRiskDelta) * 10 + Math.abs(b.workloadDelta);
      if (bWeight !== aWeight) {
        return bWeight - aWeight;
      }
      return a.owner.localeCompare(b.owner);
    })
    .slice(0, 5);

  return {
    previousGeneratedAt: previous.generatedAt,
    summaryDelta: {
      open: calculateOpen(current.summary) - calculateOpen(previous.summary),
      inProgress: current.summary.inProgress - previous.summary.inProgress,
      resolved: current.summary.resolved - previous.summary.resolved,
      atRisk: current.summary.atRisk - previous.summary.atRisk,
      unassigned: current.summary.unassigned - previous.summary.unassigned,
    },
    ownerDelta,
  };
}
