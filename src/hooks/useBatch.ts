import { useState, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface BatchResult {
  input: string;
  response: string;
  sources: Array<{
    chunk_id: string;
    document_id: string;
    file_path: string;
    title: string | null;
    heading_path: string | null;
    score: number;
  }>;
  duration_ms: number;
}

export interface BatchStatus {
  job_id: string;
  status: string; // 'queued' | 'running' | 'succeeded' | 'failed' | 'cancelled'
  total: number;
  completed: number;
  results: BatchResult[];
  error: string | null;
}

export function useBatch() {
  const [status, setStatus] = useState<BatchStatus | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const pollRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const stopPolling = useCallback(() => {
    if (pollRef.current) {
      clearInterval(pollRef.current);
      pollRef.current = null;
    }
  }, []);

  const startBatch = useCallback(async (inputs: string[], responseLength: string): Promise<string> => {
    setLoading(true);
    setError(null);
    try {
      const jobId = await invoke<string>('batch_generate', { inputs, responseLength });

      // Start polling for status
      const poll = async () => {
        try {
          const s = await invoke<BatchStatus>('get_batch_status', { jobId });
          setStatus(s);
          if (s.status === 'succeeded' || s.status === 'failed' || s.status === 'cancelled') {
            stopPolling();
            setLoading(false);
          }
        } catch (err) {
          console.error('Poll error:', err);
        }
      };

      // Initial poll
      await poll();

      // Poll every 2 seconds
      pollRef.current = setInterval(poll, 2000);

      return jobId;
    } catch (err) {
      setError(String(err));
      setLoading(false);
      throw err;
    }
  }, [stopPolling]);

  const getBatchStatus = useCallback(async (jobId: string): Promise<BatchStatus> => {
    return invoke<BatchStatus>('get_batch_status', { jobId });
  }, []);

  const exportResults = useCallback(async (jobId: string, format: string): Promise<boolean> => {
    try {
      return await invoke<boolean>('export_batch_results', { jobId, format });
    } catch (err) {
      setError(String(err));
      return false;
    }
  }, []);

  const cancelBatch = useCallback(async () => {
    if (status?.job_id) {
      try {
        await invoke('cancel_job', { jobId: status.job_id });
        stopPolling();
        setLoading(false);
      } catch (err) {
        setError(String(err));
      }
    }
  }, [status, stopPolling]);

  const reset = useCallback(() => {
    stopPolling();
    setStatus(null);
    setLoading(false);
    setError(null);
  }, [stopPolling]);

  return {
    status,
    loading,
    error,
    startBatch,
    getBatchStatus,
    exportResults,
    cancelBatch,
    reset,
  };
}
