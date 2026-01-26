import { useState, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

import type {
  ModelInfo,
  GenerationParams,
  GenerationResult,
  GenerateWithContextParams,
  GenerateWithContextResult,
  ResponseLength,
  StreamToken,
  TreeDecisions,
  JiraTicketContext,
  GgufFileInfo,
  FirstResponseParams,
  FirstResponseResult,
  ChecklistGenerateParams,
  ChecklistUpdateParams,
  ChecklistResult,
} from '../types';

// Maximum streaming text buffer size (500KB) to prevent memory spikes
const MAX_STREAMING_TEXT_SIZE = 500 * 1024;

export interface LlmState {
  modelInfo: ModelInfo | null;
  isLoaded: boolean;
  loading: boolean;
  generating: boolean;
  error: string | null;
  streamingText: string;
  isStreaming: boolean;
}

export function useLlm() {
  const [state, setState] = useState<LlmState>({
    modelInfo: null,
    isLoaded: false,
    loading: false,
    generating: false,
    error: null,
    streamingText: '',
    isStreaming: false,
  });
  const unlistenRef = useRef<UnlistenFn | null>(null);

  const checkModelStatus = useCallback(async () => {
    try {
      const isLoaded = await invoke<boolean>('is_model_loaded');
      if (isLoaded) {
        const info = await invoke<ModelInfo | null>('get_model_info');
        setState(prev => ({
          ...prev,
          isLoaded: true,
          modelInfo: info,
          error: null,
        }));
      } else {
        setState(prev => ({
          ...prev,
          isLoaded: false,
          modelInfo: null,
        }));
      }
    } catch (e) {
      setState(prev => ({
        ...prev,
        error: String(e),
      }));
    }
  }, []);

  const getLoadedModel = useCallback(async (): Promise<string | null> => {
    try {
      const isLoaded = await invoke<boolean>('is_model_loaded');
      if (isLoaded) {
        const info = await invoke<ModelInfo | null>('get_model_info');
        return info?.id ?? info?.name ?? null;
      }
      return null;
    } catch {
      return null;
    }
  }, []);

  const listModels = useCallback(async (): Promise<string[]> => {
    try {
      const models = await invoke<string[]>('list_downloaded_models');
      return models;
    } catch {
      return [];
    }
  }, []);

  const loadModel = useCallback(async (modelId: string, nGpuLayers?: number) => {
    setState(prev => ({ ...prev, loading: true, error: null }));
    try {
      const info = await invoke<ModelInfo>('load_model', {
        modelId,
        nGpuLayers: nGpuLayers ?? 1000,
      });
      setState(prev => ({
        ...prev,
        loading: false,
        isLoaded: true,
        modelInfo: info,
      }));
      return info;
    } catch (e) {
      setState(prev => ({
        ...prev,
        loading: false,
        error: String(e),
      }));
      throw e;
    }
  }, []);

  const unloadModel = useCallback(async () => {
    try {
      await invoke('unload_model');
      setState(prev => ({
        ...prev,
        isLoaded: false,
        modelInfo: null,
      }));
    } catch (e) {
      setState(prev => ({
        ...prev,
        error: String(e),
      }));
      throw e;
    }
  }, []);

  const generate = useCallback(async (
    prompt: string,
    params?: GenerationParams
  ): Promise<GenerationResult> => {
    setState(prev => ({ ...prev, generating: true, error: null }));
    try {
      const result = await invoke<GenerationResult>('generate_text', {
        prompt,
        params,
      });
      setState(prev => ({ ...prev, generating: false }));
      return result;
    } catch (e) {
      setState(prev => ({
        ...prev,
        generating: false,
        error: String(e),
      }));
      throw e;
    }
  }, []);

  const generateWithContext = useCallback(async (
    query: string,
    responseLength: ResponseLength = 'Medium'
  ): Promise<GenerateWithContextResult> => {
    setState(prev => ({ ...prev, generating: true, error: null }));
    try {
      const params: GenerateWithContextParams = {
        user_input: query,
        response_length: responseLength,
      };
      const result = await invoke<GenerateWithContextResult>('generate_with_context', {
        params,
      });
      setState(prev => ({ ...prev, generating: false }));
      return result;
    } catch (e) {
      setState(prev => ({
        ...prev,
        generating: false,
        error: String(e),
      }));
      throw e;
    }
  }, []);

  const generateWithContextParams = useCallback(async (
    params: GenerateWithContextParams
  ): Promise<GenerateWithContextResult> => {
    setState(prev => ({ ...prev, generating: true, error: null }));
    try {
      const result = await invoke<GenerateWithContextResult>('generate_with_context', {
        params,
      });
      setState(prev => ({ ...prev, generating: false }));
      return result;
    } catch (e) {
      setState(prev => ({
        ...prev,
        generating: false,
        error: String(e),
      }));
      throw e;
    }
  }, []);

  const generateFirstResponse = useCallback(async (
    params: FirstResponseParams
  ): Promise<FirstResponseResult> => {
    setState(prev => ({ ...prev, generating: true, error: null }));
    try {
      const result = await invoke<FirstResponseResult>('generate_first_response', {
        params,
      });
      setState(prev => ({ ...prev, generating: false }));
      return result;
    } catch (e) {
      setState(prev => ({
        ...prev,
        generating: false,
        error: String(e),
      }));
      throw e;
    }
  }, []);

  const generateChecklist = useCallback(async (
    params: ChecklistGenerateParams
  ): Promise<ChecklistResult> => {
    setState(prev => ({ ...prev, generating: true, error: null }));
    try {
      const result = await invoke<ChecklistResult>('generate_troubleshooting_checklist', {
        params,
      });
      setState(prev => ({ ...prev, generating: false }));
      return result;
    } catch (e) {
      setState(prev => ({
        ...prev,
        generating: false,
        error: String(e),
      }));
      throw e;
    }
  }, []);

  const updateChecklist = useCallback(async (
    params: ChecklistUpdateParams
  ): Promise<ChecklistResult> => {
    setState(prev => ({ ...prev, generating: true, error: null }));
    try {
      const result = await invoke<ChecklistResult>('update_troubleshooting_checklist', {
        params,
      });
      setState(prev => ({ ...prev, generating: false }));
      return result;
    } catch (e) {
      setState(prev => ({
        ...prev,
        generating: false,
        error: String(e),
      }));
      throw e;
    }
  }, []);

  const testModel = useCallback(async () => {
    setState(prev => ({ ...prev, generating: true, error: null }));
    try {
      const result = await invoke<{ text: string; tokens_generated: number; duration_ms: number }>('test_model');
      setState(prev => ({ ...prev, generating: false }));
      return result;
    } catch (e) {
      setState(prev => ({
        ...prev,
        generating: false,
        error: String(e),
      }));
      throw e;
    }
  }, []);

  const generateStreaming = useCallback(async (
    query: string,
    responseLength: ResponseLength = 'Medium',
    options?: {
      onToken?: (token: string) => void;
      treeDecisions?: TreeDecisions;
      diagnosticNotes?: string;
      jiraTicket?: JiraTicketContext;
    }
  ): Promise<GenerateWithContextResult> => {
    // Clean up any previous listener
    if (unlistenRef.current) {
      unlistenRef.current();
      unlistenRef.current = null;
    }

    setState(prev => ({
      ...prev,
      generating: true,
      isStreaming: true,
      streamingText: '',
      error: null,
    }));

    // Set up token listener with size limit to prevent memory spikes
    const unlisten = await listen<StreamToken>('llm-token', (event) => {
      if (event.payload.done) {
        setState(prev => ({ ...prev, isStreaming: false }));
      } else {
        setState(prev => {
          // Enforce maximum buffer size to prevent memory exhaustion
          const newText = prev.streamingText + event.payload.token;
          if (newText.length > MAX_STREAMING_TEXT_SIZE) {
            // Truncate from the beginning, keeping recent content
            const truncated = newText.slice(newText.length - MAX_STREAMING_TEXT_SIZE);
            return {
              ...prev,
              streamingText: '...[truncated]...' + truncated,
            };
          }
          return {
            ...prev,
            streamingText: newText,
          };
        });
        options?.onToken?.(event.payload.token);
      }
    });
    unlistenRef.current = unlisten;

    try {
      const params: GenerateWithContextParams = {
        user_input: query,
        response_length: responseLength,
        diagnostic_notes: options?.diagnosticNotes,
        tree_decisions: options?.treeDecisions,
        jira_ticket: options?.jiraTicket,
      };
      const result = await invoke<GenerateWithContextResult>('generate_streaming', {
        params,
      });
      setState(prev => ({
        ...prev,
        generating: false,
        isStreaming: false,
      }));
      return result;
    } catch (e) {
      setState(prev => ({
        ...prev,
        generating: false,
        isStreaming: false,
        error: String(e),
      }));
      throw e;
    } finally {
      if (unlistenRef.current) {
        unlistenRef.current();
        unlistenRef.current = null;
      }
    }
  }, []);

  const clearStreamingText = useCallback(() => {
    setState(prev => ({ ...prev, streamingText: '' }));
  }, []);

  const cancelGeneration = useCallback(async () => {
    try {
      await invoke('cancel_generation');
      setState(prev => ({
        ...prev,
        generating: false,
        isStreaming: false,
      }));
    } catch (e) {
      console.error('Failed to cancel generation:', e);
    }
  }, []);

  const getContextWindow = useCallback(async (): Promise<number | null> => {
    try {
      const size = await invoke<number | null>('get_context_window');
      return size;
    } catch {
      return null;
    }
  }, []);

  const setContextWindow = useCallback(async (size: number | null) => {
    try {
      await invoke('set_context_window', { size });
    } catch (e) {
      console.error('Failed to set context window:', e);
      throw e;
    }
  }, []);

  const validateGgufFile = useCallback(async (modelPath: string): Promise<GgufFileInfo> => {
    try {
      const info = await invoke<GgufFileInfo>('validate_gguf_file', { modelPath });
      return info;
    } catch (e) {
      throw e;
    }
  }, []);

  const loadCustomModel = useCallback(async (modelPath: string, nGpuLayers?: number): Promise<ModelInfo> => {
    setState(prev => ({ ...prev, loading: true, error: null }));
    try {
      // First validate the file
      const validation = await validateGgufFile(modelPath);
      if (!validation.is_valid) {
        throw new Error(`Invalid GGUF file: ${validation.file_name}`);
      }

      const info = await invoke<ModelInfo>('load_custom_model', {
        modelPath,
        nGpuLayers: nGpuLayers ?? 1000,
      });
      setState(prev => ({
        ...prev,
        loading: false,
        isLoaded: true,
        modelInfo: info,
      }));
      return info;
    } catch (e) {
      setState(prev => ({
        ...prev,
        loading: false,
        error: String(e),
      }));
      throw e;
    }
  }, [validateGgufFile]);

  return {
    ...state,
    checkModelStatus,
    getLoadedModel,
    listModels,
    loadModel,
    loadCustomModel,
    validateGgufFile,
    unloadModel,
    generate,
    generateWithContext,
    generateWithContextParams,
    generateStreaming,
    clearStreamingText,
    cancelGeneration,
    testModel,
    getContextWindow,
    setContextWindow,
    generateFirstResponse,
    generateChecklist,
    updateChecklist,
  };
}
