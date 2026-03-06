import { describe, expect, it } from 'vitest';
import {
  classifySearchApiFailure,
  classifySearchApiHealthStatus,
  getMemoryKernelGuidance,
  normalizeDegradedReason,
} from './degradedSemantics';

describe('degradedSemantics', () => {
  it('normalizes degraded reason values consistently', () => {
    expect(normalizeDegradedReason('  OFFLINE  ')).toBe('offline');
    expect(normalizeDegradedReason(null)).toBe('unknown');
    expect(normalizeDegradedReason('')).toBe('unknown');
  });

  it('provides deterministic MemoryKernel guidance', () => {
    expect(getMemoryKernelGuidance('timeout')).toContain('timed out');
    expect(getMemoryKernelGuidance('version-mismatch')).toContain('contract mismatch');
    expect(getMemoryKernelGuidance('not-a-known-reason')).toContain('fallback is active');
  });

  it('classifies auth failures for Search API', () => {
    const classified = classifySearchApiFailure('Search API error (401): unauthorized');
    expect(classified.reason).toBe('auth-failure');
    expect(classified.status).toBe('degraded');
  });

  it('classifies rate limits for Search API', () => {
    const classified = classifySearchApiFailure('Search API error (429): too many requests');
    expect(classified.reason).toBe('rate-limit');
    expect(classified.status).toBe('degraded');
  });

  it('classifies offline transport failures for Search API', () => {
    const classified = classifySearchApiFailure('Search API unavailable: connect ECONNREFUSED');
    expect(classified.reason).toBe('offline');
    expect(classified.status).toBe('offline');
  });

  it('classifies wrong-service HTML responses for Search API', () => {
    const classified = classifySearchApiFailure('Health check failed: serving HTML on localhost');
    expect(classified.reason).toBe('wrong-service');
    expect(classified.status).toBe('degraded');
  });

  it('classifies timeout responses for Search API', () => {
    const classified = classifySearchApiFailure('request timed out after 5000ms');
    expect(classified.reason).toBe('timeout');
    expect(classified.status).toBe('offline');
  });

  it('maps Search API health status into deterministic guidance', () => {
    const classified = classifySearchApiHealthStatus('invalid-config', 'Invalid base url');
    expect(classified.reason).toBe('invalid-config');
    expect(classified.guidance).toContain('base URL configuration is invalid');
  });

  it('derives degraded reason from Search API health message when status is generic degraded', () => {
    const classified = classifySearchApiHealthStatus('degraded', 'Search API error (429): too many requests');
    expect(classified.reason).toBe('rate-limit');
    expect(classified.status).toBe('degraded');
  });

  it('returns healthy classification for healthy Search API status', () => {
    const classified = classifySearchApiHealthStatus('ok', 'Connected');
    expect(classified.reason).toBe('ok');
    expect(classified.status).toBe('healthy');
  });
});
