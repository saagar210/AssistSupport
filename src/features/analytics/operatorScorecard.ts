import type { CoachingSignal, CoachingSeverity, ResponseQualityCoachingSummary } from './qualityCoaching';

export type OperatorScorecardPosture = 'on-track' | 'watch' | 'at-risk';

export interface OperatorScorecard {
  score: number;
  posture: OperatorScorecardPosture;
  summary: string;
  prioritySignals: CoachingSignal[];
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

function buildSummary(posture: OperatorScorecardPosture, score: number): string {
  if (posture === 'on-track') {
    return `Team quality posture is stable (${score}/100). Keep current workflow settings and monitor weekly drift.`;
  }
  if (posture === 'watch') {
    return `Team quality posture needs attention (${score}/100). Address watch/action signals before volume increases.`;
  }
  return `Team quality posture is at risk (${score}/100). Prioritize corrective actions before expanding usage.`;
}

export function buildOperatorScorecard(
  coaching: ResponseQualityCoachingSummary | null,
): OperatorScorecard | null {
  if (!coaching) {
    return null;
  }

  const score = Math.max(
    0,
    Math.min(
      100,
      Math.round(
        100 - coaching.signals.reduce((sum, signal) => sum + SEVERITY_PENALTY[signal.severity], 0),
      ),
    ),
  );

  const hasActionSignal = coaching.signals.some((signal) => signal.severity === 'action');
  let posture: OperatorScorecardPosture = 'on-track';
  if (hasActionSignal || score < 70) {
    posture = 'at-risk';
  } else if (score < 85 || coaching.overallSeverity === 'watch') {
    posture = 'watch';
  }

  const prioritySignals = coaching.signals
    .filter((signal) => signal.severity !== 'healthy')
    .sort((a, b) => {
      const severityDiff = SEVERITY_ORDER[b.severity] - SEVERITY_ORDER[a.severity];
      if (severityDiff !== 0) {
        return severityDiff;
      }
      return a.label.localeCompare(b.label);
    })
    .slice(0, 3);

  return {
    score,
    posture,
    summary: buildSummary(posture, score),
    prioritySignals,
  };
}

