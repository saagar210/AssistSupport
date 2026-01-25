import { useState, useCallback, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { DownloadProgress } from '../types';

export interface DownloadState {
  isDownloading: boolean;
  downloadProgress: DownloadProgress | null;
  error: string | null;
}

type DownloadProgressEvent =
  | { Started: { url: string; total_bytes: number | null } }
  | { Progress: { downloaded: number; total: number | null; speed_bps: number } }
  | { Completed: { path: string; sha256: string } }
  | { Error: { message: string } }
  | 'Cancelled';

export function useDownload() {
  const [state, setState] = useState<DownloadState>({
    isDownloading: false,
    downloadProgress: null,
    error: null,
  });
  const currentModelIdRef = useRef<string | null>(null);

  // Listen for download progress events
  useEffect(() => {
    const unlisten = listen<DownloadProgressEvent>('download-progress', (event) => {
      const payload = event.payload;
      const modelId = currentModelIdRef.current ?? 'unknown';

      if (payload === 'Cancelled') {
        setState(prev => ({
          ...prev,
          isDownloading: false,
          error: 'Download cancelled',
        }));
        return;
      }

      if ('Error' in payload) {
        setState(prev => ({
          ...prev,
          isDownloading: false,
          error: payload.Error.message,
        }));
        return;
      }

      if ('Started' in payload) {
        const totalBytes = payload.Started.total_bytes ?? 0;
        setState(prev => ({
          ...prev,
          downloadProgress: {
            model_id: modelId,
            percent: 0,
            downloaded_bytes: 0,
            total_bytes: totalBytes,
            speed_bps: 0,
          },
          isDownloading: true,
        }));
        return;
      }

      if ('Progress' in payload) {
        const { downloaded, total, speed_bps } = payload.Progress;
        const totalBytes = total ?? 0;
        const percent = totalBytes > 0 ? (downloaded / totalBytes) * 100 : 0;
        setState(prev => ({
          ...prev,
          downloadProgress: {
            model_id: modelId,
            percent,
            downloaded_bytes: downloaded,
            total_bytes: totalBytes,
            speed_bps,
          },
          isDownloading: true,
        }));
        return;
      }

      if ('Completed' in payload) {
        setState(prev => ({
          ...prev,
          downloadProgress: prev.downloadProgress ? {
            ...prev.downloadProgress,
            model_id: modelId,
            percent: 100,
            speed_bps: 0,
          } : {
            model_id: modelId,
            percent: 100,
            downloaded_bytes: 0,
            total_bytes: 0,
            speed_bps: 0,
          },
          isDownloading: false,
        }));
      }
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, []);

  const downloadModel = useCallback(async (modelId: string): Promise<string> => {
    currentModelIdRef.current = modelId;
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
