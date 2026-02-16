import type { CoachingSeverity, ResponseQualityCoachingSummary } from './qualityCoaching';
import type { QueueHandoffSnapshot } from '../inbox/queueModel';

export type OperatorScorecardPosture = 'on-track' | 'watch' | 'at-risk';

export interface OperatorScorecardSignal {
  id: string;
  label: string;
  severity: CoachingSeverity;
  guidance: string;
}

export interface OperatorQueueTelemetrySummary {
  openQueue: number;
  atRiskRate: number;
  unassignedRate: number;
  workloadSkew: number;
}

export interface OperatorScorecard {
  score: number;
  posture: OperatorScorecardPosture;
  summary: string;
  prioritySignals: OperatorScorecardSignal[];
  queueTelemetry: OperatorQueueTelemetrySummary | null;
}

const SEVERITY_PENALTY: Record<CoachingSeverity, number> = {
  healthy: 0,
  watch: 8,
  action: 18,
};

const SEVERITY_ORDER: Record<CoachingSeverity, number> = {
  healthy: 0,
  watch: 1,
  action: 2,
};

const QUEUE_SIGNAL_PENALTY: Record<CoachingSeverity, number> = {
  healthy: 0,
  watch: 7,
  action: 15,
};

function classifyHigherIsRisk(value: number, watchThreshold: number, actionThreshold: number): CoachingSeverity {
  if (value >= actionThreshold) {
    return 'action';
  }
  if (value >= watchThreshold) {
    return 'watch';
  }
  return 'healthy';
}

function buildSummary(
  posture: OperatorScorecardPosture,
  score: number,
  queueTelemetry: OperatorQueueTelemetrySummary | null,
): string {
  const queueSegment =
    queueTelemetry && queueTelemetry.openQueue > 0
      ? ` Queue risk: ${(queueTelemetry.atRiskRate * 100).toFixed(0)}% at-risk, ${(queueTelemetry.unassignedRate * 100).toFixed(0)}% unassigned.`
      : '';

  if (posture === 'on-track') {
    return `Team quality posture is stable (${score}/100). Keep current workflow settings and monitor weekly drift.${queueSegment}`;
  }
  if (posture === 'watch') {
    return `Team quality posture needs attention (${score}/100). Address watch/action signals before volume increases.${queueSegment}`;
  }
  return `Team quality posture is at risk (${score}/100). Prioritize corrective actions before expanding usage.${queueSegment}`;
}

function buildQueueSignals(snapshot: QueueHandoffSnapshot | null): {
  telemetry: OperatorQueueTelemetrySummary | null;
  signals: OperatorScorecardSignal[];
  penalty: number;
} {
  if (!snapshot) {
    return { telemetry: null, signals: [], penalty: 0 };
  }

  const openQueue = Math.max(snapshot.summary.total - snapshot.summary.resolved, 0);
  if (openQueue <= 0) {
    return {
      telemetry: {
        openQueue,
        atRiskRate: 0,
        unassignedRate: 0,
        workloadSkew: 0,
      },
      signals: [],
      penalty: 0,
    };
  }

  const atRiskRate = snapshot.summary.atRisk / openQueue;
  const unassignedRate = snapshot.summary.unassigned / openQueue;

  const ownerLoads = snapshot.ownerWorkload
    .map((owner) => owner.openCount + owner.inProgressCount)
    .filter((count) => count > 0);
  const totalOwnerLoad = ownerLoads.reduce((sum, count) => sum + count, 0);
  const avgOwnerLoad = ownerLoads.length > 0 ? totalOwnerLoad / ownerLoads.length : 0;
  const maxOwnerLoad = ownerLoads.length > 0 ? Math.max(...ownerLoads) : 0;
  const workloadSkew = avgOwnerLoad > 0 ? maxOwnerLoad / avgOwnerLoad : 0;

  const atRiskSeverity = classifyHigherIsRisk(atRiskRate, 0.12, 0.25);
  const unassignedSeverity = classifyHigherIsRisk(unassignedRate, 0.2, 0.35);
  const skewSeverity = classifyHigherIsRisk(workloadSkew, 1.7, 2.4);

  const queueSignals: OperatorScorecardSignal[] = [
    {
      id: 'queue_at_risk_rate',
      label: 'At-risk queue rate',
      severity: atRiskSeverity,
      guidance:
        'Reduce queue risk by prioritizing urgent tickets and increasing triage coverage in high-SLA windows.',
    },
    {
      id: 'queue_unassigned_rate',
      label: 'Unassigned queue rate',
      severity: unassignedSeverity,
      guidance:
        'Decrease unassigned backlog by enforcing owner assignment during intake and routing handoff checks.',
    },
    {
      id: 'queue_workload_skew',
      label: 'Owner workload skew',
      severity: skewSeverity,
      guidance:
        'Rebalance owner load to prevent single-operator bottlenecks and protect response-time consistency.',
    },
  ].filter((signal) => signal.severity !== 'healthy');

  const penalty =
    QUEUE_SIGNAL_PENALTY[atRiskSeverity] +
    QUEUE_SIGNAL_PENALTY[unassignedSeverity] +
    QUEUE_SIGNAL_PENALTY[skewSeverity];

  return {
    telemetry: {
      openQueue,
      atRiskRate,
      unassignedRate,
      workloadSkew,
    },
    signals: queueSignals,
    penalty,
  };
}

export function buildOperatorScorecard(
  coaching: ResponseQualityCoachingSummary | null,
  queueSnapshot: QueueHandoffSnapshot | null = null,
): OperatorScorecard | null {
  if (!coaching) {
    return null;
  }

  const queueSignals = buildQueueSignals(queueSnapshot);
  const qualityPenalty = coaching.signals.reduce((sum, signal) => sum + SEVERITY_PENALTY[signal.severity], 0);

  const score = Math.max(
    0,
    Math.min(
      100,
      Math.round(
        100 - qualityPenalty - queueSignals.penalty,
      ),
    ),
  );

  const hasActionSignal =
    coaching.signals.some((signal) => signal.severity === 'action') ||
    queueSignals.signals.some((signal) => signal.severity === 'action');
  let posture: OperatorScorecardPosture = 'on-track';
  if (hasActionSignal || score < 70) {
    posture = 'at-risk';
  } else if (score < 85 || coaching.overallSeverity === 'watch') {
    posture = 'watch';
  }

  const prioritySignals = coaching.signals
    .filter((signal) => signal.severity !== 'healthy')
    .map<OperatorScorecardSignal>((signal) => ({
      id: signal.id,
      label: signal.label,
      severity: signal.severity,
      guidance: signal.guidance,
    }))
    .concat(queueSignals.signals)
    .sort((a, b) => {
      const severityDiff = SEVERITY_ORDER[b.severity] - SEVERITY_ORDER[a.severity];
      if (severityDiff !== 0) {
        return severityDiff;
      }
      return a.label.localeCompare(b.label);
    })
    .slice(0, 4);

  return {
    score,
    posture,
    summary: buildSummary(posture, score, queueSignals.telemetry),
    prioritySignals,
    queueTelemetry: queueSignals.telemetry,
  };
}
