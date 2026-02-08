import { describe, expect, it } from 'vitest';
import { calculateEditRatio, countWords } from './qualityMetrics';

describe('qualityMetrics', () => {
  it('counts words from user-facing content', () => {
    expect(countWords('')).toBe(0);
    expect(countWords('  hello world  ')).toBe(2);
    expect(countWords('one\ntwo   three')).toBe(3);
  });

  it('calculates normalized edit ratio', () => {
    expect(calculateEditRatio('', '')).toBe(0);
    expect(calculateEditRatio('same', 'same')).toBe(0);
    expect(calculateEditRatio('abc', 'axc')).toBeCloseTo(1 / 3, 5);
    expect(calculateEditRatio('short', 'completely different')).toBeGreaterThan(0.6);
  });

  it('uses bounded computation for long responses', () => {
    const original = Array.from({ length: 900 }, (_, i) => `token-${i}`).join(' ');
    const current = `${original} extra-token`;
    const ratio = calculateEditRatio(original, current);
    expect(ratio).toBeGreaterThanOrEqual(0);
    expect(ratio).toBeLessThan(0.05);
  });
});
