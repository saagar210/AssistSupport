import { describe, expect, it } from 'vitest';
import { buildResponseQualityCoaching } from './qualityCoaching';

describe('qualityCoaching', () => {
  it('returns null when snapshots are unavailable', () => {
    expect(buildResponseQualityCoaching(null)).toBeNull();
    expect(
      buildResponseQualityCoaching({
        snapshots_count: 0,
        avg_edit_ratio: 0,
        avg_time_to_draft_ms: null,
        copy_per_saved_ratio: 0,
        edited_save_rate: 0,
      }),
    ).toBeNull();
  });

  it('classifies healthy bands correctly', () => {
    const summary = buildResponseQualityCoaching({
      snapshots_count: 12,
      avg_edit_ratio: 0.12,
      avg_time_to_draft_ms: 72_000,
      copy_per_saved_ratio: 0.72,
      edited_save_rate: 0.54,
    });

    expect(summary?.overallSeverity).toBe('healthy');
    expect(summary?.signals.every((signal) => signal.severity === 'healthy')).toBe(true);
  });

  it('classifies mixed and action bands with action precedence', () => {
    const summary = buildResponseQualityCoaching({
      snapshots_count: 8,
      avg_edit_ratio: 0.4,
      avg_time_to_draft_ms: 120_000,
      copy_per_saved_ratio: 0.5,
      edited_save_rate: 0.9,
    });

    expect(summary?.overallSeverity).toBe('action');
    expect(summary?.signals.find((signal) => signal.id === 'edit_ratio')?.severity).toBe('action');
    expect(summary?.signals.find((signal) => signal.id === 'time_to_draft')?.severity).toBe('watch');
    expect(summary?.signals.find((signal) => signal.id === 'copy_per_save')?.severity).toBe('watch');
    expect(summary?.signals.find((signal) => signal.id === 'edited_save_rate')?.severity).toBe('action');
  });

  it('honors custom thresholds', () => {
    const summary = buildResponseQualityCoaching(
      {
        snapshots_count: 4,
        avg_edit_ratio: 0.25,
        avg_time_to_draft_ms: 75_000,
        copy_per_saved_ratio: 0.65,
        edited_save_rate: 0.62,
      },
      {
        editRatioWatch: 0.3,
        editRatioAction: 0.5,
        timeToDraftWatchMs: 80_000,
        timeToDraftActionMs: 120_000,
        copyPerSaveWatch: 0.7,
        copyPerSaveAction: 0.5,
        editedSaveRateWatch: 0.65,
        editedSaveRateAction: 0.9,
      },
    );

    expect(summary?.overallSeverity).toBe('watch');
    expect(summary?.signals.find((signal) => signal.id === 'edit_ratio')?.severity).toBe('healthy');
    expect(summary?.signals.find((signal) => signal.id === 'time_to_draft')?.severity).toBe('healthy');
    expect(summary?.signals.find((signal) => signal.id === 'copy_per_save')?.severity).toBe('watch');
    expect(summary?.signals.find((signal) => signal.id === 'edited_save_rate')?.severity).toBe('healthy');
  });
});
