import { describe, expect, it } from 'vitest';
import { buildOperatorScorecard } from './operatorScorecard';
import type { ResponseQualityCoachingSummary } from './qualityCoaching';
import type { QueueHandoffSnapshot } from '../inbox/queueModel';

function createCoaching(signals: ResponseQualityCoachingSummary['signals']): ResponseQualityCoachingSummary {
  const overallSeverity = signals.some((signal) => signal.severity === 'action')
    ? 'action'
    : signals.some((signal) => signal.severity === 'watch')
      ? 'watch'
      : 'healthy';
  return { overallSeverity, signals };
}

function createQueueSnapshot(overrides?: Partial<QueueHandoffSnapshot>): QueueHandoffSnapshot {
  return {
    generatedAt: '2026-02-08T00:00:00.000Z',
    summary: {
      total: 12,
      unassigned: 4,
      inProgress: 3,
      resolved: 2,
      atRisk: 4,
    },
    prioritySummary: {
      low: 1,
      normal: 5,
      high: 4,
      urgent: 2,
    },
    ownerWorkload: [
      { owner: 'alice', openCount: 6, inProgressCount: 2, atRiskCount: 3 },
      { owner: 'bob', openCount: 1, inProgressCount: 0, atRiskCount: 1 },
    ],
    topAtRisk: [],
    ...overrides,
  };
}

describe('buildOperatorScorecard', () => {
  it('returns null without coaching data', () => {
    expect(buildOperatorScorecard(null)).toBeNull();
  });

  it('classifies healthy posture for all healthy signals', () => {
    const scorecard = buildOperatorScorecard(createCoaching([
      { id: 'edit_ratio', label: 'Edit ratio', value: '12%', severity: 'healthy', threshold: '', guidance: '', drilldownHint: '' },
      { id: 'time_to_draft', label: 'Avg time to draft', value: '45s', severity: 'healthy', threshold: '', guidance: '', drilldownHint: '' },
      { id: 'copy_per_save', label: 'Copy per save', value: '88%', severity: 'healthy', threshold: '', guidance: '', drilldownHint: '' },
      { id: 'edited_save_rate', label: 'Edited save rate', value: '28%', severity: 'healthy', threshold: '', guidance: '', drilldownHint: '' },
    ]));

    expect(scorecard).not.toBeNull();
    expect(scorecard?.posture).toBe('on-track');
    expect(scorecard?.score).toBe(100);
    expect(scorecard?.prioritySignals).toHaveLength(0);
  });

  it('classifies at-risk when action signals are present and prioritizes them first', () => {
    const scorecard = buildOperatorScorecard(createCoaching([
      { id: 'edit_ratio', label: 'Edit ratio', value: '42%', severity: 'action', threshold: '', guidance: '', drilldownHint: '' },
      { id: 'time_to_draft', label: 'Avg time to draft', value: '120s', severity: 'watch', threshold: '', guidance: '', drilldownHint: '' },
      { id: 'copy_per_save', label: 'Copy per save', value: '60%', severity: 'watch', threshold: '', guidance: '', drilldownHint: '' },
      { id: 'edited_save_rate', label: 'Edited save rate', value: '38%', severity: 'action', threshold: '', guidance: '', drilldownHint: '' },
    ]));

    expect(scorecard).not.toBeNull();
    expect(scorecard?.posture).toBe('at-risk');
    expect(scorecard?.score).toBe(48);
    expect(scorecard?.prioritySignals[0].severity).toBe('action');
    expect(scorecard?.prioritySignals).toHaveLength(4);
  });

  it('incorporates queue telemetry risk signals into scorecard posture and priority actions', () => {
    const scorecard = buildOperatorScorecard(createCoaching([
      { id: 'edit_ratio', label: 'Edit ratio', value: '12%', severity: 'healthy', threshold: '', guidance: '', drilldownHint: '' },
      { id: 'time_to_draft', label: 'Avg time to draft', value: '45s', severity: 'healthy', threshold: '', guidance: '', drilldownHint: '' },
      { id: 'copy_per_save', label: 'Copy per save', value: '88%', severity: 'healthy', threshold: '', guidance: '', drilldownHint: '' },
      { id: 'edited_save_rate', label: 'Edited save rate', value: '28%', severity: 'healthy', threshold: '', guidance: '', drilldownHint: '' },
    ]), createQueueSnapshot());

    expect(scorecard).not.toBeNull();
    expect(scorecard?.posture).toBe('at-risk');
    expect(scorecard?.score).toBeLessThan(100);
    expect(scorecard?.queueTelemetry?.openQueue).toBe(10);
    expect(scorecard?.prioritySignals.some((signal) => signal.id === 'queue_at_risk_rate')).toBe(true);
  });
});
