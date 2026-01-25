import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { EmbeddingModelInfo } from '../types';

export interface EmbeddingState {
  modelInfo: EmbeddingModelInfo | null;
  isLoaded: boolean;
  loading: boolean;
  error: string | null;
}

export function useEmbedding() {
  const [state, setState] = useState<EmbeddingState>({
    modelInfo: null,
    isLoaded: false,
    loading: false,
    error: null,
  });

  const initEngine = useCallback(async () => {
    try {
      await invoke('init_embedding_engine');
    } catch (e) {
      setState(prev => ({ ...prev, error: String(e) }));
      throw e;
    }
  }, []);

  const checkModelStatus = useCallback(async () => {
    try {
      const isLoaded = await invoke<boolean>('is_embedding_model_loaded');
      if (isLoaded) {
        const info = await invoke<EmbeddingModelInfo | null>('get_embedding_model_info');
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
      return isLoaded;
    } catch (e) {
      setState(prev => ({ ...prev, error: String(e) }));
      return false;
    }
  }, []);

  const loadModel = useCallback(async (modelPath: string, nGpuLayers?: number) => {
    setState(prev => ({ ...prev, loading: true, error: null }));
    try {
      const info = await invoke<EmbeddingModelInfo>('load_embedding_model', {
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
  }, []);

  const unloadModel = useCallback(async () => {
    try {
      await invoke('unload_embedding_model');
      setState(prev => ({
        ...prev,
        isLoaded: false,
        modelInfo: null,
      }));
    } catch (e) {
      setState(prev => ({ ...prev, error: String(e) }));
      throw e;
    }
  }, []);

  const getModelInfo = useCallback(async (): Promise<EmbeddingModelInfo | null> => {
    try {
      return await invoke<EmbeddingModelInfo | null>('get_embedding_model_info');
    } catch {
      return null;
    }
  }, []);

  const isModelLoaded = useCallback(async (): Promise<boolean> => {
    try {
      return await invoke<boolean>('is_embedding_model_loaded');
    } catch {
      return false;
    }
  }, []);

  const isModelDownloaded = useCallback(async (): Promise<boolean> => {
    try {
      return await invoke<boolean>('is_embedding_model_downloaded');
    } catch {
      return false;
    }
  }, []);

  const getModelPath = useCallback(async (modelId: string): Promise<string | null> => {
    try {
      return await invoke<string | null>('get_embedding_model_path', { modelId });
    } catch {
      return null;
    }
  }, []);

  return {
    ...state,
    initEngine,
    checkModelStatus,
    loadModel,
    unloadModel,
    getModelInfo,
    isModelLoaded,
    isModelDownloaded,
    getModelPath,
  };
}
