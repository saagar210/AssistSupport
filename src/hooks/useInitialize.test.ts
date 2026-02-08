import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { invoke } from '@tauri-apps/api/core';
import { useInitialize } from './useInitialize';

vi.mock('@tauri-apps/api/core');

const mockInvoke = vi.mocked(invoke);

type InvokeOverrides = Partial<Record<string, (args?: Record<string, unknown>) => unknown>>;

function installInvokeMocks(overrides: InvokeOverrides = {}) {
  mockInvoke.mockImplementation(async (command: string, args?: Record<string, unknown>) => {
    if (overrides[command]) {
      return overrides[command]?.(args);
    }

    switch (command) {
      case 'initialize_app':
        return {
          is_first_run: false,
          keychain_available: true,
          fts5_available: true,
          vector_store_ready: true,
        };
      case 'check_fts5_enabled':
        return true;
      case 'get_vector_consent':
        return { enabled: true, consented_at: '2026-01-01T00:00:00Z', encryption_supported: true };
      case 'get_memory_kernel_preflight_status':
        return {
          enabled: true,
          ready: true,
          enrichment_enabled: true,
          status: 'ready',
          message: 'MemoryKernel preflight passed',
          base_url: 'http://127.0.0.1:4010',
          service_contract_version: 'service.v2',
          api_contract_version: 'api.v1',
          expected_service_contract_version: 'service.v2',
          expected_api_contract_version: 'api.v1',
          integration_baseline: 'integration/v1',
          release_tag: 'v0.3.0',
          commit_sha: 'b9e1b397558dfba1fa8a4948fcf723ed4b505e1c',
        };
      case 'create_session_token':
        return 'session-token-new';
      case 'validate_session_token':
        return true;
      case 'init_llm_engine':
      case 'init_embedding_engine':
      case 'load_model':
      case 'load_embedding_model':
        return undefined;
      case 'get_model_state':
        return {
          llm_model_id: null,
          llm_model_path: null,
          llm_loaded: false,
          embeddings_model_path: null,
          embeddings_loaded: false,
        };
      default:
        return undefined;
    }
  });
}

describe('useInitialize', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    localStorage.clear();
    vi.spyOn(console, 'warn').mockImplementation(() => {});
    vi.spyOn(console, 'error').mockImplementation(() => {});
    vi.spyOn(console, 'log').mockImplementation(() => {});
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('initializes app, creates a session token, and marks engines ready', async () => {
    installInvokeMocks();

    const { result } = renderHook(() => useInitialize());

    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.initialized).toBe(true);
    expect(result.current.error).toBeNull();
    expect(result.current.vectorConsent?.enabled).toBe(true);
    expect(localStorage.getItem('assistsupport_session_token')).toBe('session-token-new');

    await waitFor(() => expect(result.current.enginesReady).toBe(true));
    expect(mockInvoke).toHaveBeenCalledWith('initialize_app');
    expect(mockInvoke).toHaveBeenCalledWith('check_fts5_enabled');
    expect(mockInvoke).toHaveBeenCalledWith('get_memory_kernel_preflight_status');
    expect(mockInvoke).toHaveBeenCalledWith('init_llm_engine');
    expect(mockInvoke).toHaveBeenCalledWith('init_embedding_engine');
  });

  it('replaces an invalid saved session token', async () => {
    localStorage.setItem('assistsupport_session_token', 'stale-token');
    installInvokeMocks({
      validate_session_token: () => false,
      create_session_token: () => 'session-token-rotated',
    });

    const { result } = renderHook(() => useInitialize());
    await waitFor(() => expect(result.current.loading).toBe(false));

    expect(result.current.initialized).toBe(true);
    expect(localStorage.getItem('assistsupport_session_token')).toBe('session-token-rotated');
    expect(mockInvoke).toHaveBeenCalledWith('validate_session_token', { sessionId: 'stale-token' });
  });

  it('falls back to safe vector consent defaults when consent lookup fails', async () => {
    installInvokeMocks({
      get_vector_consent: () => {
        throw new Error('consent unavailable');
      },
    });

    const { result } = renderHook(() => useInitialize());
    await waitFor(() => expect(result.current.loading).toBe(false));

    expect(result.current.initialized).toBe(true);
    expect(result.current.vectorConsent).toEqual({
      enabled: false,
      consented_at: null,
      encryption_supported: false,
    });
  });

  it('surfaces an error when FTS5 is unavailable', async () => {
    installInvokeMocks({
      check_fts5_enabled: () => false,
    });

    const { result } = renderHook(() => useInitialize());
    await waitFor(() => expect(result.current.loading).toBe(false));

    expect(result.current.initialized).toBe(false);
    expect(result.current.error).toContain('FTS5 full-text search is not available');
  });

  it('continues initialization when session token creation fails', async () => {
    installInvokeMocks({
      create_session_token: () => {
        throw new Error('token subsystem down');
      },
    });

    const { result } = renderHook(() => useInitialize());
    await waitFor(() => expect(result.current.loading).toBe(false));

    expect(result.current.initialized).toBe(true);
    expect(result.current.error).toBeNull();
    expect(localStorage.getItem('assistsupport_session_token')).toBeNull();
  });

  it('continues initialization when MemoryKernel preflight fails', async () => {
    installInvokeMocks({
      get_memory_kernel_preflight_status: () => {
        throw new Error('service unavailable');
      },
    });

    const { result } = renderHook(() => useInitialize());
    await waitFor(() => expect(result.current.loading).toBe(false));

    expect(result.current.initialized).toBe(true);
    expect(result.current.error).toBeNull();
    expect(result.current.memoryKernelPreflight).toBeNull();
  });
});
