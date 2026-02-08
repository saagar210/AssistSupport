import { describe, expect, it, beforeEach } from 'vitest';
import {
  DEFAULT_RESPONSE_QUALITY_THRESHOLDS,
  RESPONSE_QUALITY_THRESHOLDS_STORAGE_KEY,
  getResponseQualityThresholds,
  resetResponseQualityThresholds,
  saveResponseQualityThresholds,
} from './qualityThresholds';

describe('qualityThresholds', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('returns defaults when no settings are saved', () => {
    expect(getResponseQualityThresholds()).toEqual(
      DEFAULT_RESPONSE_QUALITY_THRESHOLDS,
    );
  });

  it('sanitizes invalid settings and falls back to defaults for bad pairs', () => {
    localStorage.setItem(
      RESPONSE_QUALITY_THRESHOLDS_STORAGE_KEY,
      JSON.stringify({
        editRatioWatch: 0.9,
        editRatioAction: 0.2,
        copyPerSaveWatch: 0.2,
        copyPerSaveAction: 0.8,
        editedSaveRateWatch: -1,
        editedSaveRateAction: 2,
      }),
    );

    const thresholds = getResponseQualityThresholds();
    expect(thresholds.editRatioWatch).toBe(
      DEFAULT_RESPONSE_QUALITY_THRESHOLDS.editRatioWatch,
    );
    expect(thresholds.editRatioAction).toBe(
      DEFAULT_RESPONSE_QUALITY_THRESHOLDS.editRatioAction,
    );
    expect(thresholds.copyPerSaveWatch).toBe(
      DEFAULT_RESPONSE_QUALITY_THRESHOLDS.copyPerSaveWatch,
    );
    expect(thresholds.copyPerSaveAction).toBe(
      DEFAULT_RESPONSE_QUALITY_THRESHOLDS.copyPerSaveAction,
    );
    expect(thresholds.editedSaveRateWatch).toBe(0);
    expect(thresholds.editedSaveRateAction).toBe(1);
  });

  it('persists and resets settings', () => {
    const saved = saveResponseQualityThresholds({
      editRatioWatch: 0.22,
      editRatioAction: 0.4,
      timeToDraftWatchMs: 80_000,
      timeToDraftActionMs: 150_000,
      copyPerSaveWatch: 0.7,
      copyPerSaveAction: 0.45,
      editedSaveRateWatch: 0.6,
      editedSaveRateAction: 0.9,
    });

    expect(saved.editRatioWatch).toBe(0.22);
    expect(getResponseQualityThresholds().timeToDraftActionMs).toBe(150_000);

    const reset = resetResponseQualityThresholds();
    expect(reset).toEqual(DEFAULT_RESPONSE_QUALITY_THRESHOLDS);
    expect(localStorage.getItem(RESPONSE_QUALITY_THRESHOLDS_STORAGE_KEY)).toBeNull();
  });
});
