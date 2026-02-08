import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook } from '@testing-library/react';
import { invoke } from '@tauri-apps/api/core';
import { useMemoryKernelEnrichment } from './useMemoryKernelEnrichment';

vi.mock('@tauri-apps/api/core');

const mockInvoke = vi.mocked(invoke);

describe('useMemoryKernelEnrichment', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('adds MemoryKernel section when enrichment is applied', async () => {
    mockInvoke.mockResolvedValue({
      applied: true,
      status: 'applied',
      message: 'ok',
      fallback_reason: null,
      machine_error_code: null,
      context_package_id: 'ctx_1',
      enrichment_text: 'Policy decision: allow',
      preflight: {
        enabled: true,
        ready: true,
        enrichment_enabled: true,
        status: 'ready',
        message: 'ready',
        base_url: 'http://127.0.0.1:4010',
        service_contract_version: 'service.v2',
        api_contract_version: 'api.v1',
        expected_service_contract_version: 'service.v2',
        expected_api_contract_version: 'api.v1',
        integration_baseline: 'integration/v1',
        release_tag: 'v0.3.2',
        commit_sha: 'cf331449e1589581a5dcbb3adecd3e9ae4509277',
      },
    } as never);

    const { result } = renderHook(() => useMemoryKernelEnrichment());
    const enriched = await result.current.enrichDiagnosticNotes('Need USB guidance', 'Checked logs');

    expect(enriched.enrichmentApplied).toBe(true);
    expect(enriched.fallbackReason).toBeNull();
    expect(enriched.machineErrorCode).toBeNull();
    expect(enriched.diagnosticNotes).toContain('Checked logs');
    expect(enriched.diagnosticNotes).toContain('MemoryKernel Policy Context');
    expect(enriched.diagnosticNotes).toContain('Policy decision: allow');
  });

  it('returns original notes when enrichment falls back', async () => {
    mockInvoke.mockResolvedValue({
      applied: false,
      status: 'fallback',
      message: 'service unavailable',
      fallback_reason: 'offline',
      machine_error_code: null,
      context_package_id: null,
      enrichment_text: null,
      preflight: {
        enabled: true,
        ready: false,
        enrichment_enabled: false,
        status: 'offline',
        message: 'offline',
        base_url: 'http://127.0.0.1:4010',
        service_contract_version: null,
        api_contract_version: null,
        expected_service_contract_version: 'service.v2',
        expected_api_contract_version: 'api.v1',
        integration_baseline: 'integration/v1',
        release_tag: 'v0.3.2',
        commit_sha: 'cf331449e1589581a5dcbb3adecd3e9ae4509277',
      },
    } as never);

    const { result } = renderHook(() => useMemoryKernelEnrichment());
    const enriched = await result.current.enrichDiagnosticNotes('Need USB guidance', 'Checked logs');

    expect(enriched.enrichmentApplied).toBe(false);
    expect(enriched.status).toBe('fallback');
    expect(enriched.fallbackReason).toBe('offline');
    expect(enriched.diagnosticNotes).toBe('Checked logs');
  });

  it('falls back deterministically when adapter command errors', async () => {
    mockInvoke.mockRejectedValue(new Error('network timeout'));

    const { result } = renderHook(() => useMemoryKernelEnrichment());
    const enriched = await result.current.enrichDiagnosticNotes('Need USB guidance');

    expect(enriched.enrichmentApplied).toBe(false);
    expect(enriched.status).toBe('fallback');
    expect(enriched.fallbackReason).toBe('adapter-error');
    expect(enriched.diagnosticNotes).toBeUndefined();
    expect(enriched.message).toContain('MemoryKernel enrichment unavailable');
  });
});
