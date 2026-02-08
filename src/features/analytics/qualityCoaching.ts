export type CoachingSeverity = 'healthy' | 'watch' | 'action';

export interface ResponseQualityCoachingInput {
  snapshots_count: number;
  avg_edit_ratio: number;
  avg_time_to_draft_ms: number | null;
  copy_per_saved_ratio: number;
  edited_save_rate: number;
}

export interface CoachingSignal {
  id: 'edit_ratio' | 'time_to_draft' | 'copy_per_save' | 'edited_save_rate';
  label: string;
  value: string;
  severity: CoachingSeverity;
  threshold: string;
  guidance: string;
}

export interface ResponseQualityCoachingSummary {
  overallSeverity: CoachingSeverity;
  signals: CoachingSignal[];
}

const SEVERITY_RANK: Record<CoachingSeverity, number> = {
  healthy: 0,
  watch: 1,
  action: 2,
};

function classifyHigherIsRisk(
  value: number,
  watchThreshold: number,
  actionThreshold: number,
): CoachingSeverity {
  if (value >= actionThreshold) {
    return 'action';
  }
  if (value >= watchThreshold) {
    return 'watch';
  }
  return 'healthy';
}

function classifyLowerIsRisk(
  value: number,
  watchThreshold: number,
  actionThreshold: number,
): CoachingSeverity {
  if (value <= actionThreshold) {
    return 'action';
  }
  if (value <= watchThreshold) {
    return 'watch';
  }
  return 'healthy';
}

function calculateOverallSeverity(signals: CoachingSignal[]): CoachingSeverity {
  let maxSeverity: CoachingSeverity = 'healthy';
  for (const signal of signals) {
    if (SEVERITY_RANK[signal.severity] > SEVERITY_RANK[maxSeverity]) {
      maxSeverity = signal.severity;
    }
  }
  return maxSeverity;
}

export function buildResponseQualityCoaching(
  summary: ResponseQualityCoachingInput | null,
): ResponseQualityCoachingSummary | null {
  if (!summary || summary.snapshots_count === 0) {
    return null;
  }

  const editRatioSeverity = classifyHigherIsRisk(summary.avg_edit_ratio, 0.2, 0.35);
  const avgTimeMs = summary.avg_time_to_draft_ms ?? 0;
  const timeSeverity = classifyHigherIsRisk(avgTimeMs, 90_000, 180_000);
  const copySeverity = classifyLowerIsRisk(summary.copy_per_saved_ratio, 0.6, 0.35);
  const editedSaveSeverity = classifyHigherIsRisk(summary.edited_save_rate, 0.7, 0.85);

  const signals: CoachingSignal[] = [
    {
      id: 'edit_ratio',
      label: 'Edit ratio',
      value: `${(summary.avg_edit_ratio * 100).toFixed(1)}%`,
      severity: editRatioSeverity,
      threshold: '<20% healthy, 20-35% watch, >35% action',
      guidance: 'High edit churn usually signals weak first-pass quality. Tighten prompt framing and diagnostics.',
    },
    {
      id: 'time_to_draft',
      label: 'Avg time to draft',
      value: `${(avgTimeMs / 1000).toFixed(1)}s`,
      severity: timeSeverity,
      threshold: '<90s healthy, 90-180s watch, >180s action',
      guidance: 'Long draft times indicate workflow friction. Re-check queue context quality and checklist depth.',
    },
    {
      id: 'copy_per_save',
      label: 'Copy per save',
      value: `${(summary.copy_per_saved_ratio * 100).toFixed(1)}%`,
      severity: copySeverity,
      threshold: '>60% healthy, 35-60% watch, <35% action',
      guidance: 'Low copy conversion means outputs are not getting reused. Review tone presets and response structure.',
    },
    {
      id: 'edited_save_rate',
      label: 'Edited save rate',
      value: `${(summary.edited_save_rate * 100).toFixed(1)}%`,
      severity: editedSaveSeverity,
      threshold: '<70% healthy, 70-85% watch, >85% action',
      guidance: 'If nearly every saved response is heavily edited, improve defaults before wider rollout.',
    },
  ];

  return {
    overallSeverity: calculateOverallSeverity(signals),
    signals,
  };
}
