import {
  DEFAULT_RESPONSE_QUALITY_THRESHOLDS,
  ResponseQualityThresholds,
} from './qualityThresholds';

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
  drilldownHint: string;
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
  thresholds: ResponseQualityThresholds = DEFAULT_RESPONSE_QUALITY_THRESHOLDS,
): ResponseQualityCoachingSummary | null {
  if (!summary || summary.snapshots_count === 0) {
    return null;
  }

  const editRatioSeverity = classifyHigherIsRisk(
    summary.avg_edit_ratio,
    thresholds.editRatioWatch,
    thresholds.editRatioAction,
  );
  const avgTimeMs = summary.avg_time_to_draft_ms ?? 0;
  const timeSeverity = classifyHigherIsRisk(
    avgTimeMs,
    thresholds.timeToDraftWatchMs,
    thresholds.timeToDraftActionMs,
  );
  const copySeverity = classifyLowerIsRisk(
    summary.copy_per_saved_ratio,
    thresholds.copyPerSaveWatch,
    thresholds.copyPerSaveAction,
  );
  const editedSaveSeverity = classifyHigherIsRisk(
    summary.edited_save_rate,
    thresholds.editedSaveRateWatch,
    thresholds.editedSaveRateAction,
  );

  const signals: CoachingSignal[] = [
    {
      id: 'edit_ratio',
      label: 'Edit ratio',
      value: `${(summary.avg_edit_ratio * 100).toFixed(1)}%`,
      severity: editRatioSeverity,
      threshold: `<${(thresholds.editRatioWatch * 100).toFixed(0)}% healthy, ${(thresholds.editRatioWatch * 100).toFixed(0)}-${(thresholds.editRatioAction * 100).toFixed(0)}% watch, >${(thresholds.editRatioAction * 100).toFixed(0)}% action`,
      guidance: 'High edit churn usually signals weak first-pass quality. Tighten prompt framing and diagnostics.',
      drilldownHint: 'Review high edit-ratio drafts in the drill-down panel and compare prompt framing.',
    },
    {
      id: 'time_to_draft',
      label: 'Avg time to draft',
      value: `${(avgTimeMs / 1000).toFixed(1)}s`,
      severity: timeSeverity,
      threshold: `<${(thresholds.timeToDraftWatchMs / 1000).toFixed(0)}s healthy, ${(thresholds.timeToDraftWatchMs / 1000).toFixed(0)}-${(thresholds.timeToDraftActionMs / 1000).toFixed(0)}s watch, >${(thresholds.timeToDraftActionMs / 1000).toFixed(0)}s action`,
      guidance: 'Long draft times indicate workflow friction. Re-check queue context quality and checklist depth.',
      drilldownHint: 'Inspect longest time-to-draft examples to identify missing diagnostics or weak source context.',
    },
    {
      id: 'copy_per_save',
      label: 'Copy per save',
      value: `${(summary.copy_per_saved_ratio * 100).toFixed(1)}%`,
      severity: copySeverity,
      threshold: `>${(thresholds.copyPerSaveWatch * 100).toFixed(0)}% healthy, ${(thresholds.copyPerSaveAction * 100).toFixed(0)}-${(thresholds.copyPerSaveWatch * 100).toFixed(0)}% watch, <${(thresholds.copyPerSaveAction * 100).toFixed(0)}% action`,
      guidance: 'Low copy conversion means outputs are not getting reused. Review tone presets and response structure.',
      drilldownHint: 'Inspect saved-without-copy examples to identify responses that need structure or tone tuning.',
    },
    {
      id: 'edited_save_rate',
      label: 'Edited save rate',
      value: `${(summary.edited_save_rate * 100).toFixed(1)}%`,
      severity: editedSaveSeverity,
      threshold: `<${(thresholds.editedSaveRateWatch * 100).toFixed(0)}% healthy, ${(thresholds.editedSaveRateWatch * 100).toFixed(0)}-${(thresholds.editedSaveRateAction * 100).toFixed(0)}% watch, >${(thresholds.editedSaveRateAction * 100).toFixed(0)}% action`,
      guidance: 'If nearly every saved response is heavily edited, improve defaults before wider rollout.',
      drilldownHint: 'Review heavily edited saves and compare with the generated baseline to tune prompts.',
    },
  ];

  return {
    overallSeverity: calculateOverallSeverity(signals),
    signals,
  };
}
