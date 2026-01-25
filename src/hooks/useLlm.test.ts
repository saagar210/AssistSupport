import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useLlm } from './useLlm';
import { invoke } from '@tauri-apps/api/core';

vi.mock('@tauri-apps/api/core');
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

const mockInvoke = vi.mocked(invoke);

describe('useLlm', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('initializes with default state', () => {
    const { result } = renderHook(() => useLlm());

    expect(result.current.modelInfo).toBeNull();
    expect(result.current.isLoaded).toBe(false);
    expect(result.current.loading).toBe(false);
    expect(result.current.generating).toBe(false);
    expect(result.current.error).toBeNull();
    expect(result.current.streamingText).toBe('');
    expect(result.current.isStreaming).toBe(false);
  });

  it('checkModelStatus updates state when model is loaded', async () => {
    const mockModelInfo = {
      id: 'test-model',
      path: '/models/test.gguf',
      name: 'test.gguf',
      size_bytes: 1000000,
      n_params: 1000000,
      n_ctx_train: 2048,
      n_embd: 768,
      n_vocab: 32000,
    };

    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === 'is_model_loaded') return true;
      if (cmd === 'get_model_info') return mockModelInfo;
      return undefined;
    });

    const { result } = renderHook(() => useLlm());

    await act(async () => {
      await result.current.checkModelStatus();
    });

    expect(result.current.isLoaded).toBe(true);
    expect(result.current.modelInfo).toEqual(mockModelInfo);
    expect(result.current.error).toBeNull();
  });

  it('checkModelStatus updates state when no model loaded', async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === 'is_model_loaded') return false;
      return undefined;
    });

    const { result } = renderHook(() => useLlm());

    await act(async () => {
      await result.current.checkModelStatus();
    });

    expect(result.current.isLoaded).toBe(false);
    expect(result.current.modelInfo).toBeNull();
  });

  it('loadModel sets loading state and updates on success', async () => {
    const mockModelInfo = {
      id: 'test-model',
      path: '/models/test.gguf',
      name: 'test.gguf',
      size_bytes: 1000000,
      n_params: 1000000,
      n_ctx_train: 2048,
      n_embd: 768,
      n_vocab: 32000,
    };

    mockInvoke.mockResolvedValue(mockModelInfo);

    const { result } = renderHook(() => useLlm());

    let loadPromise: Promise<unknown>;
    act(() => {
      loadPromise = result.current.loadModel('test-model');
    });

    // Check loading state
    expect(result.current.loading).toBe(true);

    await act(async () => {
      await loadPromise;
    });

    expect(result.current.loading).toBe(false);
    expect(result.current.isLoaded).toBe(true);
    expect(result.current.modelInfo).toEqual(mockModelInfo);
  });

  it('loadModel sets error on failure', async () => {
    mockInvoke.mockRejectedValue(new Error('Model not found'));

    const { result } = renderHook(() => useLlm());

    await act(async () => {
      try {
        await result.current.loadModel('invalid-model');
      } catch {
        // Expected to throw
      }
    });

    expect(result.current.loading).toBe(false);
    expect(result.current.error).toContain('Model not found');
  });

  it('unloadModel clears model state', async () => {
    mockInvoke.mockResolvedValue(undefined);

    const { result } = renderHook(() => useLlm());

    // Set initial state as if model was loaded
    await act(async () => {
      await result.current.unloadModel();
    });

    expect(result.current.isLoaded).toBe(false);
    expect(result.current.modelInfo).toBeNull();
  });

  it('cancelGeneration invokes cancel command', async () => {
    mockInvoke.mockResolvedValue(undefined);

    const { result } = renderHook(() => useLlm());

    await act(async () => {
      await result.current.cancelGeneration();
    });

    expect(mockInvoke).toHaveBeenCalledWith('cancel_generation');
    expect(result.current.generating).toBe(false);
    expect(result.current.isStreaming).toBe(false);
  });

  it('getContextWindow returns context size', async () => {
    mockInvoke.mockResolvedValue(4096);

    const { result } = renderHook(() => useLlm());

    let contextWindow: number | null = null;
    await act(async () => {
      contextWindow = await result.current.getContextWindow();
    });

    expect(contextWindow).toBe(4096);
    expect(mockInvoke).toHaveBeenCalledWith('get_context_window');
  });

  it('setContextWindow invokes set command', async () => {
    mockInvoke.mockResolvedValue(undefined);

    const { result } = renderHook(() => useLlm());

    await act(async () => {
      await result.current.setContextWindow(8192);
    });

    expect(mockInvoke).toHaveBeenCalledWith('set_context_window', { size: 8192 });
  });
});
