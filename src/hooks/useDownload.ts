import { useState, useCallback, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { DownloadProgress } from '../types';

export interface DownloadState {
  isDownloading: boolean;
  downloadProgress: DownloadProgress | null;
  error: string | null;
}

interface DownloadEvent {
  model_id: string;
  downloaded_bytes: number;
  total_bytes: number;
  percent: number;
  speed_bps: number;
}

export function useDownload() {
  const [state, setState] = useState<DownloadState>({
    isDownloading: false,
    downloadProgress: null,
    error: null,
  });

  // Listen for download progress events
  useEffect(() => {
    const unlisten = listen<DownloadEvent>('download-progress', (event) => {
      const progress = event.payload;
      setState(prev => ({
        ...prev,
        downloadProgress: {
          model_id: progress.model_id,
          percent: progress.percent,
          downloaded_bytes: progress.downloaded_bytes,
          total_bytes: progress.total_bytes,
          speed_bps: progress.speed_bps,
        },
        isDownloading: progress.percent < 100,
      }));
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, []);

  const downloadModel = useCallback(async (modelId: string): Promise<string> => {
    setState(prev => ({
      ...prev,
      isDownloading: true,
      downloadProgress: {
        model_id: modelId,
        percent: 0,
        downloaded_bytes: 0,
        total_bytes: 0,
        speed_bps: 0,
      },
      error: null,
    }));
    try {
      const path = await invoke<string>('download_model', { modelId });
      setState(prev => ({
        ...prev,
        isDownloading: false,
        downloadProgress: null,
      }));
      return path;
    } catch (e) {
      setState(prev => ({
        ...prev,
        isDownloading: false,
        downloadProgress: null,
        error: String(e),
      }));
      throw e;
    }
  }, []);

  return {
    ...state,
    downloadModel,
  };
}
