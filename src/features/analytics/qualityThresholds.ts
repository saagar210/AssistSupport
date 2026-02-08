export interface ResponseQualityThresholds {
  editRatioWatch: number;
  editRatioAction: number;
  timeToDraftWatchMs: number;
  timeToDraftActionMs: number;
  copyPerSaveWatch: number;
  copyPerSaveAction: number;
  editedSaveRateWatch: number;
  editedSaveRateAction: number;
}

export const RESPONSE_QUALITY_THRESHOLDS_STORAGE_KEY =
  'assistsupport-response-quality-thresholds-v1';
export const RESPONSE_QUALITY_THRESHOLDS_UPDATED_EVENT =
  'assistsupport:response-quality-thresholds-updated';

export const DEFAULT_RESPONSE_QUALITY_THRESHOLDS: ResponseQualityThresholds = {
  editRatioWatch: 0.2,
  editRatioAction: 0.35,
  timeToDraftWatchMs: 90_000,
  timeToDraftActionMs: 180_000,
  copyPerSaveWatch: 0.6,
  copyPerSaveAction: 0.35,
  editedSaveRateWatch: 0.7,
  editedSaveRateAction: 0.85,
};

function isFiniteNumber(value: unknown): value is number {
  return typeof value === 'number' && Number.isFinite(value);
}

function cloneDefaultThresholds(): ResponseQualityThresholds {
  return { ...DEFAULT_RESPONSE_QUALITY_THRESHOLDS };
}

function parseUnknownObject(
  value: unknown,
): Partial<ResponseQualityThresholds> | null {
  if (!value || typeof value !== 'object') {
    return null;
  }
  const candidate = value as Record<string, unknown>;
  return {
    editRatioWatch: isFiniteNumber(candidate.editRatioWatch)
      ? candidate.editRatioWatch
      : undefined,
    editRatioAction: isFiniteNumber(candidate.editRatioAction)
      ? candidate.editRatioAction
      : undefined,
    timeToDraftWatchMs: isFiniteNumber(candidate.timeToDraftWatchMs)
      ? candidate.timeToDraftWatchMs
      : undefined,
    timeToDraftActionMs: isFiniteNumber(candidate.timeToDraftActionMs)
      ? candidate.timeToDraftActionMs
      : undefined,
    copyPerSaveWatch: isFiniteNumber(candidate.copyPerSaveWatch)
      ? candidate.copyPerSaveWatch
      : undefined,
    copyPerSaveAction: isFiniteNumber(candidate.copyPerSaveAction)
      ? candidate.copyPerSaveAction
      : undefined,
    editedSaveRateWatch: isFiniteNumber(candidate.editedSaveRateWatch)
      ? candidate.editedSaveRateWatch
      : undefined,
    editedSaveRateAction: isFiniteNumber(candidate.editedSaveRateAction)
      ? candidate.editedSaveRateAction
      : undefined,
  };
}

function clamp01(value: number): number {
  return Math.min(1, Math.max(0, value));
}

function sanitizeThresholds(
  thresholds: Partial<ResponseQualityThresholds>,
): ResponseQualityThresholds {
  const merged: ResponseQualityThresholds = {
    ...cloneDefaultThresholds(),
    ...thresholds,
  };

  merged.editRatioWatch = clamp01(merged.editRatioWatch);
  merged.editRatioAction = clamp01(merged.editRatioAction);
  if (merged.editRatioWatch >= merged.editRatioAction) {
    merged.editRatioWatch = DEFAULT_RESPONSE_QUALITY_THRESHOLDS.editRatioWatch;
    merged.editRatioAction = DEFAULT_RESPONSE_QUALITY_THRESHOLDS.editRatioAction;
  }

  merged.timeToDraftWatchMs = Math.max(1, merged.timeToDraftWatchMs);
  merged.timeToDraftActionMs = Math.max(1, merged.timeToDraftActionMs);
  if (merged.timeToDraftWatchMs >= merged.timeToDraftActionMs) {
    merged.timeToDraftWatchMs =
      DEFAULT_RESPONSE_QUALITY_THRESHOLDS.timeToDraftWatchMs;
    merged.timeToDraftActionMs =
      DEFAULT_RESPONSE_QUALITY_THRESHOLDS.timeToDraftActionMs;
  }

  merged.copyPerSaveWatch = clamp01(merged.copyPerSaveWatch);
  merged.copyPerSaveAction = clamp01(merged.copyPerSaveAction);
  if (merged.copyPerSaveWatch <= merged.copyPerSaveAction) {
    merged.copyPerSaveWatch = DEFAULT_RESPONSE_QUALITY_THRESHOLDS.copyPerSaveWatch;
    merged.copyPerSaveAction = DEFAULT_RESPONSE_QUALITY_THRESHOLDS.copyPerSaveAction;
  }

  merged.editedSaveRateWatch = clamp01(merged.editedSaveRateWatch);
  merged.editedSaveRateAction = clamp01(merged.editedSaveRateAction);
  if (merged.editedSaveRateWatch >= merged.editedSaveRateAction) {
    merged.editedSaveRateWatch =
      DEFAULT_RESPONSE_QUALITY_THRESHOLDS.editedSaveRateWatch;
    merged.editedSaveRateAction =
      DEFAULT_RESPONSE_QUALITY_THRESHOLDS.editedSaveRateAction;
  }

  return merged;
}

function loadRawThresholds(): Partial<ResponseQualityThresholds> | null {
  if (typeof window === 'undefined' || !window.localStorage) {
    return null;
  }
  const raw = window.localStorage.getItem(RESPONSE_QUALITY_THRESHOLDS_STORAGE_KEY);
  if (!raw) {
    return null;
  }
  try {
    const parsed = JSON.parse(raw);
    return parseUnknownObject(parsed);
  } catch {
    return null;
  }
}

export function getResponseQualityThresholds(): ResponseQualityThresholds {
  const stored = loadRawThresholds();
  if (!stored) {
    return cloneDefaultThresholds();
  }
  return sanitizeThresholds(stored);
}

function dispatchThresholdsUpdated(thresholds: ResponseQualityThresholds): void {
  if (typeof window === 'undefined' || typeof window.dispatchEvent !== 'function') {
    return;
  }
  window.dispatchEvent(
    new CustomEvent(RESPONSE_QUALITY_THRESHOLDS_UPDATED_EVENT, {
      detail: { thresholds },
    }),
  );
}

export function saveResponseQualityThresholds(
  thresholds: Partial<ResponseQualityThresholds>,
): ResponseQualityThresholds {
  const next = sanitizeThresholds(thresholds);
  if (typeof window !== 'undefined' && window.localStorage) {
    window.localStorage.setItem(
      RESPONSE_QUALITY_THRESHOLDS_STORAGE_KEY,
      JSON.stringify(next),
    );
  }
  dispatchThresholdsUpdated(next);
  return next;
}

export function resetResponseQualityThresholds(): ResponseQualityThresholds {
  const defaults = cloneDefaultThresholds();
  if (typeof window !== 'undefined' && window.localStorage) {
    window.localStorage.removeItem(RESPONSE_QUALITY_THRESHOLDS_STORAGE_KEY);
  }
  dispatchThresholdsUpdated(defaults);
  return defaults;
}
